mod commands;
// mod sql;
mod tasks;

use robbot::model::id::GuildId;
use robbot::store::id::Snowflake;
use robbot::store::{create, StoreData};
use robbot_core::command::Command;
use robbot_core::state::State;

use robbot::Result;

#[derive(Clone, Debug, StoreData)]
pub(crate) struct Event {
    pub id: Snowflake,
    pub guild_id: GuildId,
    pub title: String,
    pub description: String,
    pub time: u64,
    pub timezone: String,
    pub repeat_time: u64,
}

impl Default for Event {
    fn default() -> Self {
        Self {
            id: Snowflake(0),
            guild_id: GuildId(0),
            title: String::new(),
            description: String::new(),
            time: 0,
            timezone: String::new(),
            repeat_time: 0,
        }
    }
}

pub async fn init(state: &State) -> Result {
    create!(state.store(), Event).await?;

    state.commands().load_command(events(), None)?;
    state
        .commands()
        .load_command(commands::list(), Some("events"))?;
    state
        .commands()
        .load_command(commands::create(), Some("events"))?;

    // state
    //     .add_command(commands::delete(), Some("events"))
    //     .unwrap();

    state.tasks().add_task(announce()).await;

    Ok(())
}

crate::task!(
    announce,
    robbot::task::TaskSchedule::Interval(chrono::Duration::hours(1)),
    tasks::_announce,
);

fn events() -> Command {
    let mut cmd = Command::new("events");
    cmd.set_description("events");
    cmd
}
