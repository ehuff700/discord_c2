use std::process::Stdio;

use tracing::{debug, error};

use super::{
	api::{exit, kill},
	Unix,
};
use crate::{os::traits::process::ProcessModule, RuscordError};

impl ProcessModule for Unix {
	fn spawn(&self, name: &str, args: Option<String>) -> Result<u32, RuscordError> {
		let mut command = &mut std::process::Command::new(name);
		command
			.stderr(Stdio::piped())
			.stdout(Stdio::piped())
			.stdin(Stdio::piped());

		if let Some(args) = args {
			let split_args = args.split(" ").collect::<Vec<_>>();
			debug!("args: {:?}", split_args);
			command = command.args(split_args);
		}
		let child = command.spawn()?;
		Ok(child.id())
	}

	fn kill_other(&self, pid: u32, exit_code: Option<u32>) -> Result<(), crate::RuscordError> {
		let exit_code = exit_code.unwrap_or(15);
		unsafe {
			let result = kill(pid as i32, exit_code as i32);
			if result != 0 {
				error!("failed to kill process {}", pid);
				return Err(std::io::Error::last_os_error().into());
			}
		}
		Ok(())
	}

	fn kill_self(&self, exit_code: Option<u32>) -> ! {
		let exit_code = exit_code.unwrap_or(15);
		unsafe { exit(exit_code as i32) }
	}

	fn process_info(&self) -> crate::RuscordResult<crate::os::traits::process::CurrentProcessInfo> { todo!() }
}
