pub mod recon {
	pub trait ReconModule {
		/// Returns the currently logged in user
		fn username(&self) -> String;
		/// Returns the currently logged in hostname
		fn hostname(&self) -> String;
		/// Returns the list of running processes on the host;
		fn processes(&self) -> Vec<Process>;

		/// Retrieves OS version information
		fn os_version(&self) -> String;
	}

	pub struct Process {
		pub name: String,
		pub pid:  u32,
		pub ppid: u32,
	}
}
