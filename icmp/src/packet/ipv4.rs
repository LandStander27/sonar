use logger::prelude::*;

#[derive(PartialEq)]
pub enum IPV4Protocol {
	Icmp,
}

impl IPV4Protocol {
	fn decode(data: u8) -> Option<Self> {
		return match data {
			1 => Some(IPV4Protocol::Icmp),
			_ => None,
		};
	}
}

#[allow(dead_code)]
pub struct IPV4Packet {
	pub version: u8,
	pub ihl: u8,
	
	pub tos: u8,
	pub tot_len: u16,
	pub id: u16,
	pub frag_off: u16,
	pub ttl: u8,
	pub protocol: IPV4Protocol,
	pub check: u16,
	
	pub from_addr: u32,
	pub dest_addr: u32,
	
	pub data: Vec<u8>,
}

impl IPV4Packet {
	pub fn decode(buffer: &[u8]) -> Result<Self, String> {
		if buffer.len() < 20 {
			return Err("invalid ipv4 header".to_string());
		}
		
		let byte0 = buffer[0];
		let version = (byte0 & 0xf0) >> 4;
		let header_size = 4 * ((byte0 & 0x0f) as usize);
		trace!(header_size);
		
		if version != 4 {
			return Err("invalid version".to_string());
		}
		
		if buffer.len() < header_size {
			return Err("invalid header size".to_string());
		}
		
		let protocol = match IPV4Protocol::decode(buffer[9]) {
			Some(p) => p,
			None => {
				return Err("invalid ipv4 protocol".to_string());
			}
		};
		
		return Ok(Self {
			version,
			ihl: byte0 & 0x0f,
			tos: buffer[1],
			tot_len: ((buffer[2] as u16) << 8) | (buffer[3] as u16),
			id: ((buffer[4] as u16) << 8) | (buffer[5] as u16),
			frag_off: ((buffer[6] as u16) << 8) | (buffer[7] as u16),
			ttl: buffer[8],
			protocol,
			check: ((buffer[10] as u16) << 8) | (buffer[11] as u16),
			
			from_addr: ((buffer[12] as u32) << 24) | ((buffer[13] as u32) << 16) | ((buffer[14] as u32) << 8) | (buffer[15] as u32),
			dest_addr: ((buffer[16] as u32) << 24) | ((buffer[17] as u32) << 16) | ((buffer[18] as u32) << 8) | (buffer[19] as u32),
			
			data: Vec::from(&buffer[header_size..])
		});
	}
}