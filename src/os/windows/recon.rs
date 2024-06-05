use std::ffi::CStr;

use tracing::error;

use super::{
	api::{
		CreateToolhelp32Snapshot, Process32First, Process32Next, INVALID_HANDLE_VALUE, PROCESSENTRY32,
		TH32CS_SNAPPROCESS,
	},
	Windows,
};
use crate::os::{
	traits::recon::{self, Process, User},
	windows::api::{RtlGetVersion, OSVERSIONINFOEXW},
};

impl recon::ReconModule for Windows {
	fn username(&self) -> String { std::env::var("USERNAME").unwrap_or(String::from("Unknown User")) }

	fn hostname(&self) -> String { std::env::var("USERDOMAIN").unwrap_or(String::from("Unknown Hostname")) }

	fn processes(&self) -> Option<Vec<Process>> {
		let h_process_snap = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
		if h_process_snap == INVALID_HANDLE_VALUE {
			error!("failed to create process snapshot");
			return None;
		}

		let mut process_entry: PROCESSENTRY32 = unsafe { std::mem::zeroed() };
		process_entry.dwSize = std::mem::size_of::<PROCESSENTRY32>() as u32;
		// TODO: error check the APIs
		unsafe { Process32First(h_process_snap, &mut process_entry) };

		let mut processes = Vec::with_capacity(20);
		while unsafe { Process32Next(h_process_snap, &mut process_entry) } {
			let pid = process_entry.th32ProcessID;
			let ppid = process_entry.th32ParentProcessID;

			let name = CStr::from_bytes_until_nul(&process_entry.szExeFile)
				.unwrap_or_default()
				.to_string_lossy()
				.to_string();
			let process = Process { name, pid, ppid };
			processes.push(process);
		}
		Some(processes)
	}

	fn os_version(&self) -> String {
		let mut os_version_exw: OSVERSIONINFOEXW = unsafe { std::mem::zeroed() };
		os_version_exw.dwOSVersionInfoSize = std::mem::size_of::<OSVERSIONINFOEXW>() as u32;
		unsafe {
			RtlGetVersion(&mut os_version_exw);
		}

		// TODO: add windows version (e.g 10 / 8) and workstation
		format!(
			"Windows {}.{}.{} Build {}",
			os_version_exw.dwMajorVersion,
			os_version_exw.dwMinorVersion,
			os_version_exw.dwBuildNumber,
			os_version_exw.dwBuildNumber
		)
	}

	fn users(&self) -> Vec<User> { todo!() }
}

#[cfg(test)]
mod tests {
	use recon::ReconModule;

	use super::*;

	#[test]
	fn test_user_and_host() {
		let recon = Windows {};
		let username = recon.username();
		let hostname = recon.hostname();
		assert_ne!(username, String::from("Unknown User"));
		assert_ne!(hostname, String::from("Unknown Hostname"));
	}

	#[test]
	fn test_os_version() {
		let recon = Windows {};
		let os_version = recon.os_version();
		println!("OS Version: {}", os_version);
		assert!(!os_version.is_empty());
	}

	#[test]
	fn test_processes() {
		let recon = Windows {};
		let processes = recon.processes();
		println!("{:?}", processes);
		assert!(processes.is_some_and(|v| !v.is_empty()));
	}
}
