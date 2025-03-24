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
    (finished_in = $elapsed:expr, $($arg:tt)*) => {
        {
            use indicatif::{ProgressBar, ProgressStyle};

            ProgressBar::new_spinner()
                .with_style(
                    ProgressStyle::with_template("{spinner} {msg} — {elapsed}")
                        .unwrap()
                        .tick_chars("⠋⠙⠹⠸⠼⠴⠦⠧⠇⠏✔"))
                .with_message(format!($($arg)*))
                .with_elapsed($elapsed)
                .finish();
        }
    };
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
    };
}

macro_rules! output_has_filename_or {
    ($output:expr, $default_output_filepath:expr) => {
        if $output.is_dir()
            || $output
                .to_string_lossy()
                .ends_with(std::path::MAIN_SEPARATOR_STR)
        {
            $default_output_filepath
        } else {
            $output
        }
    };
}

mod cli;
pub use cli::Cli;

mod app;

mod pkg;

mod utils;
mod workspace_config;
