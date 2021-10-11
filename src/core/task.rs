//! Automatic timed task scheduling.
//!
//!

use {
    super::executor::Executor,
    crate::bot::Context,
    chrono::{DateTime, Duration, Utc},
    std::collections::VecDeque,
    tokio::{
        select,
        sync::{mpsc, oneshot},
        task,
    },
};

#[derive(Clone)]
pub struct Task {
    pub name: String,
    pub schedule: TaskSchedule,
    pub executor: Executor<Context<()>>,
    /// Immedietly schedule a task execution when the
    /// task is first loaded.
    pub on_load: bool,
}

impl Task {
    pub fn next_execution<T>(&self, time: &DateTime<T>) -> DateTime<T>
    where
        T: chrono::TimeZone,
    {
        match self.schedule {
            TaskSchedule::Interval(duration) => time.clone() + duration,
            TaskSchedule::RepeatTime(_) => unimplemented!(),
        }
    }
}

#[derive(Clone)]
pub(crate) struct LoadedTask {
    pub name: String,
    pub schedule: TaskSchedule,
    executor: Executor<Context<()>>,
    /// Next command execution
    pub next_exec: DateTime<Utc>,
}

impl LoadedTask {
    pub fn next_execution<T>(&self, time: &DateTime<T>) -> DateTime<T>
    where
        T: chrono::TimeZone,
    {
        match self.schedule {
            TaskSchedule::Interval(duration) => time.clone() + duration,
            TaskSchedule::RepeatTime(_) => unimplemented!(),
        }
    }
}

#[derive(Clone, Debug)]
pub enum TaskSchedule {
    /// Repeat task based on intervals.
    Interval(Duration),
    /// Schedules a new task every time the time is matched.
    RepeatTime(DateTime<Utc>),
}

impl TaskSchedule {
    // pub fn repeat_time() -> Self {
    //     // Self::RepeatTime()
    // }
}

// pub struct RepeatTimeBuilder(DateTime<Utc>);

// impl RepeatTimeBuilder {
//     pub fn now() -> Self {
//         Self(Utc::now())
//     }

//     pub fn second(mut self, sec: u32) -> Self {
//         Self(self.0.with_second(sec).unwrap())
//     }

//     pub fn minute(mut self, min: u32) -> Self {
//         Self(self.0.with_minute(min).unwrap())
//     }

//     pub fn hour(mut self, hour: u32) -> Self {
//         Self(self.0.with_hour(hour).unwrap())
//     }

//     pub fn day(mut self, day: u32) -> Self {
//         Self(self.0.with_day(day).unwrap())
//     }

//     pub fn month(mut self, month: u32) -> Self {
//         Self(self.0.with_month(month).unwrap())
//     }

//     pub fn year(mut self, year: i32) -> Self {
//         Self(self.0.with_year(year).unwrap())
//     }

//     pub fn weekday(mut self, weekday: Weekday) -> Self {
//         let days = weekday.num_days_from_monday() - self.0.weekday().num_days_from_monday();
//         Self(self.0 + Duration::days(days as i64))
//     }
// }

struct InnerTaskScheduler {
    tasks: VecDeque<LoadedTask>,
    context: Option<Context<()>>,
}

impl InnerTaskScheduler {
    fn new() -> Self {
        Self {
            tasks: VecDeque::new(),
            context: None,
        }
    }

    pub fn add_task(&mut self, task: Task) {
        let now = Utc::now();

        let next_exec = match task.on_load {
            true => now,
            false => task.next_execution(&now),
        };

        let task = LoadedTask {
            name: task.name,
            schedule: task.schedule,
            executor: task.executor,
            next_exec,
        };

        self.queue_task(task);
    }

    /// Put a new task into the queue. The queue is ordered is
    /// ordered by next execution time in ascending order.
    fn queue_task(&mut self, task: LoadedTask) {
        for (i, s_task) in self.tasks.iter().enumerate() {
            if task.next_exec < s_task.next_exec {
                self.tasks.push_front(task);
                self.tasks.swap(i, 0);
                return;
            }
        }

        self.tasks.push_back(task);
    }

    // Panics if `self.tasks` is empty.
    async fn wait(&self) {
        if self.context.is_none() {
            return;
        }

        let now = Utc::now();
        let until = self.tasks[0].next_exec - now;

        if until <= Duration::seconds(0) {
            return;
        }

        tokio::time::sleep(until.to_std().unwrap()).await
    }

    /// Take the next queued task from the queue, dispatch it,
    /// and put it back into the queue.
    /// # Panics
    /// Panics if `self.context` is `None` or the queue is empty.
    fn run_task(&mut self) {
        let mut task = self.tasks.pop_front().unwrap();

        {
            let ctx = self.context.clone();
            let task = task.clone();
            println!("Starting task {}", task.name);
            task::spawn(async move {
                let res = task.executor.send(ctx.unwrap()).await;
                if let Err(err) = res {
                    eprintln!("Task {} returned an error: {:?}", task.name, err);
                }
            });
        }

        // Update the task with the next execution time.
        let now = Utc::now();
        task.next_exec = task.next_execution(&now);

        // Put the task back into the queue.
        self.queue_task(task);
    }

    async fn proccess_message(&mut self, message: TaskSchedulerMessage) {
        match message {
            TaskSchedulerMessage::AddTask(task) => self.add_task(task),
            TaskSchedulerMessage::GetTasks(tx) => {
                let tasks = self.tasks.iter().cloned().collect();
                let _ = tx.send(tasks);
            }
            TaskSchedulerMessage::UpdateContext(ctx) => self.context = ctx,
        }
    }

    fn start(mut self) -> mpsc::Sender<TaskSchedulerMessage> {
        let (tx, mut rx) = mpsc::channel(32);

        tokio::task::spawn(async move {
            loop {
                if self.context.is_none() || self.tasks.is_empty() {
                    let msg = rx.recv().await;
                    self.proccess_message(msg.unwrap()).await;
                    continue;
                }

                select! {
                    _  = self.wait() => self.run_task(),
                    msg = rx.recv() => self.proccess_message(msg.unwrap()).await
                };
            }
        });

        tx
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TaskScheduler {
    tx: mpsc::Sender<TaskSchedulerMessage>,
}

enum TaskSchedulerMessage {
    AddTask(Task),
    UpdateContext(Option<Context<()>>),
    GetTasks(oneshot::Sender<Vec<LoadedTask>>),
}

impl TaskScheduler {
    pub fn new() -> Self {
        let task_scheduler = InnerTaskScheduler::new();
        Self {
            tx: task_scheduler.start(),
        }
    }

    pub async fn add_task(&self, task: Task) {
        let _ = self.tx.send(TaskSchedulerMessage::AddTask(task)).await;
    }

    pub async fn update_context(&self, ctx: Option<Context<()>>) {
        let _ = self.tx.send(TaskSchedulerMessage::UpdateContext(ctx)).await;
    }

    pub async fn get_tasks(&self) -> Vec<LoadedTask> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(TaskSchedulerMessage::GetTasks(tx)).await;
        rx.await.unwrap()
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::{Task, TaskSchedule, TaskScheduler};
    use crate::core::executor::Executor;
    use chrono::Duration;
    use tokio::sync::mpsc;

    #[test]
    fn test_task_scheduler() {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        let (tx, _rx) = mpsc::channel(0);

        rt.block_on(async move {
            async fn as_vec(task_scheduler: &TaskScheduler) -> Vec<String> {
                let tasks = task_scheduler.get_tasks().await;
                tasks.iter().map(|task| task.name.clone()).collect()
            }
            let task_scheduler = TaskScheduler::new();
            task_scheduler
                .add_task(Task {
                    name: String::from("test1"),
                    schedule: TaskSchedule::Interval(Duration::minutes(1)),
                    executor: Executor::new(tx.clone()),
                    on_load: false,
                })
                .await;

            assert_eq!(as_vec(&task_scheduler).await, vec![String::from("test1")]);

            task_scheduler
                .add_task(Task {
                    name: String::from("test2"),
                    schedule: TaskSchedule::Interval(Duration::hours(1)),
                    executor: Executor::new(tx.clone()),
                    on_load: false,
                })
                .await;

            assert_eq!(
                as_vec(&task_scheduler).await,
                vec![String::from("test1"), String::from("test2")]
            );

            task_scheduler
                .add_task(Task {
                    name: String::from("test3"),
                    schedule: TaskSchedule::Interval(Duration::minutes(30)),
                    executor: Executor::new(tx.clone()),
                    on_load: false,
                })
                .await;

            assert_eq!(
                as_vec(&task_scheduler).await,
                vec![
                    String::from("test1"),
                    String::from("test3"),
                    String::from("test2")
                ]
            );
        });
    }
}
