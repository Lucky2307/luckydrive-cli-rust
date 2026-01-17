use std::time::Duration;

#[cfg(windows)]
use winapi::um::consoleapi::{GetConsoleMode, SetConsoleMode};
#[cfg(windows)]
use winapi::um::processenv::GetStdHandle;
#[cfg(windows)]
use winapi::um::winbase::STD_OUTPUT_HANDLE;
#[cfg(windows)]
use winapi::um::wincon::ENABLE_VIRTUAL_TERMINAL_PROCESSING;

use indicatif::{ProgressBar, ProgressStyle};

#[cfg(windows)]
fn set_console() {
    unsafe {
        let handle = GetStdHandle(STD_OUTPUT_HANDLE);
        let mut mode = 0;
        GetConsoleMode(handle, &mut mode);
        SetConsoleMode(handle, mode | ENABLE_VIRTUAL_TERMINAL_PROCESSING);
    }
}

pub fn get_spinner(message: String) -> impl FnOnce() {
    #[cfg(windows)]
    set_console();
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
