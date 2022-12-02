//! Sync Guild Wars 2 guild members to users.
//!
//! The `guildsync` plugin allows to create Guild Wars 2 rank to role mappings, and synchronises
//! the mapped roles as member roles change and members leave the guild.
//!
//! # Features
//!
//! - Add a general role for being a member of a guild
//! - Link a guild rank to a role
//! - Automatically synchronise guild ranks as they are changed ingame
//! - Reguarly checks and removes inappropriate roles from *all* server members
//! - Support for multiple guilds within a single server, even when sharing roles
//!
mod commands;
mod gw2api;
mod predicates;
mod task;
mod utils;

use ::gw2api::Client;
use robbot::arguments::{ArgumentsExt, RoleMention};
use robbot::builder::CreateMessage;
use robbot::command;
use robbot::model::id::{GuildId, RoleId, UserId};
use robbot::store::{create, delete, get};
use robbot::{Error, Result, StoreData};
use robbot_core::command::Command;
use robbot_core::context::{Context, MessageContext};
use robbot_core::state::State;
use snowflaked::sync::Generator;

use std::fmt::Write;

pub const PERMISSION_MANAGE: &str = "guildsync.manage";
pub const PERMISSION_MANAGE_MEMBERS: &str = "guildsync.manage_members";

pub async fn init(state: &State) -> Result {
    create!(state.store(), GuildLink).await?;
    create!(state.store(), GuildMember).await?;
    create!(state.store(), GuildRank).await?;

    state
        .commands()
        .load_command(commands::verify_api(), None)?;

    state.commands().load_command(guildsync(), None)?;

    state.commands().load_command(setup(), Some("guildsync"))?;

    state
        .commands()
        .load_command(commands::setup_list(), Some("guildsync setup"))?;

    state
        .commands()
        .load_command(commands::setup_create(), Some("guildsync setup"))?;

    state
        .commands()
        .load_command(commands::verify(), Some("guildsync"))?;
    state
        .commands()
        .load_command(commands::unverify(), Some("guildsync"))?;
    state
        .commands()
        .load_command(commands::whois(), Some("guildsync"))?;
    state
        .commands()
        .load_command(commands::list(), Some("guildsync"))?;
    state
        .commands()
        .load_command(commands::sync(), Some("guildsync"))?;

    state.commands().load_command(ranks(), Some("guildsync"))?;
    state
        .commands()
        .load_command(_ranks_list(), Some("guildsync ranks"))?;
    state
        .commands()
        .load_command(ranks_set(), Some("guildsync ranks"))?;

    state.tasks().add_task(task::sync()).await;

    Ok(())
}

static GUILD_LINK_ID_GENERATOR: Generator = Generator::new(0);
static GUILD_MEMBER_ID_GENERATOR: Generator = Generator::new(0);
static GUILD_RANK_ID_GENERATOR: Generator = Generator::new(0);

#[derive(Clone, Debug, StoreData)]
pub(crate) struct GuildLink {
    pub id: u64,
    pub guild_id: GuildId,
    pub gw_guild_id: String,
    pub api_token: String,
}

impl GuildLink {
    /// Extract a requested [`GuildLink`] from a message.
    ///
    /// This function searches for a matching [`GuildLink`] in the following order:
    /// 1. If a server only has a single link, it is always returned.
    /// 1. An exact match of the `id` field.
    /// 2. An exact match of the ingame guild id.
    /// 3. If the argument is the prefix of a guild name.
    pub async fn extract(
        ctx: &mut MessageContext,
    ) -> std::result::Result<Option<Self>, robbot::Error> {
        let guild_id = ctx.event.guild_id.unwrap();

        let links = get!(ctx.state.store(), GuildLink => {
            guild_id == guild_id,
        })
        .await?;

        if links.len() == 1 {
            return Ok(Some(links[0].clone()));
        }

        let argument = ctx.args.pop().ok_or(robbot::Error::InvalidCommandUsage)?;

        // 1. Search for exact `id` field match.
        // Only search for the `id` field if `argument` parses.
        if let Ok(id) = argument.parse::<u64>() {
            for link in &links {
                if link.id == id {
                    return Ok(Some(link.clone()));
                }
            }
        }

        // 2. Search for exact inagme guild id match.
        for link in &links {
            if link.gw_guild_id == argument {
                return Ok(Some(link.clone()));
            }
        }

        for link in links {
            // TODO: Some caching would be beneficial here.
            let client: Client = Client::builder().access_token(&link.api_token).into();

            let guild = ::gw2api::v2::guild::Guild::get(&client, &link.gw_guild_id).await?;

            if guild.name.starts_with(&argument) {
                return Ok(Some(link));
            }
        }

        Ok(None)
    }
}

impl Default for GuildLink {
    fn default() -> Self {
        Self {
            id: 0,
            guild_id: GuildId::default(),
            gw_guild_id: String::new(),
            api_token: String::new(),
        }
    }
}

impl GuildLink {
    pub fn new(guild_id: GuildId, gw_guild_id: String, api_token: String) -> Self {
        let id = GUILD_LINK_ID_GENERATOR.generate();

        Self {
            id,
            guild_id,
            gw_guild_id,
            api_token,
        }
    }

    pub async fn members<T>(&self, ctx: &Context<T>) -> std::result::Result<Vec<GuildMember>, Error>
    where
        T: Send + Sync,
    {
        let members = get!(ctx.state.store(), GuildMember => {
            link_id == self.id,
        })
        .await?;

        Ok(members)
    }

    pub async fn ranks<T>(&self, ctx: &Context<T>) -> std::result::Result<Vec<GuildRank>, Error>
    where
        T: Send + Sync,
    {
        let ranks = get!(ctx.state.store(), GuildRank => {
            link_id == self.id,
        })
        .await?;

        Ok(ranks)
    }
}

/// A member in a guild that has been linked.
///
/// A `GuildMember` always has a [`GuildLink`] association.
#[derive(Clone, Debug, StoreData)]
pub(crate) struct GuildMember {
    /// A globally identifier, unique across all members of all links.
    pub id: u64,
    /// The id of the [`GuildLink`] this `GuildMember` belongs to.
    pub link_id: u64,
    /// The Guild Wars 2 account name of the user.
    pub account_name: String,
    /// The id of the linked Discord user.
    pub user_id: UserId,
}

impl Default for GuildMember {
    fn default() -> Self {
        Self {
            id: 0,
            link_id: 0,
            account_name: String::default(),
            user_id: UserId::default(),
        }
    }
}

impl GuildMember {
    pub fn new(link_id: u64, account_name: String, user_id: UserId) -> Self {
        let id = GUILD_MEMBER_ID_GENERATOR.generate();

        Self {
            id,
            link_id,
            account_name,
            user_id,
        }
    }
}

#[derive(Clone, Debug, StoreData)]
pub(crate) struct GuildRank {
    /// A globally identifier, unique across all ranks of all links.
    pub id: u64,
    /// The id of [`GuildLink`] this `GuildMember` belongs to.
    pub link_id: u64,
    /// The name of the Guild Wars 2 guild rank.
    pub rank_name: String,
    /// The id of the linked Discord role.
    pub role_id: RoleId,
}

impl Default for GuildRank {
    fn default() -> Self {
        Self {
            id: 0,
            link_id: 0,
            rank_name: String::new(),
            role_id: RoleId::default(),
        }
    }
}

impl GuildRank {
    pub fn new(link_id: u64, rank_name: String, role_id: RoleId) -> Self {
        let id = GUILD_RANK_ID_GENERATOR.generate();

        Self {
            id,
            link_id,
            rank_name,
            role_id,
        }
    }
}

pub fn guildsync() -> Command {
    let mut cmd = Command::new("guildsync");
    cmd.set_description("Link discord users to guild members.");
    cmd
}

pub fn ranks() -> Command {
    let mut cmd = Command::new("ranks");
    cmd.set_description("Assign role to rank mappings.");
    cmd
}

pub fn setup() -> Command {
    let mut cmd = Command::new("setup");
    cmd.set_description("Guildsync configuration");
    cmd
}

#[command(name = "set", description = "Map a rank to a role.", permissions = [PERMISSION_MANAGE])]
async fn ranks_set(mut ctx: MessageContext) -> Result {
    if ctx.args.len() < 2 {
        return Err(Error::InvalidCommandUsage);
    }

    let rank_name = ctx.args.pop().unwrap();
    let role: RoleMention = ctx.args.pop_parse()?;

    let role_id = role.id;

    let guild_link = commands::get_guild_link(&ctx).await?;

    let ranks = get!(ctx.state.store(), GuildRank => {
        link_id == guild_link.id,
    })
    .await?;

    for rank in ranks {
        if rank.rank_name == rank_name {
            delete!(ctx.state.store(), GuildRank => {
                id == rank.id,
            })
            .await?;

            break;
        }
    }

    let rank = GuildRank::new(guild_link.id, rank_name, role_id);

    ctx.state.store().insert(rank.clone()).await?;

    let _ = ctx
        .respond(format!(
            ":white_check_mark: Successfully assigned role <@&{}> to `{}`",
            rank.role_id.0, rank.rank_name
        ))
        .await?;

    // let _ = ctx
    //     .event
    //     .reply(
    //         &ctx.raw_ctx,
    //         format!(":x: Cannot find rank `{}`.", rank_name),
    //     )
    //     .await?;
    Ok(())
}

#[command(name = "list", description = "Display the current rank to role assignments.", permissions = [PERMISSION_MANAGE])]
async fn _ranks_list(ctx: MessageContext) -> Result {
    let guild_link = commands::get_guild_link(&ctx).await?;

    let ranks = get!(ctx.state.store(), GuildRank => {
        link_id == guild_link.id,
    })
    .await?;

    let _ = ctx
        .respond(CreateMessage::new(|m| {
            m.embed(|e| {
                e.title("__Rank Mappings__");
                e.description({
                    let mut string = String::new();
                    for rank in ranks {
                        writeln!(string, "<@&{}> => `{}`", rank.role_id, rank.rank_name).unwrap();
                    }
                    string
                });
            });
        }))
        .await;

    Ok(())
}
