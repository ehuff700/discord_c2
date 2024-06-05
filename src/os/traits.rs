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
	use crate::RuscordError;

	pub trait ProcessModule {
		/// Spawns a new process on the host with the given name and arguments.
		fn spawn(&self, name: &str, args: &[&str]) -> Result<(), RuscordError>;

		/// Kills the process with the given pid.
		///
		/// If no pid is provided, it will kill the current process.
		fn kill(&self, pid: Option<u32>);
	}
}
