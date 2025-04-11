use logger::prelude::*;
use whois::prelude::*;

use colored::Colorize;
use indicatif::{ProgressBar, ProgressDrawTarget, ProgressStyle};
use serde_json::Value;
use std::time::Duration;
use std::sync::mpsc::Receiver;

pub fn whois<S: Into<String> + Clone, T>(addr: S, rx: &Receiver<T>) -> Result<(), ()> {
	let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner:.cyan} {wide_msg}")
		.unwrap()
		.tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
	
	let pb = ProgressBar::new_spinner();
	pb.set_draw_target(ProgressDrawTarget::stdout());
	pb.set_style(spinner_style);
	pb.set_message("Getting whois");
	pb.enable_steady_tick(Duration::from_millis(150));
	
	trace!("WhoIs::new");
	let mut whois = match WhoIs::new(addr.clone()) {
		Ok(p) => p,
		Err(e) => {
			pb.finish_with_message("Whois failed".truecolor(255, 0, 0).to_string());
			warn!(desc = e.to_string());
			return Err(());
		}
	};
	
	trace!("WhoIs::get_whois");
	let whois_response = match whois.get_whois(rx) {
		Ok(r) => r,
		Err(e) => {
			pb.finish_with_message("Whois failed".truecolor(255, 0, 0).to_string());
			warn!(desc = e.to_string());
			return Ok(());
		}
	};
	
	pb.set_message("Getting geolocation");
	let client = match reqwest::blocking::Client::builder().timeout(Duration::from_millis(1500)).build() {
		Ok(c) => c,
		Err(e) => {
			pb.finish_with_message("Whois failed".truecolor(255, 0, 0).to_string());
			warn!(desc = e.to_string(), "could not build client");
			return Err(());
		}
	};
	
	let ip = match util::dns_lookup(addr.into() + ":0") {
		Ok(i) => i,
		Err(e) => {
			warn!(desc = e.to_string(), "dns lookup failed");
			return Err(());
		}
	};

	let response = match client.get(format!("https://api.ip2location.io/?ip={}", ip)).send() {
		Ok(r) => r,
		Err(e) => {
			warn!(desc = e.to_string());
			return Err(());
		}
	};
	
	let text = match response.text() {
		Ok(j) => j,
		Err(e) => {
			warn!(desc = e.to_string(), "failed to get response text");
			return Err(());
		}
	};

	let json: Value = match serde_json::from_str(text.as_str()) {
		Ok(j) => j,
		Err(e) => {
			warn!(desc = e.to_string(), "failed to parse json");
			return Err(());
		}
	};
	
	pb.finish_and_clear();
	println!("NetName: {}\nOrganization: {}\nLocation: {}, {}, {}",
		whois_response.netname.unwrap_or("Unknown".to_string()),
		json["as"].as_str().unwrap_or("Unknown"),
		json["country_name"].as_str().unwrap_or("Unknown"),
		json["region_name"].as_str().unwrap_or("Unknown"),
		json["city_name"].as_str().unwrap_or("Unknown")
	);
	
	return Ok(());
}