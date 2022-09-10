//! # Temprole
//! Allows to mark spcific roles as timed roles. When
//! a temporary role is assigned, it will be removed
//! after the set timer ran out.
mod hooks;
// mod sql;
mod tasks;

use crate::core::store::{id::SnowflakeID, StoreData};
use crate::{
    bot::MessageContext,
    core::{state::State, task::TaskSchedule},
    hook, task,
};
use chrono::Duration;
use robbot::prelude::*;
use robbot::{hook::EventKind, Context};
use robbot_derive::StoreData;
use serenity::model::id::{GuildId, RoleId, UserId};
use std::{fmt::Write, sync::Arc};

#[derive(Clone, Debug, Default, StoreData)]
pub struct TempRole {
    pub id: SnowflakeID,
    pub guild_id: GuildId,
    pub role_id: RoleId,
    pub lifetime: u64,
}

impl TempRole {
    pub fn new(guild_id: GuildId, role_id: RoleId, lifetime: u64) -> Self {
        Self {
            id: SnowflakeID::new(),
            guild_id,
            role_id,
            lifetime,
        }
    }
}

#[derive(Clone, Debug, Default, StoreData)]
pub struct AssignedRole {
    pub id: SnowflakeID,
    pub guild_id: GuildId,
    pub role_id: RoleId,
    pub user_id: UserId,
    pub remove_timestamp: i64,
}

impl AssignedRole {
    pub fn new(guild_id: GuildId, role_id: RoleId, user_id: UserId, remove_timestamp: i64) -> Self {
        Self {
            id: SnowflakeID::new(),
            guild_id,
            role_id,
            user_id,
            remove_timestamp,
        }
    }
}

pub fn init(state: Arc<State>) {
    state.add_command(temprole(), None).unwrap();
    state.add_command(list(), Some("temprole")).unwrap();
    state.add_command(set(), Some("temprole")).unwrap();

    state.add_task(sync_roles());
    state.add_task(clear_roles());

    on_member_update(state.clone());

    tokio::task::spawn(async move {
        state
            .store
            .as_ref()
            .unwrap()
            .create::<TempRole>()
            .await
            .unwrap();
        state
            .store
            .as_ref()
            .unwrap()
            .create::<AssignedRole>()
            .await
            .unwrap();
    });
}

crate::command!(
    temprole,
    description: "Mark roles as temporary."
);

task!(
    sync_roles,
    TaskSchedule::Interval(chrono::Duration::hours(1)),
    tasks::sync_roles,
);

task!(
    clear_roles,
    TaskSchedule::Interval(chrono::Duration::hours(1)),
    tasks::clear_roles,
);

hook!(
    on_member_update,
    EventKind::GuildMemberUpdate,
    hooks::guild_member_update,
);

#[command(description = "List all temporary roles.", guild_only = true)]
async fn list(ctx: MessageContext) -> Result {
    let roles: Vec<TempRole> = ctx
        .state
        .store
        .as_ref()
        .unwrap()
        .get(Some(
            TempRole::query().guild_id(ctx.event.guild_id.unwrap()),
        ))
        .await?;

    let mut string = String::new();
    for role in roles {
        write!(
            string,
            "<@&{}>: {}",
            role.role_id.0,
            Duration::seconds(role.lifetime as i64)
        )
        .unwrap();
    }

    let _ = ctx
        .event
        .channel_id
        .send_message(&ctx.raw_ctx, |m| {
            m.embed(|e| {
                e.title("Temporary Roles");
                e.description(string);
                e
            });
            m
        })
        .await;
    Ok(())
}

#[command(description = "Set a temporary lifetime of a role.", guild_only = true)]
async fn set(mut ctx: MessageContext) -> Result {
    if ctx.args.len() != 2 {
        return Err(InvalidCommandUsage);
    }

    let role_id: RoleId = ctx.args.remove(0).parse().or(Err(InvalidCommandUsage))?;
    let lifetime = ctx.args.remove(0);

    let lifetime = parse_lifetime(&lifetime).ok_or(InvalidCommandUsage)?;

    let temprole = TempRole::new(
        ctx.event.guild_id.unwrap(),
        role_id,
        lifetime.num_seconds() as u64,
    );

    ctx.state.store.as_ref().unwrap().insert(temprole).await?;

    let _ = ctx
        .respond(format!("Set lifetime of {} on <@&{}>.", lifetime, role_id))
        .await;
    Ok(())
}

fn parse_lifetime(lifetime: &str) -> Option<Duration> {
    let (i, _) = lifetime
        .chars()
        .enumerate()
        .find(|(_, c)| !c.is_ascii_digit())?;

    let num: i64 = lifetime[0..i].parse().ok()?;

    Some(match &lifetime[i..] {
        "s" | "second" | "seconds" => Duration::seconds(num),
        "m" | "minute" | "minutes" => Duration::minutes(num),
        "h" | "hour" | "hours" => Duration::hours(num),
        "d" | "day" | "days" => Duration::days(num),
        _ => return None,
    })
}

#[cfg(test)]
mod test {
    use super::parse_lifetime;
    use chrono::Duration;

    #[test]
    fn test_parse_lifetime() {
        assert_eq!(parse_lifetime("none"), None);
        assert_eq!(parse_lifetime("100s").unwrap(), Duration::seconds(100));
        assert_eq!(parse_lifetime("300m").unwrap(), Duration::minutes(300));
        assert_eq!(parse_lifetime("9123hours").unwrap(), Duration::hours(9123));
    }
}
