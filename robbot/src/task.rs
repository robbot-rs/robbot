use crate::util::SmallOption;

use chrono::{Datelike, Duration, Timelike};
use std::ops::Add;

pub trait Task: Sized {
    type Executor;

    fn name(&self) -> &str;
    fn schedule(&self) -> &TaskSchedule;
    fn executor(&self) -> &Self::Executor;
    fn on_load(&self) -> bool;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum TaskSchedule {
    /// Repeats the task after a specific interval.
    Interval(Duration),
    /// Repeats the task everytime all requirements are satisfied.
    RepeatTime(DateTimeRequirement),
}

impl TaskSchedule {
    /// Creates a new `TaskSchedule` that runs the task once every minute.
    pub fn minutely() -> Self {
        Self::Interval(Duration::minutes(1))
    }

    /// Creates a new `TaskSchedule` that runs the task once every hour.
    pub fn hourly() -> Self {
        Self::Interval(Duration::hours(1))
    }

    /// Creates a new `TaskSchedule` that runs the task once every day.
    pub fn daily() -> Self {
        Self::Interval(Duration::days(1))
    }

    /// Creates a new `TaskSchedule` that runs the once every day at `00:00:00`.
    pub fn at_midnight() -> Self {
        let mut dt_req = DateTimeRequirement::new();
        dt_req.with_hour(0);

        Self::RepeatTime(dt_req)
    }

    pub fn advance<T>(&self, datetime: T) -> Option<T>
    where
        T: Datelike + Timelike + Add<Duration, Output = T>,
    {
        match self {
            Self::Interval(duration) => Some(datetime + *duration),
            Self::RepeatTime(requirements) => requirements.advance(datetime),
        }
    }
}

/// Defines a number of requirements on a date/time. These requirements are
/// only satisfied when all values of the given date/time have the same values
/// are the requirements.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct DateTimeRequirement {
    second: SmallOption<u8>,
    minute: SmallOption<u8>,
    hour: SmallOption<u8>,
    day: SmallOption<u8>,
    month: SmallOption<u8>,
    year: Option<i32>,
}

impl DateTimeRequirement {
    /// Creates a new [`DateTimeRequirement`] with no requirements.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the requirement for seconds.
    pub fn with_second(&mut self, second: u8) -> Option<&mut Self> {
        if second >= 60 {
            return None;
        }

        // SAFETY: `second` was checked to be a valid value.
        unsafe { Some(self.with_second_unchecked(second)) }
    }

    /// Sets the requirement for seconds without validating `second`.
    ///
    /// # Safety
    /// Calling this method with in invalid `second` value has unspecified effects
    /// on the values returned by [`Self::advance`].
    pub unsafe fn with_second_unchecked(&mut self, second: u8) -> &mut Self {
        self.second = SmallOption::new_unchecked(second);
        self
    }

    /// Sets the requirement for minutes.
    pub fn with_minute(&mut self, minute: u8) -> Option<&mut Self> {
        if minute >= 60 {
            return None;
        }

        // SAFETY: `minute` was checked to be a valid value.
        unsafe { Some(self.with_minute_unchecked(minute)) }
    }

    /// Sets the requirement for minutes without validating `minute`.
    ///
    /// # Safety
    /// Calling this method with in invalid `minute` value has unspecified effects
    /// on the values returned by [`Self::advance`].
    pub unsafe fn with_minute_unchecked(&mut self, minute: u8) -> &mut Self {
        self.minute = SmallOption::new_unchecked(minute);
        self
    }

    /// Sets the requirement for hours.
    pub fn with_hour(&mut self, hour: u8) -> Option<&mut Self> {
        if hour >= 24 {
            return None;
        }

        // SAFETY: `hour` was checked to be a valid value.
        unsafe { Some(self.with_hour_unchecked(hour)) }
    }

    /// Sets the requirement for hours without validating `hour`.
    ///
    /// # Safety
    /// Calling this method with in invalid `hour` value has unspecified effects
    /// on the values returned by [`Self::advance`].
    pub unsafe fn with_hour_unchecked(&mut self, hour: u8) -> &mut Self {
        self.hour = SmallOption::new_unchecked(hour);
        self
    }

    /// Sets the requirement for days.
    /// **Note: Days start counting from 1. A value of `0` is invalid.**
    pub fn with_day(&mut self, day: u8) -> Option<&mut Self> {
        if day == 0 || day >= 32 {
            return None;
        }

        // SAFETY: `day` was checked to be a valid value.
        unsafe { Some(self.with_day_unchecked(day)) }
    }

    /// Sets the requirement for days without validating `day`.
    ///
    /// # Safety
    /// Calling this method with in invalid `day` value has unspecified effects
    /// on the values returned by [`Self::advance`].
    pub unsafe fn with_day_unchecked(&mut self, day: u8) -> &mut Self {
        self.day = SmallOption::new_unchecked(day);
        self
    }

    /// Sets the requirement for months.
    /// **Note: Months start counting from 1. A value of `0` is invalid.**
    pub fn with_month(&mut self, month: u8) -> Option<&mut Self> {
        if month == 0 || month >= 13 {
            return None;
        }

        unsafe { Some(self.with_month_unchecked(month)) }
    }

    /// Sets the requirement for months without validating `month`.
    ///
    /// # Safety
    /// Calling this method with in invalid `month` value has unspecified effects
    /// on the values returned by [`Self::advance`].
    pub unsafe fn with_month_unchecked(&mut self, month: u8) -> &mut Self {
        self.month = SmallOption::new_unchecked(month);
        self
    }

    /// Sets the requirement for the year. You probaly never want to use this. Unless
    /// you somehow travel back in time, setting a year requirement causes the [`Task`]
    /// to  only run a single time. [`advance`] will always return `None` after the time
    ///  passed.
    ///
    /// [`advance`]: Self::advance
    pub fn with_year(&mut self, year: i32) -> &mut Self {
        self.year = Some(year);
        self
    }

    /// Returns `true` if the requirements are ever satisfied in `datetime`. If `is_upcoming`
    /// returns `false` for `datetime`, it will always return `false` for the same type `T`
    /// as long as the time of `T` does not move backwards.
    pub fn is_upcoming<T>(&self, datetime: &T) -> bool
    where
        T: Datelike + Timelike,
    {
        match self.year {
            Some(year) => year <= datetime.year(),
            None => true,
        }
    }

    /// Advances `datetime` to the next Date/Time that satisfies the requirements.
    /// Returns `None` when the next Date/Time doesn't exist or `T` doesn't support
    /// the input ranges.
    pub fn advance<T>(&self, mut datetime: T) -> Option<T>
    where
        T: Datelike + Timelike,
    {
        // Adds 2 `i8`s together and returns the sum as a `u32`.
        // If the sum is negative or the sum over/underflows 0 is returned.
        fn add(a: i8, b: i8) -> u32 {
            match a.checked_add(b) {
                Some(n) => {
                    if n.is_negative() {
                        0
                    } else {
                        n as u32
                    }
                }
                None => 0,
            }
        }

        // If the wanted time value is **before** the current value of `datetime`
        // the next greater time value needs to be increased (e.g. wanted seconds = 3,
        // current seconds = 4, difference = -1: the minute value needs to be increased
        // by 1).
        let mut upcast = false;

        if self.second.is_some() {
            let second = unsafe { self.second.unwrap_unchecked() };

            let current_second = datetime.second() as i8;
            let diff = second as i8 - current_second;

            if diff != 0 {
                if diff < 0 {
                    upcast = true;
                }

                datetime = datetime.with_second(current_second as u32 + diff as u32)?;
            }
        }

        {
            let current_minute = datetime.minute() as i8;
            let minute = self.minute.unwrap_or(current_minute as u8);

            let mut diff = minute as i8 - current_minute;

            println!("{}", diff);

            if upcast {
                diff += 1;
                upcast = false;
            }

            if diff != 0 {
                if diff < 0 {
                    upcast = true;
                }

                let new_minute = add(current_minute, diff);

                datetime = datetime.with_minute(new_minute)?;
            }
        }

        {
            let current_hour = datetime.hour() as i8;
            let hour = self.hour.unwrap_or(current_hour as u8);

            let mut diff = hour as i8 - current_hour;

            if upcast {
                diff += 1;
                upcast = false;
            }

            if diff != 0 {
                if diff < 0 {
                    upcast = true;
                }

                let new_hour = add(current_hour as i8, diff);

                datetime = datetime.with_hour(new_hour)?;
            }
        }

        {
            let current_day = datetime.day() as i8;
            let day = self.day.unwrap_or(current_day as u8);

            let mut diff = day as i8 - current_day;

            if upcast {
                diff += 1;
                upcast = false;
            }

            if diff != 0 {
                if diff < 0 {
                    upcast = true;
                }

                let new_day = add(current_day as i8, diff);

                datetime = datetime.with_day(new_day)?;
            }
        }

        {
            let current_month = datetime.month() as i8;
            let month = self.month.unwrap_or(current_month as u8);

            let mut diff = month as i8 - current_month;

            if upcast {
                diff += 1;
                upcast = false;
            }

            if diff != 0 {
                if diff < 0 {
                    upcast = true;
                }

                let new_month = add(current_month, diff);

                datetime = datetime.with_month(new_month)?;
            }
        }

        {
            let current_year = datetime.year();
            let year = self.year.unwrap_or(current_year);

            let mut diff = year - current_year;

            if upcast {
                diff += 1;
            }

            if diff != 0 {
                // We can't go back in time. The wanted year is before the
                // current year.
                if diff < 0 {
                    return None;
                }

                let new_year = current_year + diff;

                datetime = datetime.with_year(new_year)?;
            }
        }

        Some(datetime)
    }

    /// Returns `true` if all requirements are satisfied on `datetime`.
    pub fn matches<T>(&self, datetime: &T) -> bool
    where
        T: Datelike + Timelike,
    {
        // Check the seconds.
        if self.second.is_some() {
            let second = unsafe { self.second.unwrap_unchecked() as u32 };

            if second != datetime.second() {
                return false;
            }
        }

        // Check the minutes.
        if self.minute.is_some() {
            let minute = unsafe { self.minute.unwrap_unchecked() as u32 };

            if minute != datetime.minute() {
                return false;
            }
        }

        // Check the hours.
        if self.hour.is_some() {
            let hour = unsafe { self.hour.unwrap_unchecked() as u32 };

            if hour != datetime.hour() {
                return false;
            }
        }

        // Check the days.
        if self.day.is_some() {
            let day = unsafe { self.day.unwrap_unchecked() as u32 };

            if day != datetime.day() {
                return false;
            }
        }

        // Check the months.
        if self.month.is_some() {
            let month = unsafe { self.month.unwrap_unchecked() as u32 };

            if month != datetime.month() {
                return false;
            }
        }

        // Check the year.
        if let Some(year) = self.year {
            if year != datetime.year() {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::{DateTimeRequirement, TaskSchedule};

    use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, NaiveTime, Timelike};

    #[test]
    fn test_task_schedule_interval() {
        let schedule = TaskSchedule::minutely();
        assert_eq!(schedule, TaskSchedule::Interval(Duration::minutes(1)));

        let schedule = TaskSchedule::hourly();
        assert_eq!(schedule, TaskSchedule::Interval(Duration::hours(1)));

        let schedule = TaskSchedule::daily();
        assert_eq!(schedule, TaskSchedule::Interval(Duration::days(1)));
    }

    #[test]
    fn test_date_time_requirement() {
        let date = NaiveDate::from_ymd(2022, 2, 15);
        let time = NaiveTime::from_hms(3, 7, 0);
        let dt = NaiveDateTime::new(date, time);

        let mut repeat_dt = DateTimeRequirement::new();
        repeat_dt.with_second(1).unwrap();

        assert_eq!(repeat_dt.advance(dt).unwrap(), dt.with_second(1).unwrap());

        let mut repeat_dt = DateTimeRequirement::new();
        repeat_dt.with_second(32).unwrap();
        repeat_dt.with_minute(23).unwrap();

        assert_eq!(
            repeat_dt.advance(dt).unwrap(),
            dt.with_second(32).unwrap().with_minute(23).unwrap()
        );

        let mut repeat_dt = DateTimeRequirement::new();
        repeat_dt.with_second(32).unwrap();
        repeat_dt.with_minute(6).unwrap();

        assert_eq!(
            repeat_dt.advance(dt).unwrap(),
            dt.with_second(32)
                .unwrap()
                .with_minute(6)
                .unwrap()
                .with_hour(4)
                .unwrap()
        );

        let mut repeat_dt = DateTimeRequirement::new();
        repeat_dt.with_day(14).unwrap();

        assert_eq!(
            repeat_dt.advance(dt).unwrap(),
            dt.with_day(14).unwrap().with_month(3).unwrap()
        );

        let mut repeat_dt = DateTimeRequirement::new();
        repeat_dt.with_year(2021);

        assert_eq!(repeat_dt.advance(dt), None);
    }
}
