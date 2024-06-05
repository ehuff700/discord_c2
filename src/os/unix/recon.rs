use super::Unix;
use crate::os::traits::recon;

extern "C" {
	pub fn gethostname(name: *mut libc::c_char, size: libc::size_t) -> libc::c_int;

}
impl recon::ReconModule for Unix {
	fn username() -> String {
		std::env::var("USER").unwrap_or(String::from("Unknown User"))
	}

	fn hostname() -> String {
		const HOST_NAME_MAX: usize = 256;
		let mut hostname_vec = vec![0u8; HOST_NAME_MAX];
		unsafe {
			let hostname_ptr = hostname_vec.as_mut_ptr() as *mut libc::c_char;
			let result = gethostname(hostname_ptr, HOST_NAME_MAX);
			if result == 0 {
				std::ffi::CStr::from_ptr(hostname_ptr).to_str().unwrap().to_string()
			} else {
				String::from("Unknown Hostname")
			}
		}
	}

	fn services<'a>() -> Vec<crate::os::traits::recon::Service<'a>> {
		todo!()
	}
}
