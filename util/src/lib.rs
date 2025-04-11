use logger::prelude::*;

use std::net::{
	ToSocketAddrs,
	IpAddr,
};

pub fn dns_lookup<S: Into<String>>(url: S) -> Result<IpAddr, String> {
	let url: String = url.into();
	let mut iter = match url.to_socket_addrs() {
		Ok(i) => i,
		Err(e) => {
			error!(desc = e.to_string(), "could not resolve dns");
			return Err("could not resolve dns".to_string());
		}
	};
	
	let addr = match iter.next() {
		Some(a) => a,
		None => {
			error!("could not resolve dns");
			return Err("could not resolve dns".to_string());
		}
	};
	
	return Ok(addr.ip());
}