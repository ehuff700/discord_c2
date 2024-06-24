#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
mod types {
	pub type pid_t = i32;
	pub type c_int = i32;
	pub type c_char = i8;
	pub type size_t = usize;
}

#[allow(non_snake_case, unused, clippy::upper_case_acronyms)]
mod constants {
	use super::types::*;
	pub const SIGTERM: c_int = 15;
}

#[allow(non_camel_case_types, clippy::upper_case_acronyms)]
mod structs {
	use super::{constants::*, types::*};

	#[repr(C)]
	pub struct utsname {
		pub sysname: [c_char; 256],
		pub nodename: [c_char; 256],
		pub release: [c_char; 256],
		pub version: [c_char; 256],
		pub machine: [c_char; 256],
	}
}

mod prototypes {
	use super::{structs::*, types::*};
	extern "C" {
		pub fn kill(pid: pid_t, sig: c_int) -> c_int;
		pub fn exit(status: c_int) -> !;
		pub fn gethostname(name: *mut c_char, len: size_t) -> c_int;
		pub fn uname(utsname: *mut utsname) -> c_int;
		#[cfg(target_os = "macos")]
		pub fn proc_pidinfo(
			pid: pid_t, flavor: u32, arg: u32, buffer: *mut std::ffi::c_void, buffersize: size_t,
		) -> c_int;
		#[cfg(target_os = "macos")]
		pub fn proc_listallpids(buffer: *mut std::ffi::c_void, buffersize: size_t) -> c_int;
	}
}

pub use constants::*;
pub use prototypes::*;
pub use structs::*;
pub use types::*;
