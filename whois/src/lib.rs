
use logger::prelude::*;

use std::{
	io::Read, net::{
		IpAddr,
		SocketAddr,
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

pub mod prelude;

#[derive(Default)]
pub struct WhoIsResponse {
	pub netname: Option<String>
}

fn parse_whois<S: Into<String>>(s: S) -> WhoIsResponse {
	let s: String = s.into();
	let mut res = WhoIsResponse::default();

	for line in s.lines() {
		if res.netname.is_none() && line.to_lowercase().starts_with("netname") {
			res.netname = Some(line.split_whitespace().last().unwrap_or("Unknown").to_string());
		}
	}
	
	return res;
}

pub struct WhoIs {
	addr: IpAddr,
	// socket: Option<Socket>,
	timeout: Duration,
}

impl WhoIs {
	pub fn new<S: Into<String>>(addr: S) -> Result<Self, String> {
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
			timeout: Duration::from_secs(2),
			addr,
			// socket: None,
		});
	}
	
	pub fn get_whois<T>(&mut self, rx: &Receiver<T>) -> Result<WhoIsResponse, String> {
		let first = self.send_query("whois.iana.org", &rx)?;
		let server = if let Some(s) = first.split_whitespace().find(|s| s.starts_with("whois.")) {
			s
		} else {
			return Err("invalid response".to_string());
		};
		// let server = if let Some(line) = first.lines().find(|s| s.starts_with("whois:")) {
		// 	if let Some(s) = line.split_ascii_whitespace().last() {
		// 		s
		// 	} else {
		// 		return Err("invalid response".to_string());
		// 	}
		// } else {
		// 	return Err("invalid response".to_string());
		// };
		
		let result = self.send_query(server, &rx)?;
		return Ok(parse_whois(result));
	}
	
	fn send_query<S: Into<String>, T>(&mut self, server: S, rx: &Receiver<T>) -> Result<String, String> {
		let mut socket = match Socket::new(Domain::IPV4, Type::STREAM, Some(Protocol::TCP)) {
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
		
		let server: String = server.into();
		let iana_addr = util::dns_lookup(server.clone() + ":0")?;
		if let Err(e) = socket.connect_timeout(&SocketAddr::new(iana_addr.into(), 43).into(), self.timeout) {
			error!(desc = e.to_string(), addr = self.addr.to_string(), server = server, "could not connect");
			return Err("could not connect".to_string());
		}
		
		let start_time = Instant::now();
		let bytes = match socket.send(format!("{}\r\n", self.addr.to_string()).as_bytes()) {
			Ok(b) => b,
			Err(e) => {
				error!(desc = e.to_string(), "Socket::send");
				return Err("Socket::send".to_string());
			}
		};
		debug!(bytes_sent = bytes);
		
		let mut elapsed = Duration::from_secs(0);
		let mut result: Vec<u8> = Vec::new();
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
			
			elapsed = start_time.elapsed();
			if elapsed >= self.timeout {
				return Err("timeout".to_string());
			}
			
			if bytes != 0 {
				result.extend_from_slice(&buffer[..bytes]);
				continue;
			}
			
			break;
		};
		
		let str = match String::from_utf8(result) {
			Ok(s) => s,
			Err(e) => {
				error!(desc = e.to_string(), "invalid utf8");
				return Err("invalid utf8".to_string());
			}
		};
		
		return Ok(str);
	}
}
