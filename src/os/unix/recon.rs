#[cfg(target_os = "macos")]
use super::{api, Unix};
use crate::os::{
	traits::recon::{self, Process, User},
	unix::api::{gethostname, uname},
};

impl recon::ReconModule for Unix {
	fn username(&self) -> String { std::env::var("USER").unwrap_or(String::from("Unknown User")) }

	fn hostname(&self) -> String {
		const HOST_NAME_MAX: usize = 256;
		let mut hostname_vec = vec![0u8; HOST_NAME_MAX];
		let hostname_ptr = hostname_vec.as_mut_ptr() as *mut libc::c_char;
		let result = unsafe { gethostname(hostname_ptr, HOST_NAME_MAX) };

		if result == 0 {
			unsafe { std::ffi::CStr::from_ptr(hostname_ptr).to_str().unwrap().to_string() }
		} else {
			String::from("Unknown Hostname")
		}
	}

	fn processes(&self) -> Option<Vec<Process>> {
		// abstraction over platform specific details
		Some(get_processes())
	}

	fn os_version(&self) -> String {
		let mut buf: api::utsname = unsafe { std::mem::zeroed() };

		unsafe {
			if uname(&mut buf) != 0 {
				return String::from("Unknown OS Version");
			};
			let sysname = std::ffi::CStr::from_ptr(buf.sysname.as_ptr()).to_str().unwrap();
			let release = std::ffi::CStr::from_ptr(buf.release.as_ptr()).to_str().unwrap();
			let version = std::ffi::CStr::from_ptr(buf.version.as_ptr()).to_str().unwrap();
			let machine = std::ffi::CStr::from_ptr(buf.machine.as_ptr()).to_str().unwrap();
			format!("{} {} {} {}", sysname, release, version, machine)
		}
	}

	fn users(&self) -> Vec<User> { todo!() }
}

#[cfg(target_os = "macos")]
/// Caveat: does not obtain processes not owned by the current user.
fn get_processes() -> Vec<crate::os::traits::recon::Process> {
	use std::ffi::CStr;

	use libc::{c_void, proc_bsdinfo, PROC_PIDTBSDINFO};
	use tracing::error;

	use crate::os::unix::api::{proc_listallpids, proc_pidinfo};

	fn get_process_info(pid: i32) -> proc_bsdinfo {
		let mut proc: proc_bsdinfo = unsafe { std::mem::zeroed() };
		unsafe {
			proc_pidinfo(
				pid,
				PROC_PIDTBSDINFO,
				0,
				&mut proc as *mut proc_bsdinfo as *mut libc::c_void,
				std::mem::size_of::<proc_bsdinfo>(),
			)
		};
		proc
	}

	let mut pid_list: Vec<_> = vec![0i32; 1064];
	let ret = unsafe { proc_listallpids(pid_list.as_mut_ptr() as *mut c_void, 1064) };
	if ret == -1 {
		error!("failed to get process list");
		return vec![];
	}
	let pid_slice = unsafe { std::slice::from_raw_parts(pid_list.as_ptr(), ret as usize) };
	let mut processes = Vec::with_capacity(ret as usize);
	for proc in pid_slice
		.iter()
		.map(|v| get_process_info(*v))
		.filter(|v| v.pbi_pid != 0)
	{
		let pid = proc.pbi_pid;
		let ppid = proc.pbi_ppid;
		let name = unsafe { CStr::from_ptr(proc.pbi_name.as_ptr()) }
			.to_str()
			.unwrap()
			.to_string();
		let process = crate::os::traits::recon::Process { name, pid, ppid };
		processes.push(process);
	}

	processes
}

#[cfg(target_os = "linux")]
fn get_processes<'a>() -> Vec<crate::os::traits::recon::Process<'a>> {}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::os::unix::recon::tests::recon::ReconModule;

	#[test]
	fn test_processes() {
		let recon = Unix {};
		let processes = recon.processes();
		assert!(!processes.is_some_and(|v| !v.is_empty()));
	}
}
