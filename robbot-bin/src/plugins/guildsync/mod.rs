mod commands;
mod gw2api;
mod task;
mod utils;

use crate::{command, task};

use robbot::arguments::ArgumentsExt;
use robbot::store::id::{Snowflake, SnowflakeGenerator};
use robbot::Context as _;
use robbot::{Error, Result, StoreData};
use robbot_core::context::{Context, MessageContext};
use robbot_core::state::State;
use robbot_core::task::TaskSchedule;

use serenity::model::id::{GuildId, RoleId, UserId};

use std::fmt::Write;
use std::sync::Mutex;

pub const PERMISSION_MANAGE: &str = "guildsync.manage";
pub const PERMISSION_MANAGE_MEMBERS: &str = "guildsync.manage_members";

pub async fn init(state: &State) -> Result {
    state.store().create::<GuildLink>().await.unwrap();
    state.store().create::<GuildMember>().await.unwrap();
    state.store().create::<GuildRank>().await.unwrap();

    unsafe {
        let guildlink = Box::new(Mutex::new(SnowflakeGenerator::new_unchecked(0)));
        GUILD_LINK_ID_GENERATOR = Box::leak(guildlink) as *const _;

        let guildmember = Box::new(Mutex::new(SnowflakeGenerator::new_unchecked(0)));
        GUILD_MEMBER_ID_GENERATOR = Box::leak(guildmember) as *const _;

        let guildrank = Box::new(Mutex::new(SnowflakeGenerator::new_unchecked(0)));
        GUILD_RANK_ID_GENERATOR = Box::leak(guildrank) as *const _;
    }

    state.commands().load_command(guildsync(), None)?;
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
        .load_command(show(), Some("guildsync ranks"))?;
    state
        .commands()
        .load_command(set(), Some("guildsync ranks"))?;

    state.tasks().add_task(tsync()).await;

    Ok(())
}

// Statics for id generators. It's safe to assume valid pointers
// after the init is called.
static mut GUILD_LINK_ID_GENERATOR: *const Mutex<SnowflakeGenerator> = 0 as *const _;
static mut GUILD_MEMBER_ID_GENERATOR: *const Mutex<SnowflakeGenerator> = 0 as *const _;
static mut GUILD_RANK_ID_GENERATOR: *const Mutex<SnowflakeGenerator> = 0 as *const _;

#[derive(Clone, Debug, StoreData)]
pub(crate) struct GuildLink {
    pub id: Snowflake,
    pub guild_id: GuildId,
    pub gw_guild_id: String,
    pub api_token: String,
}

impl Default for GuildLink {
    fn default() -> Self {
        Self {
            id: Snowflake(0),
            guild_id: GuildId::default(),
            gw_guild_id: String::new(),
            api_token: String::new(),
        }
    }
}

impl GuildLink {
    pub fn new(guild_id: GuildId, gw_guild_id: String, api_token: String) -> Self {
        let id = {
            let gen = unsafe { &*GUILD_LINK_ID_GENERATOR };

            let mut gen = gen.lock().unwrap();

            gen.yield_id()
        };

        Self {
            id,
            guild_id,
            gw_guild_id,
            api_token,
        }
    }

    pub async fn members<T>(
        &self,
        ctx: &Context<T>,
    ) -> std::result::Result<Vec<GuildMember>, Error> {
        let members = ctx
            .state
            .store()
            .get(GuildMember::query().link_id(self.id))
            .await?;
        Ok(members)
    }

    pub async fn ranks<T>(&self, ctx: &Context<T>) -> std::result::Result<Vec<GuildRank>, Error> {
        let ranks = ctx
            .state
            .store()
            .get(GuildRank::query().link_id(self.id))
            .await?;
        Ok(ranks)
    }
}

#[derive(Clone, Debug, StoreData)]
pub(crate) struct GuildMember {
    pub id: Snowflake,
    pub link_id: Snowflake,
    pub account_name: String,
    pub user_id: UserId,
}

impl Default for GuildMember {
    fn default() -> Self {
        Self {
            id: Snowflake(0),
            link_id: Snowflake(0),
            account_name: String::default(),
            user_id: UserId::default(),
        }
    }
}

impl GuildMember {
    pub fn new(link_id: Snowflake, account_name: String, user_id: UserId) -> Self {
        let id = {
            let gen = unsafe { &*GUILD_MEMBER_ID_GENERATOR };

            let mut gen = gen.lock().unwrap();

            gen.yield_id()
        };

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
    pub id: Snowflake,
    pub link_id: Snowflake,
    pub rank_name: String,
    pub role_id: RoleId,
}

impl Default for GuildRank {
    fn default() -> Self {
        Self {
            id: Snowflake(0),
            link_id: Snowflake(0),
            rank_name: String::new(),
            role_id: RoleId::default(),
        }
    }
}

impl GuildRank {
    pub fn new(link_id: Snowflake, rank_name: String, role_id: RoleId) -> Self {
        let id = {
            let gen = unsafe { &*GUILD_RANK_ID_GENERATOR };

            let mut gen = gen.lock().unwrap();

            gen.yield_id()
        };

        Self {
            id,
            link_id,
            rank_name,
            role_id,
        }
    }
}

task!(
    tsync,
    TaskSchedule::Interval(chrono::Duration::hours(1)),
    task::_sync,
);

command!(guildsync, description: "Disocrd <==> GW2 Links");

command!(
    ranks,
    description: "Assign role to rank mappings.",
);

command!(
    show,
    description: "Display the current rank to role assignments.",
    executor: _ranks_list,
);

command!(
    set,
    description: "Map a rank to a role.",
    executor: _ranks_set,
);

async fn _ranks_set(mut ctx: MessageContext) -> Result {
    if ctx.args.len() < 2 {
        return Err(Error::InvalidCommandUsage);
    }

    let rank_name = ctx.args.pop().unwrap();
    let role_id: RoleId = ctx
        .args
        .pop()
        .unwrap()
        .parse()
        .or(Err(Error::InvalidCommandUsage))?;

    let guild_link = commands::get_guild_link(&ctx).await?;

    let ranks = ctx
        .state
        .store()
        .get(GuildRank::query().link_id(guild_link.id))
        .await?;

    for rank in ranks {
        if rank.rank_name == rank_name {
            ctx.state
                .store()
                .delete(GuildRank::query().id(rank.id))
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

async fn _ranks_list(ctx: MessageContext) -> Result {
    let guild_link = commands::get_guild_link(&ctx).await?;

    let ranks = ctx
        .state
        .store()
        .get(GuildRank::query().link_id(guild_link.id))
        .await?;

    let _ = ctx
        .event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("__Rank Mappings__");
                e.description({
                    let mut string = String::new();
                    for rank in ranks {
                        writeln!(string, "<@&{}> => `{}`", rank.role_id, rank.rank_name).unwrap();
                    }
                    string
                });
                e
            });
            m
        })
        .await;

    Ok(())
}
