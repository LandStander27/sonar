pub mod prelude;

pub enum Level {
	Error,
	Warn,
	Info,
	Debug,
	Trace,
}

impl Level {
	fn to_tracing(&self) -> tracing::Level {
		return match self {
			Self::Trace => tracing::Level::TRACE,
			Self::Debug => tracing::Level::DEBUG,
			Self::Info => tracing::Level::INFO,
			Self::Warn => tracing::Level::WARN,
			Self::Error => tracing::Level::ERROR,
		};
	}
}

impl From<u8> for Level {
	fn from(value: u8) -> Self {
		return match value {
			0 => Self::Error,
			1 => Self::Warn,
			2 => Self::Info,
			3 => Self::Debug,
			_ => Self::Trace,
		};
	}
}

#[deprecated]
pub fn set_log_level(level: Level) {
	let subscriber = tracing_subscriber::fmt().with_max_level(level.to_tracing()).finish();
	tracing::subscriber::set_global_default(subscriber).unwrap();
}

pub fn register(level: Level) {
	let subscriber = tracing_subscriber::fmt().with_max_level(level.to_tracing()).finish();
	tracing::subscriber::set_global_default(subscriber).unwrap();
	std::panic::set_hook(Box::new(|panic| {
		let payload = panic.payload();
		let message: String = if let Some(&s) = payload.downcast_ref::<&str>() {
			s.to_string()
		} else if let Some(s) = payload.downcast_ref::<String>() {
			s.to_owned()
		} else {
			"None".to_string()
		};
		
		if let Some(location) = panic.location() {
			tracing::error!(
				message,
				file = location.file(),
				line = location.line(),
				column = location.column(),
				"panic:",
			);
		} else {
			tracing::error!(message = %panic);
		}
	}));
}