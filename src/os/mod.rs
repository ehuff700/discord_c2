#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_family = "unix")]
pub use unix::Unix as OS;
#[cfg(target_os = "windows")]
pub use windows::Windows as OS;

pub(super) mod traits;
