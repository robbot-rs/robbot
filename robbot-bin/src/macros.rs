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
                cmd.executor = ::std::option::Option::Some($crate::core::command::CommandExecutor::Message(executor));
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
                executor: $crate::core::executor::Executor::from_fn($executor),
                on_load: true,
            }
        }
    };
}

#[macro_export]
macro_rules! hook {
    ($name:ident, $event:expr, $executor:expr $(,)?) => {
        fn $name(state: ::std::sync::Arc<$crate::core::state::State>) {
            let hook = robbot::hook::Hook {
                name: stringify!($name).to_owned(),
                on_event: $event,
            };

            ::tokio::task::spawn(async move {
                let mut rx = state.add_hook(hook).await;

                ::tokio::task::spawn(async move {
                    let executor = $crate::core::executor::Executor::from_fn($executor);
                    while let Ok(event) = rx.recv().await {
                        let ctx = match event {
                            $crate::core::hook::Event::GuildMemberUpdate(ctx) => ctx,
                            _ => unreachable!(),
                        };
                        if let Err(err) = executor.send(*ctx).await {
                            eprintln!("[Hook] Hook execution failed: {:?}", err);
                        }
                    }
                });
            });
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
