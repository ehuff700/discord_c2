#[cfg(target_family = "unix")]
mod unix;
#[cfg(target_os = "windows")]
mod windows;
use std::ops::Deref;

#[cfg(target_family = "unix")]
use unix::Unix as OS;
#[cfg(target_os = "windows")]
use windows::Windows as OS;

pub(super) mod traits;

#[derive(Default)]
pub struct OsModule {
	pub inner: OS,
}

impl Deref for OsModule {
	type Target = OS;

	fn deref(&self) -> &Self::Target {
		&self.inner
	}
}
