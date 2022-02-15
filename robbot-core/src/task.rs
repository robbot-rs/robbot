use crate::context::Context;
use crate::executor::Executor;

use robbot::executor::Executor as ExecutorExt;
use robbot::task::TaskSchedule;

use chrono::{DateTime, Duration, TimeZone, Utc};
use std::collections::VecDeque;
use tokio::{
    select,
    sync::{mpsc, oneshot},
    task, time,
};

const SCHEDULER_MESSAGEQUEUE_SIZE: usize = 32;

#[derive(Clone)]
pub struct Task {
    pub name: String,
    pub schedule: TaskSchedule,
    pub executor: Executor<Context<()>>,

    pub on_load: bool,
}

impl Task {
    pub fn new<T>(name: T, schedule: TaskSchedule, executor: Executor<Context<()>>) -> Self
    where
        T: ToString,
    {
        Self {
            name: name.to_string(),
            schedule,
            executor,
            on_load: false,
        }
    }

    pub fn next_execution<T>(&self, time: DateTime<T>) -> DateTime<T>
    where
        T: TimeZone,
    {
        self.schedule.advance(time).unwrap()
    }
}

impl From<LoadedTask> for Task {
    fn from(task: LoadedTask) -> Self {
        Self {
            name: task.name,
            schedule: task.schedule,
            executor: task.executor,

            on_load: false,
        }
    }
}

#[derive(Clone)]
struct LoadedTask {
    name: String,
    schedule: TaskSchedule,
    executor: Executor<Context<()>>,

    execution_time: DateTime<Utc>,
}

impl LoadedTask {
    pub fn next_execution<T>(&self, time: DateTime<T>) -> DateTime<T>
    where
        T: TimeZone,
    {
        match self.schedule {
            TaskSchedule::Interval(duration) => time + duration,
            TaskSchedule::RepeatTime(_) => unimplemented!(),
        }
    }
}

impl From<Task> for LoadedTask {
    fn from(task: Task) -> LoadedTask {
        let now = Utc::now();

        let execution_time = match task.on_load {
            true => now,
            false => task.next_execution(now),
        };

        Self {
            name: task.name,
            schedule: task.schedule,
            executor: task.executor,
            execution_time,
        }
    }
}

#[derive(Clone, Default)]
struct TaskQueue {
    tasks: VecDeque<LoadedTask>,
}

impl TaskQueue {
    /// Pushes a new task into the queue.
    fn push<T>(&mut self, task: T)
    where
        T: Into<LoadedTask>,
    {
        let task = task.into();

        for (i, t) in self.tasks.iter().enumerate() {
            if task.execution_time < t.execution_time {
                self.tasks.push_front(task);
                self.tasks.swap(i, 0);
                return;
            }
        }

        self.tasks.push_back(task);
    }

    /// Returns the next task to be executed.
    fn pop(&mut self) -> Option<LoadedTask> {
        self.tasks.pop_front()
    }

    /// Wait until the next task reaches it's execution
    /// time, then pops the task.
    /// If the Future is dropped before the execution time
    /// is reached, the task will not be removed.
    async fn await_pop(&mut self) -> Option<LoadedTask> {
        let now = Utc::now();

        let time_wait = self.get(0)?.execution_time - now;

        if time_wait > Duration::seconds(0) {
            time::sleep(time_wait.to_std().unwrap()).await;
        }

        self.pop()
    }

    fn get(&mut self, index: usize) -> Option<&LoadedTask> {
        self.tasks.get(index)
    }

    /// Returns the number of queued tasks.
    fn len(&self) -> usize {
        self.tasks.len()
    }

    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[derive(Clone, Default)]
struct InnerTaskScheduler {
    tasks: TaskQueue,
    context: Option<Context<()>>,
}

impl InnerTaskScheduler {
    fn new() -> Self {
        Self::default()
    }

    fn add_task(&mut self, task: Task) {
        log::info!("[TASK] Added new task '{}'", task.name);
        self.tasks.push(task);
    }

    fn get_tasks(&self, tx: oneshot::Sender<Vec<(Task, DateTime<Utc>)>>) {
        let tasks = self
            .tasks
            .tasks
            .clone()
            .into_iter()
            .map(|task| (task.clone().into(), task.execution_time))
            .collect();

        let _ = tx.send(tasks);
    }

    fn update_context(&mut self, context: Option<Context<()>>) {
        self.context = context;
    }

    async fn await_task(&mut self) {
        // Wait until the execution time is reached.
        let mut task = self.tasks.await_pop().await.unwrap();

        {
            let task = task.clone();
            let ctx = self.context.clone();

            task::spawn(async move {
                log::info!("Spawning task {}", task.name);

                let res = task.executor.send(ctx.unwrap()).await;
                match res {
                    Ok(_) => log::info!("Task {} completed", task.name),
                    Err(err) => log::error!("Task {} failed: {:?}", task.name, err),
                }
            });
        }

        let now = Utc::now();
        task.execution_time = task.next_execution(now);

        // Put the task back into the queue.
        self.tasks.push(task);
    }

    async fn handle_message(&mut self, message: TaskSchedulerMessage) {
        match message {
            TaskSchedulerMessage::AddTask(task) => self.add_task(task),
            TaskSchedulerMessage::GetTasks(tx) => self.get_tasks(tx),
            TaskSchedulerMessage::UpdateContext(ctx) => self.update_context(ctx),
        }
    }

    fn start(mut self) -> mpsc::Sender<TaskSchedulerMessage> {
        let (tx, mut rx) = mpsc::channel(SCHEDULER_MESSAGEQUEUE_SIZE);

        task::spawn(async move {
            loop {
                // While no tasks are queued or no context is given no
                // tasks can be executed.
                if self.tasks.is_empty() || self.context.is_none() {
                    let msg = rx.recv().await.unwrap();
                    self.handle_message(msg).await;
                    continue;
                }

                select! {
                    _ = self.await_task() => {}
                    msg = rx.recv() => self.handle_message(msg.unwrap()).await
                }
            }
        });

        tx
    }
}

enum TaskSchedulerMessage {
    AddTask(Task),
    GetTasks(oneshot::Sender<Vec<(Task, DateTime<Utc>)>>),
    UpdateContext(Option<Context<()>>),
}

#[derive(Clone, Debug)]
pub struct TaskScheduler {
    tx: mpsc::Sender<TaskSchedulerMessage>,
}

impl TaskScheduler {
    /// Creates a new `TaskScheduler` with a new internal
    /// task queue.
    pub fn new() -> Self {
        let inner = InnerTaskScheduler::new();

        Self { tx: inner.start() }
    }

    /// Add a new task to the task queue.
    pub async fn add_task(&self, task: Task) {
        let _ = self.tx.send(TaskSchedulerMessage::AddTask(task)).await;
    }

    pub async fn get_tasks(&self) -> Vec<(Task, DateTime<Utc>)> {
        let (tx, rx) = oneshot::channel();

        let _ = self.tx.send(TaskSchedulerMessage::GetTasks(tx)).await;

        rx.await.unwrap()
    }

    pub async fn update_context(&self, ctx: Option<Context<()>>) {
        let _ = self.tx.send(TaskSchedulerMessage::UpdateContext(ctx)).await;
    }
}

impl Default for TaskScheduler {
    fn default() -> Self {
        Self::new()
    }
}
