use byteorder::{BigEndian, ByteOrder};

pub mod icmp;
pub mod ipv4;

fn sum_big_endian_words(bs: &[u8]) -> u32 {
	if bs.is_empty() {
		return 0;
	}
	
	let len = bs.len();
	let mut data = bs;
	let mut sum = 0u32;
	
	// Iterate by word which is two bytes.
	while data.len() >= 2 {
		sum += BigEndian::read_u16(&data[0..2]) as u32;
		// remove the first two bytes now that we've already summed them
		data = &data[2..];
	}
	
	if (len % 2) != 0 {
		// If odd then checksum the last byte
		sum += (data[0] as u32) << 8;
	}
	
	return sum;
}


