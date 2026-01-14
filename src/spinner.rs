use std::time::Duration;

use indicatif::{ProgressBar, ProgressStyle};

pub fn get_spinner(message: String) -> impl FnOnce() {
    let spinner = ProgressBar::new_spinner();
    spinner.set_style(
        ProgressStyle::default_spinner()
            .template("{spinner} {msg}")
            .unwrap(),
    );

    spinner.enable_steady_tick(Duration::from_millis(80));
    spinner.set_message(message.clone());

    move || {
        spinner.finish_with_message(format!("✓ {}", &message));
    }
}
