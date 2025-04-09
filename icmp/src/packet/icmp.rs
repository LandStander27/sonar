use logger::prelude::*;
use super::sum_big_endian_words;

use std::io::Write;
use byteorder::{BigEndian, ByteOrder};

pub struct ICMPPacket {
	pub typ: u8,
	pub code: u8,
	pub checksum: u16,
	pub ident: u16,
	pub sequence: u16,
	pub payload: Vec<u8>,
}

impl ICMPPacket {
	pub fn calculate_checksum(&self) -> u16 {
		// First sum the pseudo header
		let mut sum = 0u32;
		
		// Then sum the len of the message bytes and then the message bytes starting
		// with the message type field and with the checksum field set to 0.
		let bytes = self.get_bytes(false);
		sum += sum_big_endian_words(&bytes);
		
		// handle the carry
		while sum >> 16 != 0 {
			sum = (sum >> 16) + (sum & 0xFFFF);
		}
		return !sum as u16;
	}
	
	pub fn get_bytes(&self, with_checksum: bool) -> Vec<u8> {
		let mut bytes = Vec::new();
		bytes.push(self.typ);
		bytes.push(self.code);
		let mut buf = vec![0; 2];
		BigEndian::write_u16(&mut buf, if with_checksum { self.checksum } else { 0 });
		bytes.append(&mut buf);
		
		bytes.append(&mut {
			let mut bytes = Vec::with_capacity(20);
			let mut buf = vec![0; 2];
			BigEndian::write_u16(&mut buf, self.ident);
			bytes.append(&mut buf);
			buf.resize(2, 0);
			BigEndian::write_u16(&mut buf, self.sequence);
			bytes.append(&mut buf);
			bytes.extend_from_slice(&self.payload);
			bytes
		});
		
		// bytes.append(&mut self.message.get_bytes());
		return bytes;
	}

	pub fn encode(&self, buffer: &mut [u8]) -> Result<(), String> {
		buffer[0] = self.typ;
		buffer[1] = self.code;
		
		buffer[2] = (self.checksum >> 8) as u8;
		buffer[3] = (self.checksum & 0xff) as u8;
		
		buffer[4] = (self.ident >> 8) as u8;
		buffer[5] = self.ident as u8;
		buffer[6] = (self.sequence >> 8) as u8;
		buffer[7] = self.sequence as u8;
		
		if let Err(e) = (&mut buffer[8..]).write_all(self.payload.as_slice()) {
			return Err(e.to_string());
		}
		
		return Ok(());
	}
	
	pub fn decode(buffer: &[u8]) -> Result<Self, String> {
		if buffer.as_ref().len() < 8 {
			return Err("invalid size".to_string());
		}
		
		trace!(type = buffer[0], code = buffer[1]);		
		if buffer[0] != 0 || buffer[1] != 0 {
			return Err("invalid packet".to_string());
		}
		
		let ident = (u16::from(buffer[4]) << 8) + u16::from(buffer[5]);
		let sequence = (u16::from(buffer[6]) << 8) + u16::from(buffer[7]);
		
		return Ok(ICMPPacket {
			typ: buffer[0],
			code: buffer[1],
			checksum: 0,
			ident,
			sequence,
			payload: Vec::from(&buffer[8..]),
		});
	}
}