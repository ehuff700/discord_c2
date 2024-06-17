use std::io::{Read, Write};

#[derive(serde::Deserialize)]
pub struct Config {
	discord: Discord,
}

#[derive(serde::Deserialize)]
pub struct Discord {
	discord_token: String,
	ruscord_guild_id: String,
}

fn main() {
	let mut config_file_string = String::new();
	let mut config_file = std::fs::File::open("ruscord.toml").expect("couldn't find configuration file!");
	config_file.read_to_string(&mut config_file_string).unwrap();

	let config: Config = toml::from_str(&config_file_string).unwrap();

	let token = config.discord.discord_token;
	let guild_id = config.discord.ruscord_guild_id;
	let out_dir = std::env::var("OUT_DIR").unwrap();

	let file = std::fs::File::create(format!("{out_dir}/constants.rs")).unwrap();
	writeln!(
		&file,
		"
    use once_cell::sync::Lazy;
    use poise::serenity_prelude::{{GuildId}};
    pub static DISCORD_TOKEN: Lazy<String> = Lazy::new(|| lc!(\"{}\"));
    pub static RUSCORD_GUILD_ID: GuildId = GuildId::new({});
	pub const MAX_DISCORD_CHARS: usize = 1999;
    ",
		token, guild_id
	)
	.unwrap();
}
