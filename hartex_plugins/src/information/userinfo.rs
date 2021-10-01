//! # The `userinfo` Module
//!
//! This module implements the `userinfo` command

use hartex_cmdsys::{
    command::{
        Command,
        CommandType
    },
    context::CommandContext
};

use hartex_conftoml::guildconf::tz::Timezone;

use hartex_core::{
    discord::{
        cache_inmemory::InMemoryCache,
        embed_builder::{
            EmbedAuthorBuilder,
            EmbedBuilder,
            EmbedFieldBuilder,
            ImageSource
        },
        mention::Mention,
        model::{
            application::{
                callback::{
                    CallbackData,
                    InteractionResponse,
                },
                command::{
                    BaseCommandOptionData,
                    CommandOption,
                },
                interaction::{
                    application_command::CommandDataOption,
                    Interaction,
                },
            },
            gateway::presence::{
                ActivityType,
                Status
            },
            id::UserId
        },
        util::snowflake::Snowflake
    },
    error::{
        HarTexError,
        HarTexResult
    },
    time::{
        offset::FixedOffset,
        DateTime,
        TimeZone
    }
};

use hartex_dbmani::guildconf::GetGuildConfig;

use hartex_utils::{
    cdn::{
        Cdn,
        CdnResourceFormat
    },
    FutureRetType
};

/// # Struct `Userinfo`
///
/// The `userinfo` command.
pub struct Userinfo;

impl Command for Userinfo {
    fn name(&self) -> String {
        String::from("userinfo")
    }

    fn description(&self) -> String {
        String::from("InformationPlugin.UserinfoCommand")
    }

    fn command_type(&self) -> CommandType {
        CommandType::ChatInput
    }

    fn execute<'asynchronous_trait>(&self, ctx: CommandContext, cache: InMemoryCache) -> FutureRetType<'asynchronous_trait, ()> {
        Box::pin(execute_userinfo_command(ctx, cache))
    }

    fn optional_cmdopts(&self) -> Vec<CommandOption> {
        vec![
            CommandOption::Mentionable(BaseCommandOptionData {
                description: String::from("(optional) the user to query the information"),
                name: String::from("user"),
                required: false
            })
        ]
    }
}

/// # Asynchronous Function `execute_userinfo_command`
///
/// Executes the `userinfo` command.
///
/// ## Parameters
/// - `ctx`, type `CommandContext`: the command context to use.
async fn execute_userinfo_command(ctx: CommandContext, cache: InMemoryCache) -> HarTexResult<()> {
    let interaction = match ctx.interaction.clone() {
        Interaction::ApplicationCommand(command) => command,
        _ => return Err(
            HarTexError::Custom {
                message: String::from("invalid interaction type: expected ApplicationCommand")
            }
        )
    };

    if interaction.guild_id.is_none() || interaction.user.is_some() {
        ctx.http
            .interaction_callback(
                interaction.id,
                &interaction.token,
                &InteractionResponse::ChannelMessageWithSource(
                    CallbackData {
                        allowed_mentions: None,
                        components: None,
                        content: Some(String::from(":x: This command can only be used in a guild.")),
                        embeds: vec![],
                        flags: None,
                        tts: None
                    }
                )
            )
            .exec()
            .await?;
    }

    let config = GetGuildConfig::new(interaction.guild_id.unwrap())
        .await?;

    let options = interaction.data.options;
    let user = if options.is_empty() {
        // unwrapping here is fine as it is now ensured that the interaction is sent from a guild,
        // not in a user DM (which is the case when interaction.member is None)
        interaction.member.unwrap().user.unwrap()
    }
    else {
        // unwrapping here is fine because the command only accepts a "user" parameter and is
        // asserted to be of type "String"; therefore, the parameter must exist if the options vec
        // is not empty and must be with the name "user" and of type "String"
        let user_option = options
            .into_iter()
            .find(|option| option.name() == "user" && option.kind() == "String")
            .unwrap();
        let value = if let CommandDataOption::String { value, ..} = user_option {
            value
        }
        else {
            unreachable!("unexpected parameter type")
        };

        ctx.http
            .user(UserId::from(value.parse::<u64>().unwrap()))
            .exec()
            .await?
            .model()
            .await?
    };
    let member = ctx.http
        // it is ok to unwrap here because it is already checked that the interaction is sent from
        // a guild (which its id should never be None)
        .guild_member(interaction.guild_id.unwrap(), user.id)
        .exec()
        .await?
        .model()
        .await?;
    let presence = cache.presence(interaction.guild_id.unwrap(), member.user.id);
    let mut roles = member
        .roles
        .iter()
        .filter_map(|role_id| cache.role(*role_id))
        .collect::<Vec<_>>();
    roles.sort_by(|prev_role, curr_role| {
        curr_role.position.cmp(&prev_role.position)
    });

    let avatar_url = if let Some(hash) = user.avatar {
        let format = if hash.starts_with("a_") {
            CdnResourceFormat::GIF
        }
        else {
            CdnResourceFormat::PNG
        };

        Cdn::user_avatar(user.id, hash, format)
    }
    else {
        Cdn::default_user_avatar(user.discriminator.clone().parse().unwrap())
    };

    let mut embed = EmbedBuilder::new()
        .author(EmbedAuthorBuilder::new()
            .name(format!("Information about {username}", username = &user.name))
            .icon_url(ImageSource::url(avatar_url)?)
        )
        .color(0x03BEFC)
        .field(EmbedFieldBuilder::new("Username", user.name).inline())
        .field(EmbedFieldBuilder::new("Discriminator", user.discriminator).inline())
        .field(EmbedFieldBuilder::new("User ID", format!("{id}", id = user.id)))
        .field(
            EmbedFieldBuilder::new(
                "Guild Nickname",
                member.nick.unwrap_or(String::from("None"))
            ).inline()
        )
        .field(
            EmbedFieldBuilder::new(
                "Highest Role in Guild",
                if roles.is_empty() {
                    String::from("none")
                }
                else {
                    roles.first().unwrap().mention().to_string()
                }
            ).inline()
        );

    if let Some(presence) = presence {
        let activities = presence.activities;

        embed = embed
            .field(EmbedFieldBuilder::new("Status",
            match presence.status {
                Status::DoNotDisturb => "do not disturb",
                Status::Idle => "idle",
                Status::Invisible => "invisible",
                Status::Offline => "offline",
                Status::Online => "online"
            }));

        if activities.is_empty() {
            embed = embed
                .field(EmbedFieldBuilder::new("Activities", "none"));
        }
        else {
            for activity in activities {
                let temp = embed.clone();
                let activity_type = match activity.kind {
                    ActivityType::Competing => "Competing",
                    ActivityType::Custom => "Custom",
                    ActivityType::Listening => "Listening",
                    ActivityType::Playing => "Playing",
                    ActivityType::Streaming => "Streaming",
                    ActivityType::Watching => "Watching"
                };

                embed = temp
                    .field(EmbedFieldBuilder::new(
                        format!("Activity - {activity_type}"),
                        if activity.kind == ActivityType::Custom {
                            activity.state.unwrap()
                        }
                        else {
                            activity.name
                        }
                    ));
            }
        }
    }
    else {
        embed = embed
            .field(EmbedFieldBuilder::new("Status", "unknown"))
            .field(EmbedFieldBuilder::new("Activities", "none"));
    }

    let timezone = if config.NightlyFeatures.localization {
        config.GuildConfiguration.timezone
    }
    else {
        Timezone::UTC
    };
    let joined_at = DateTime::parse_from_str(member.joined_at.unwrap().as_str(), "%Y-%m-%dT%H:%M:%S%.f%:z")?;
    let created_at = FixedOffset::east(timezone.into_offset_secs()).timestamp_millis(user.id.timestamp());

    let temp = embed.clone();

    embed = temp
        .field(EmbedFieldBuilder::new("Joined Guild At", format!("{joined_at} ({timezone})")).inline())
        .field(EmbedFieldBuilder::new("Account Created At", format!("{created_at} ({timezone})")).inline());

    ctx.http
        .interaction_callback(
            interaction.id,
            &interaction.token,
            &InteractionResponse::ChannelMessageWithSource(
                CallbackData {
                    allowed_mentions: None,
                    components: None,
                    content: None,
                    embeds: vec![embed.build()?],
                    flags: None,
                    tts: None
                }
            )
        )
        .exec()
        .await?;

    Ok(())
}