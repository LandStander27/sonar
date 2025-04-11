pub mod prelude;
mod packet;

use packet::{icmp, ipv4};
use logger::prelude::*;

use std::{
	io::Read, net::{
		IpAddr,
		Ipv4Addr,
		SocketAddr
	},
	str::FromStr,
	sync::mpsc::Receiver,
	time::{
		Duration,
		Instant
	}
};

use socket2::{
	Socket,
	Domain,
	Type,
	Protocol
};

pub struct PingReply {
	pub elapsed: Duration,
	pub sequence: u16,
	
	pub from_addr: Ipv4Addr,
	pub dest_addr: Ipv4Addr,
}

pub struct Pinger {
	addr: IpAddr,
	sequence: u16,
	
	socket: Option<Socket>,
	rand: fastrand::Rng,
	
	timeout: Duration,
}

impl Pinger {
	pub fn new<S: Into<String>>(addr: S) -> Result<Self, String> {
		let timeout = Duration::from_secs(2);
		debug!(timeout_secs = timeout.as_secs_f32());
		
		let addr = addr.into();
		let addr = match IpAddr::from_str(&addr) {
			Ok(a) => a,
			Err(e) => {
				if let Ok(o) = util::dns_lookup(addr + ":0") {
					o
				} else {
					return Err(e.to_string());
				}
			}
		};

		return Ok(Self {
			addr,
			sequence: 1,
			socket: None,
			rand: fastrand::Rng::new(),
			timeout,
		});
	}
	
	pub fn get_dest(&self) -> String {
		return self.addr.to_string();
	}
	
	pub fn init_socket(&mut self) -> Result<(), String> {
		let socket = match Socket::new(Domain::IPV4, Type::RAW, Some(Protocol::ICMPV4)) {
			Ok(s) => s,
			Err(e) => {
				error!(desc = e.to_string(), "could not open socket");
				return Err("could not open socket".to_string());
			}
		};
		
		if let Err(e) = socket.set_ttl(255) {
			error!(desc = e.to_string(), "could not set socket ttl");
			return Err("could not set socket ttl".to_string());
		}
		assert!(socket.ttl().is_ok());
		trace!(ttl = socket.ttl().unwrap());
		
		self.socket = Some(socket);
		return Ok(());
	}
	
	pub fn ping<T>(&mut self, rx: &Receiver<T>) -> Result<PingReply, String> {
		if self.socket.is_none() {
			return Err("invalid socket".to_string());
		}
		let socket = self.socket.as_mut().unwrap();

		let mut packet = icmp::ICMPPacket {
			typ: 8,
			code: 0,
			checksum: 0,
			
			ident: self.rand.u16(..),
			sequence: self.sequence,
			payload: ([0; 64]).into_iter().map(|_| self.rand.u8(..)).collect(),
		};
		
		packet.checksum = packet.calculate_checksum();
		trace!(packet.checksum, packet.ident, packet.sequence);
		
		let mut buffer = [0; 72];
		if let Err(e) = packet.encode(&mut buffer) {
			error!(desc = e.to_string(), "could not encode packet");
			return Err("could not encode packet".to_string());
		}
		let addr = SocketAddr::new(self.addr, 0);
		
		let start_time = Instant::now();
		let bytes = match socket.send_to(&buffer, &addr.into()) {
			Ok(b) => b,
			Err(e) => {
				error!(desc = e.to_string(), "Socket::send_to");
				return Err("Socket::send_to".to_string());
			}
		};
		debug!(bytes_sent = bytes);
		self.sequence += 1;

		let mut elapsed = Duration::from_secs(0);
		loop {
			if rx.try_recv().is_ok() {
				return Err("stop signal".to_string());
			}
			
			if let Err(e) = socket.set_read_timeout(Some(self.timeout - elapsed)) {
				error!(desc = e.to_string(), "could not set socket read timeout");
				return Err("could not set socket read timeout".to_string());
			}
			
			let mut buffer: [u8; 2048] = [0; 2048];
			let bytes = match socket.read(&mut buffer) {
				Ok(b) => b,
				Err(e) => {
					error!(desc = e.to_string(), "could not read from socket");
					return Err("could not read from socket".to_string());
				}
			};
			
			debug!(bytes_recv = bytes);
			
			let ipv4_packet = match ipv4::IPV4Packet::decode(&buffer) {
				Ok(p) => p,
				Err(e) => {
					error!(desc = e.to_string(), "could not decode packet");
					return Err("could not decode packet".to_string());
				}
			};

			let reply = match icmp::ICMPPacket::decode(&ipv4_packet.data) {
				Ok(r) => r,
				Err(_) => continue,
			};

			if reply.ident == packet.ident && reply.sequence == packet.sequence {
				return Ok(PingReply {
					elapsed: start_time.elapsed(),
					sequence: reply.sequence,
					
					from_addr: Ipv4Addr::from_bits(ipv4_packet.from_addr),
					dest_addr: Ipv4Addr::from_bits(ipv4_packet.dest_addr),
				});
			}
			
			elapsed = start_time.elapsed();
			if elapsed >= self.timeout {
				return Err("timeout".to_string());
			}
		}
	}
}
