// use super::sql::Event;
use super::Event;

use chrono::{Datelike, Duration, NaiveDateTime, TimeZone, Timelike};
use robbot::builder::CreateMessage;
use robbot::prelude::*;
use std::fmt::Write;

use robbot::arguments::ArgumentsExt;
use robbot::store::id::Snowflake;
use robbot::StoreData;
use robbot_core::context::MessageContext;

// #[command(
//     description = "Delete an event.",
//     usage = "<EventID>",
//     guild_only = true
// )]
// async fn delete(mut ctx: MessageContext) -> Result {
//     let event_id: u64 = ctx.args.pop_first()?;

//     let event = Event::get_id(event_id, &ctx.state.store).await?;
//     event.delete(&ctx.state.store).await?;

//     let _ = ctx
//         .respond(format!(
//             ":white_check_mark: Deleted event `{}`.",
//             event.title
//         ))
//         .await;
//     Ok(())
// }

#[command(
    description = "Create a new event.",
    usage = "<Title> <Description> <Date> <Timezone/Location> [Repeat Interval]",
    guild_only = true
)]
async fn create(mut ctx: MessageContext) -> Result {
    let title: String = ctx.args.pop_parse()?;
    let description: String = ctx.args.pop_parse()?;
    let date: String = ctx.args.pop_parse()?;
    let timezone: String = ctx.args.pop_parse()?;

    // Optional repeat time
    let mut repeat_time = Duration::seconds(0);
    if !ctx.args.is_empty() {
        let arg: String = ctx.args.pop_parse()?;
        match parse_repeat_time(&arg) {
            Some(t) => repeat_time = t,
            None => return Err(InvalidCommandUsage),
        }
    }

    // Parse the naive event time with defaults based on
    // UTC time.
    let date = match parse_datetime(&date) {
        Some(t) => t,
        None => return Err(InvalidCommandUsage),
    };

    // Parse the the timezone argument.
    let timezone: chrono_tz::Tz = match timezone.parse() {
        Ok(tz) => tz,
        Err(_) => {
            let _ = ctx.respond(format!(":x: Invalid timezone `{}`.", timezone));
            return Ok(());
        }
    };

    let event = Event {
        id: Snowflake(0),
        guild_id: ctx.event.guild_id.unwrap(),
        title,
        description,
        time: date.timestamp() as u64,
        timezone: timezone.to_string(),
        repeat_time: repeat_time.num_seconds() as u64,
    };

    ctx.state.store().insert(event).await?;

    let _ = ctx.respond(":white_check_mark: Created new event.").await;
    Ok(())
}

#[command(description = "List all upcoming events.", guild_only = true)]
async fn list(ctx: MessageContext) -> Result {
    let events = ctx
        .state
        .store()
        .get(Event::query().guild_id(ctx.event.guild_id.unwrap()))
        .await?;

    let description = match events.len() {
        0 => String::from("No upcoming events."),
        _ => {
            let mut description = String::new();
            for event in events {
                let tz: chrono_tz::Tz = event.timezone.parse().unwrap();
                let event_time = tz
                    .from_local_datetime(&NaiveDateTime::from_timestamp(event.time as i64, 0))
                    .unwrap();

                let _ = writeln!(
                    description,
                    "[{}]: `{}` => <t:{}:F> (Naive {})",
                    event.id.to_string(),
                    event.title,
                    event_time.timestamp(),
                    event.timezone
                );
            }

            description
        }
    };

    let _ = ctx
        .respond(CreateMessage::new(|m| {
            m.embed(|e| {
                e.title("__Upcoming Events__");
                e.description(description);
            });
        }))
        .await?;
    Ok(())
}

fn parse_datetime(s: &str) -> Option<NaiveDateTime> {
    let mut date = chrono::Utc::now()
        .naive_utc()
        .with_second(0)
        .unwrap()
        .with_nanosecond(0)
        .unwrap();

    let parts: Vec<&str> = s.split(' ').collect();
    for (i, part) in parts.iter().enumerate() {
        match i {
            // Date component
            0 => {
                let parts: Vec<&str> = part.split('.').collect();
                for (i, part) in parts.iter().enumerate() {
                    let n = part.parse().ok()?;

                    match i {
                        0 => date = date.with_day(n)?,
                        1 => date = date.with_month(n)?,
                        2 => date = date.with_year(n as i32)?,
                        _ => return None,
                    }
                }
            }
            // Time component
            1 => {
                let parts: Vec<&str> = part.split(':').collect();
                for (i, part) in parts.iter().enumerate() {
                    let n = part.parse().ok()?;

                    match i {
                        0 => date = date.with_hour(n)?,
                        1 => date = date.with_minute(n)?,
                        2 => date = date.with_second(n)?,
                        _ => return None,
                    }
                }
            }
            _ => return None,
        }
    }

    Some(date)
}

fn parse_repeat_time(s: &str) -> Option<Duration> {
    let (i, _) = s.chars().enumerate().find(|(_, c)| !c.is_ascii_digit())?;

    let num = s[0..i].parse().ok()?;

    match &s[i..] {
        "s" | "second" | "seconds" => Some(Duration::seconds(num)),
        "m" | "minute" | "minutes" => Some(Duration::minutes(num)),
        "h" | "hour" | "hours" => Some(Duration::hours(num)),
        "d" | "day" | "days" => Some(Duration::days(num)),
        "w" | "week" | "weeks" => Some(Duration::weeks(num)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::parse_repeat_time;
    use chrono::Duration;

    #[test]
    fn test_parse_repeat_time() {
        let input = "123s";
        assert_eq!(parse_repeat_time(input).unwrap(), Duration::seconds(123));

        let input = "3weeks";
        assert_eq!(parse_repeat_time(input).unwrap(), Duration::weeks(3));
    }
}
