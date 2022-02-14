#[macro_export]
macro_rules! task {
    ($name:ident, $schedule:expr, $executor:expr $(,)?) => {
        fn $name() -> robbot_core::task::Task {
            use ::robbot::executor::Executor as _;

            robbot_core::task::Task {
                name: stringify!($name).to_owned(),
                schedule: $schedule,
                executor: robbot_core::executor::Executor::from_fn($executor),
                on_load: true,
            }
        }
    };
}

#[macro_export]
macro_rules! task_schedule {
    // Interval
    (sec: $sec:expr) => {{
        const INT: ::std::time::Duration = ::std::time::Duration::from_secs($sec);
        $crate::core::task::TaskSchedule::Interval(INT)
    }};
    (min: $min:expr) => {{
        const INT: ::std::time::Duration = ::std::time::Duration::from_secs($min * 60);
        $crate::core::task::TaskSchedule::Interval(INT)
    }};
    (hrs: $hrs:expr) => {{
        const INT: ::std::time::Duration = ::std::time::Duration::from_secs($hrs * 60 * 60);
        $crate::core::task::TaskSchedule::Interval(INT)
    }};
    (hourly) => {{
        task_schedule!(min: 60)
    }};
    (daily) => {{
        task_schedule!(hrs: 24)
    }};
    (weekly) => {{
        task_schedule!(hrs: 24 * 7)
    }};
}
