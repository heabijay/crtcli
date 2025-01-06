macro_rules! spinner_precise {
    ($($arg:tt)*) => {
        {
            use indicatif::{ProgressBar, ProgressStyle};

            let progress = ProgressBar::new_spinner()
                .with_style(
                    ProgressStyle::with_template("{spinner} {msg} — {elapsed_precise}")
                        .unwrap()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✔"))
                .with_message(format!($($arg)*));

            progress.enable_steady_tick(std::time::Duration::from_millis(100));

            progress
        }
    }
}

macro_rules! spinner {
    ($($arg:tt)*) => {
        {
            use indicatif::{ProgressBar, ProgressStyle};

            let progress = ProgressBar::new_spinner()
                .with_style(
                    ProgressStyle::with_template("{spinner} {msg} — {elapsed}")
                        .unwrap()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✔"))
                .with_message(format!($($arg)*));

            progress.enable_steady_tick(std::time::Duration::from_millis(100));

            progress
        }
    }
}

mod cli;
pub use cli::Cli;

mod app;

mod pkg;

mod utils;
