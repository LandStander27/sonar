use indicatif::{HumanDuration, MultiProgress, ProgressBar, ProgressDrawTarget, ProgressStyle};

pub mod prelude;

pub struct Spinner {
	thread: Option<std::thread::Thread>,
}

impl Spinner {
	pub fn new() -> Self {
		return Self {
			thread: None,
		};
	}
	
	pub fn start_thread() {
		let spinner_style = ProgressStyle::with_template("{prefix:.bold.dim} {spinner} {wide_msg}")
			.unwrap()
			.tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");
		
		let pb = ProgressBar::new_spinner();
		pb.set_draw_target(ProgressDrawTarget::stdout());
		pb.set_style(spinner_style);
		pb.enabl
	}
	
}
