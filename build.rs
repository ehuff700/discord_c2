use std::io::Write;

fn main() {
	let token = std::env::var("DISCORD_TOKEN").unwrap();
	let guild_id = std::env::var("RUSCORD_GUILD_ID").unwrap();
	let out_dir = std::env::var("OUT_DIR").unwrap();

	let file = std::fs::File::create(format!("{out_dir}/constants.rs")).unwrap();
	writeln!(
		&file,
		"
    use once_cell::sync::Lazy;
    use poise::serenity_prelude::{{GuildId}};
    pub static DISCORD_TOKEN: Lazy<String> = Lazy::new(|| lc!(\"{}\"));
    pub static RUSCORD_GUILD_ID: GuildId = GuildId::new({});
    ",
		token, guild_id
	)
	.unwrap();
}
