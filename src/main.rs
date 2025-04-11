use logger::prelude::*;
use icmp::prelude::*;

use std::sync::mpsc::{
	channel,
	Receiver,
	Sender
};
use clap::Parser;
use tracing_subscriber::prelude::*;
use colored::Colorize;

#[derive(Parser, Debug)]
#[command(name = "sonar", version = version::version)]
#[command(about = "Modern ping", long_about = None)]
struct Args {
	#[arg(short, long, help = "increase verbosity (-v: warnings, -vv: info, -vvv: debug, -vvvv: trace)", action = clap::ArgAction::Count)]
	verbose: u8,
	
	#[arg(required = true, help = "ip address to ping")]
	ip: String,
	
	#[arg(short, long, help = "amount to attempt pinging")]
	count: Option<u64>,
	
	#[arg(short, long, help = "seconds to wait between sending packets", default_value_t = 1.0)]
	interval: f32,
	
	#[arg(short = 'x', long, help = "enable querying for extra information (uses IP2Location)", default_value_t = false)]
	extra: bool,
}

mod info_query;

fn main() -> std::process::ExitCode {
	let args = Args::parse();
	
	let layer = tracing_indicatif::IndicatifLayer::new();
	logger::register(Level::from(args.verbose))
		.with_writer(layer.get_stdout_writer())
		.finish()
		.with(layer)
		.init();
	
	let (tx, rx): (Sender<()>, Receiver<()>) = channel();
	if let Err(e) = ctrlc::set_handler(move || {
		println!();
		
		if let Err(e) = tx.send(()) {
			error!(desc = e.to_string(), "could not send signal on channel");
		}
	}) {
		error!(desc = e.to_string(), "could not set ctrlc handler");
	}
	
	if args.extra && info_query::whois(&args.ip, &rx).is_err() {
		error!("querying for extra info failed");
	}
	
	trace!("Pinger::new");
	let mut pinger = match Pinger::new(&args.ip) {
		Ok(p) => p,
		Err(e) => {
			error!(desc = e.to_string());
			return 1.into()
		}
	};
	
	trace!("Pinger::init_socket");
	if let Err(e) = pinger.init_socket() {
		error!(desc = e.to_string(), "could not init socket");
		return 1.into();
	}
	
	let ping = |pinger: &mut Pinger, wait_time: f32| -> bool {
		std::thread::sleep(std::time::Duration::from_secs_f32(wait_time));
		
		if rx.try_recv().is_ok() {
			return false;
		}
		
		trace!("Pinger::ping");
		let reply = match pinger.ping(&rx) {
			Ok(r) => r,
			Err(e) => {
				if e == "stop signal" {
					return false;
				}
				// error!(desc = e.to_string(), "ping failed");
				return true;
			}
		};

		println!("{}", format!("[{}] | {} -> {} | {:.2} ms",
			reply.sequence,
			reply.from_addr,
			reply.dest_addr,
			(reply.elapsed.as_secs_f32() * 100000.0).round() / 100.0,
		).truecolor(0, 255, 0));
		return true;
	};

	println!("Pinging {} ({}){}", args.ip, pinger.get_dest(), if let Some(c) = args.count { format!(" {} times", c) } else { "".to_string() });
	if let Some(count) = args.count {
		for _ in 0..count {
			if !ping(&mut pinger, args.interval) { break; }
		}
	} else {
		loop {
			if !ping(&mut pinger, args.interval) { break; }
		}
	}
	
	return 0.into();
}
