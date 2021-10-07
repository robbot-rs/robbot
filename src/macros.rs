#[macro_export]
macro_rules! command {
    ($name:ident $(, description: $description:expr)? $(, arguments: $arguments:expr)? $(, executor: $executor:expr)?$(,)?) => {
        fn $name() -> $crate::core::command::Command {
            let mut cmd = $crate::core::command::Command::new(stringify!($name).to_owned());

            $(
                cmd.description = $description.to_string();
            )?

            $(
                let executor = $crate::core::executor::Executor::from_fn($executor);
                cmd.executor = ::std::option::Option::Some(::std::boxed::Box::leak(::std::boxed::Box::new(executor)));
            )?

            cmd
        }
    };
}

#[macro_export]
macro_rules! task {
    ($name:ident, $schedule:expr, $executor:expr $(,)?) => {
        fn $name() -> $crate::core::task::Task {
            $crate::core::task::Task {
                name: stringify!($name).to_owned(),
                schedule: $schedule,
                executor: {
                    let executor = $crate::core::executor::Executor::from_fn($executor);
                    ::std::boxed::Box::leak(::std::boxed::Box::new(executor))
                },
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
