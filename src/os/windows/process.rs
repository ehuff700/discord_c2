use std::{os::windows::process::CommandExt, process::Stdio};

use tracing::{debug, error};

use super::{
	api::{CloseHandle, ExitProcess, OpenProcess, TerminateProcess, CREATE_NEW_CONSOLE, PROCESS_TERMINATE},
	Windows,
};
use crate::{os::traits::process::ProcessModule, RuscordError};

impl ProcessModule for Windows {
	fn spawn(&self, name: &str, args: Option<String>) -> Result<(), RuscordError> {
		let mut command = &mut std::process::Command::new(name);
		command
			.creation_flags(CREATE_NEW_CONSOLE)
			.stderr(Stdio::piped())
			.stdout(Stdio::piped())
			.stdin(Stdio::piped());

		if let Some(args) = args {
			let split_args = args.split(" ").collect::<Vec<_>>();
			debug!("args: {:?}", split_args);
			command = command.args(split_args);
		}

		command.spawn().unwrap();
		Ok(())
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
}
