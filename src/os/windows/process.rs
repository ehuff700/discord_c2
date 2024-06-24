use std::{
	ffi::OsString,
	future::Future,
	net::IpAddr,
	os::windows::ffi::{OsStrExt, OsStringExt},
	pin::Pin,
	process::Stdio,
	sync::Arc,
};

use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
	process::Command,
	sync::Mutex,
};
use tokio_util::bytes::BytesMut;
use tracing::{error, info};

use super::{
	api::{
		CloseHandle, CreateProcessW, ExitProcess, GetModuleBaseNameW, OpenProcess, TerminateProcess,
		CREATE_NEW_CONSOLE, MAX_PATH, PROCESS_INFORMATION, PROCESS_TERMINATE, STARTUPINFOW,
	},
	Windows,
};
use crate::{
	os::{
		traits::process::{CurrentProcessInfo, EnvironmentVariable, ProcessModule},
		windows::api::{
			GetCurrentProcess, NtQueryInformationProcess, DWORD, PROCESS_BASIC_INFORMATION, STATUS_SUCCESS,
		},
	},
	RuscordError,
};

impl ProcessModule for Windows {
	fn spawn(&self, name: &str, args: Option<String>) -> Result<u32, RuscordError> {
		let si: STARTUPINFOW = unsafe { std::mem::zeroed() };
		let mut pi: PROCESS_INFORMATION = unsafe { std::mem::zeroed() };
		let final_name = if let Some(args) = args {
			OsString::from(format!("\"{name}\" {args}"))
		} else {
			OsString::from(format!("\"{name}\""))
		};
		let mut name_bytes: Vec<u16> = final_name.encode_wide().collect();
		name_bytes.push(0);

		let result = unsafe {
			CreateProcessW(
				std::ptr::null_mut(),
				name_bytes.as_mut_ptr(),
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				false,
				CREATE_NEW_CONSOLE,
				std::ptr::null_mut(),
				std::ptr::null_mut(),
				&si,
				&mut pi,
			)
		};

		unsafe {
			CloseHandle(pi.hProcess);
			CloseHandle(pi.hThread);
		}

		if !result {
			error!("couldn't spawn process {}", name);
			return Err(std::io::Error::last_os_error().into());
		}

		let pid = pi.dwProcessId;
		info!("spawned process {name} with pid {}", pid);
		Ok(pid)
	}

	fn kill_other(&self, pid: u32, exit_code: Option<u32>) -> Result<(), RuscordError> {
		let exit_code = exit_code.unwrap_or(0);
		unsafe {
			let handle = OpenProcess(PROCESS_TERMINATE, false, pid);
			if handle.is_null() {
				error!("failed to open handle to process {}", pid);
				return Err(std::io::Error::last_os_error().into());
			}

			let result = TerminateProcess(handle, exit_code);
			if !result {
				error!("could not terminate process {}", pid);
				return Err(std::io::Error::last_os_error().into());
			}
			CloseHandle(handle);
		}

		Ok(())
	}

	fn kill_self(&self, exit_code: Option<u32>) -> ! { unsafe { ExitProcess(exit_code.unwrap_or(0)) } }

	fn process_info(&self) -> crate::RuscordResult<crate::os::traits::process::CurrentProcessInfo> {
		let env_variables = std::env::vars()
			.map(|(key, value)| EnvironmentVariable { key, value })
			.collect::<Vec<EnvironmentVariable>>();
		let pid = std::process::id();
		let process_handle = unsafe { GetCurrentProcess() };
		let ppid = {
			let mut pbi: PROCESS_BASIC_INFORMATION = unsafe { std::mem::zeroed() };
			let mut return_length = 0;
			let status = unsafe {
				NtQueryInformationProcess(
					process_handle,
					0, // ProcessBasicInformation,
					&mut pbi as *mut _ as *mut _,
					std::mem::size_of::<PROCESS_BASIC_INFORMATION>() as u32,
					&mut return_length,
				)
			};

			if status != STATUS_SUCCESS {
				error!("failed to get ppid");
				return Err(std::io::Error::last_os_error().into());
			}
			pbi.inheritedFromUniqueProcessId as DWORD
		};

		let name = {
			let mut buffer = vec![0u16; MAX_PATH];
			let length = unsafe { GetModuleBaseNameW(process_handle, 0, buffer.as_mut_ptr(), buffer.len() as u32) };
			if length == 0 {
				error!("failed to get process name");
				return Err(std::io::Error::last_os_error().into());
			}
			buffer.truncate(length as usize);
			std::ffi::OsString::from_wide(&buffer)
		};

		Ok(CurrentProcessInfo {
			name,
			pid,
			ppid,
			env_variables,
		})
	}
}
