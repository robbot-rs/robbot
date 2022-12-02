//! Sync Guild Wars 2 guild members to users.
//!
//! The `guildsync` plugin allows to create Guild Wars 2 rank to role mappings, and synchronises
//! the mapped roles as member roles change and members leave the guild.
//!
//! # Features
//!
//! - Link a guild rank to a role
//! - Automatically synchronise guild ranks as they are changed ingame
//! - Reguarly checks and removes inappropriate roles from *all* server members
//! - Support for multiple guilds within a single server, even when sharing roles
//!
//! # Quirks
//!
//! There are currently a number of quirks present that should be noted. When verifying a user
//! is verified all links available on the requested server. The user will however be linked to
//! the linked guilds that they were a member of at the time of verification. If a user later
//! enters another linked guild, they are not added automatically, they need to verify a second
//! time. This also applies to adding another link later.
//!
//! Additionally the current implementation does not keep track of users' api tokens. If provided
//! they are only used once to fetch the user's account name. This is rarely a problem, since
//! account names change rarely, but if it happens the user will be removed. Checking all tokens
//! would solve this problem, but result in an even more complex synchronisation process and
//! massively increased traffic.
//!
//! This also brings an benefit however. If a user verifies themself with an api token, they can
//! delete token after the verification process.
//!
//! # Implementation notes
//!
//! The current implementation synchronises every server atomically, synchronising an server
//! partially (i.e. a single guild within a multi-guild server) is not possible.
//!
//! All linked users will have a reference record, that links their account name to a unique user.
//! On every update cycle that list of records is compared against the list of members that are
//! currently in the guild and record that point to an account name that is no longer in the guild
//! are removed. This step is applied for every linked guild in the server.
//!
//! At this point we have an updated list of what users have what rank in the guild. The next step
//! builds a set of predicates over every role that is being managed. These predicates describe
//! every linked in the server and cannot be constructed correctly for a single guild. A predicate
//! describes a set of requirements (i.e. have a specific rank in a specific guild), at least one
//! of which must be true in order for a user to have the role.
//!
//! The final step can now iterate over all members in the server, validate every managed role
//! using the predicates and add invalid/remove missing roles.
//!
mod commands;
mod predicates;
mod task;
mod utils;

use ::gw2api::Client;
use robbot::arguments::ArgumentsExt;
use robbot::model::id::{GuildId, RoleId, UserId};
use robbot::store::{create, get};
use robbot::{Error, Result, StoreData};
use robbot_core::command::Command;
use robbot_core::context::{Context, MessageContext};
use robbot_core::state::State;
use snowflaked::sync::Generator;

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

    state.commands().load_command(link(), Some("guildsync"))?;
    state.commands().load_command(ranks(), Some("guildsync"))?;

    state
        .commands()
        .load_command(commands::config::create(), Some("guildsync link"))?;
    state
        .commands()
        .load_command(commands::config::delete(), Some("guildsync link"))?;
    state
        .commands()
        .load_command(commands::config::list(), Some("guildsync link"))?;
    state
        .commands()
        .load_command(commands::config::details(), Some("guildsync link"))?;

    state
        .commands()
        .load_command(commands::ranks::list(), Some("guildsync ranks"))?;
    state
        .commands()
        .load_command(commands::ranks::set(), Some("guildsync ranks"))?;
    state
        .commands()
        .load_command(commands::ranks::unset(), Some("guildsync ranks"))?;

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

    pub async fn extract_exact(
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

            if guild.name == argument {
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
    cmd.set_description("Manage guild rank to role mappings.");
    cmd
}

pub fn link() -> Command {
    let mut cmd = Command::new("link");
    cmd.set_description("Manage linked guilds for this server.");
    cmd
}
