use crate::executor::Executor;
use chrono::{DateTime, Duration, TimeZone, Timelike, Utc};

#[derive(Clone)]
pub struct Task<T> {
    pub name: String,
    pub schedule: TaskSchedule,
    pub executor: Executor<T>,
    /// Immedietly schedule a task execution when the
    /// task is first loaded. After that the next task
    /// will be queued using [`TaskSchedule`].
    pub on_load: bool,
}

impl<T> Task<T> {
    /// Calculate the next execution time for the task based on the
    /// current time `time`.
    pub fn next_execution<Tz>(&self, time: DateTime<Tz>) -> DateTime<Tz>
    where
        Tz: TimeZone,
    {
        match self.schedule {
            TaskSchedule::Interval(duration) => time + duration,
            TaskSchedule::RepeatTime(mut timestamp, interval) => {
                let offset = time.offset().clone();

                while time >= timestamp {
                    timestamp = timestamp + interval;
                }

                DateTime::from_utc(timestamp.naive_utc(), offset)
            }
        }
    }
}

/// A type to schedule command execution. [`TaskSchedule`] can
/// be one of two variants:
/// - `Interval`: Schedule the future task based on the interval
/// between the previous task.
/// - `RepeatTime`: Schedule the future task when the specific
/// time conditions are met.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskSchedule {
    Interval(Duration),
    /// RepeatTime schedules the task when the `DateTime` is reached,
    /// then waits the `Duration` before the next task.
    RepeatTime(DateTime<Utc>, Duration),
}

impl TaskSchedule {
    /// Create a [`TaskScheduler`] that runs every
    /// `i` seconds.
    pub fn seconds(i: i64) -> Self {
        Self::Interval(Duration::seconds(i))
    }

    /// Create a [`TaskScheduler`] that runs every
    /// `i` minutes.
    pub fn minutes(i: i64) -> Self {
        Self::Interval(Duration::minutes(i))
    }

    /// Create a [`TaskScheduler`] that runs every
    /// `i` hours.
    pub fn hours(i: i64) -> Self {
        Self::Interval(Duration::hours(i))
    }

    /// Create a [`TaskScheduler`] that runs every
    /// `i` days.
    pub fn days(i: i64) -> Self {
        Self::Interval(Duration::days(i))
    }

    /// Create a [`TaskScheduler`] that runs every hour.
    pub fn hourly() -> Self {
        Self::hours(1)
    }

    /// Create a [`TaskScheduler`] that runs every day (24h).
    pub fn daily() -> Self {
        Self::days(1)
    }

    /// Create a [`TaskScheduler`] that runs every week (7d).
    pub fn weekly() -> Self {
        Self::days(7)
    }

    /// Create a [`TaskScheduler`] that runs every day at the
    /// time `hour:minute:second`.
    /// # Panics
    /// Panics if `hour`, `minute` or `second` are invalid.
    pub fn daily_at(hour: u32, minute: u32, second: u32) -> Self {
        let now = Utc::now();
        let schedule = now
            .with_hour(hour)
            .unwrap()
            .with_minute(minute)
            .unwrap()
            .with_second(second)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        Self::RepeatTime(schedule, Duration::days(1))
    }
}

#[cfg(test)]
mod tests {
    use super::{Task, TaskSchedule};
    use crate::executor::Executor;
    use chrono::{Duration, Timelike, Utc};
    use tokio::sync::mpsc;

    #[test]
    fn test_task_schedule_interval() {
        let schedule = TaskSchedule::Interval(Duration::hours(1));
        assert_eq!(schedule, TaskSchedule::Interval(Duration::hours(1)));

        let schedule = TaskSchedule::seconds(1);
        assert_eq!(schedule, TaskSchedule::Interval(Duration::seconds(1)));

        let schedule = TaskSchedule::minutes(1);
        assert_eq!(schedule, TaskSchedule::Interval(Duration::minutes(1)));

        let schedule = TaskSchedule::hours(1);
        assert_eq!(schedule, TaskSchedule::Interval(Duration::hours(1)));

        let schedule = TaskSchedule::days(1);
        assert_eq!(schedule, TaskSchedule::Interval(Duration::days(1)));

        let schedule = TaskSchedule::hourly();
        assert_eq!(schedule, TaskSchedule::hours(1));

        let schedule = TaskSchedule::daily();
        assert_eq!(schedule, TaskSchedule::days(1));

        let schedule = TaskSchedule::weekly();
        assert_eq!(schedule, TaskSchedule::days(7));
    }

    #[test]
    fn test_task_schedule_repeat_time() {
        let now = Utc::now();

        let schedule = TaskSchedule::RepeatTime(now, Duration::days(1));
        assert_eq!(schedule, TaskSchedule::RepeatTime(now, Duration::days(1)));

        let schedule = TaskSchedule::daily_at(21, 0, 0);
        assert_eq!(
            schedule,
            TaskSchedule::RepeatTime(
                now.with_hour(21)
                    .unwrap()
                    .with_minute(0)
                    .unwrap()
                    .with_second(0)
                    .unwrap()
                    .with_nanosecond(0)
                    .unwrap(),
                Duration::days(1)
            )
        );
    }

    #[test]
    fn test_task_next_execution() {
        let (tx, _rx) = mpsc::channel(1);
        let now = Utc::now();

        let mut task = Task {
            name: String::from("test"),
            schedule: TaskSchedule::seconds(1),
            executor: Executor::<()>::new(tx),
            on_load: false,
        };

        task.schedule = TaskSchedule::hours(1);
        assert_eq!(task.next_execution(now), now + Duration::hours(1));

        task.schedule = TaskSchedule::days(2);
        assert_eq!(task.next_execution(now), now + Duration::days(2));

        task.schedule = TaskSchedule::daily_at(23, 0, 0);
        let mut output = now
            .with_hour(23)
            .unwrap()
            .with_minute(0)
            .unwrap()
            .with_second(0)
            .unwrap()
            .with_nanosecond(0)
            .unwrap();

        if now > output {
            output = output + Duration::days(1);
        }

        assert_eq!(task.next_execution(now), output);
    }
}
