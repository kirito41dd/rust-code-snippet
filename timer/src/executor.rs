use {
    futures::{
        future::{FutureExt, BoxFuture},
        task::{ArcWake, waker_ref},
    },
    std::{
        future::Future,
        sync::{Arc, Mutex},
        sync::mpsc::{sync_channel, SyncSender, Receiver},
        task::{Context, Poll},
    },
};

/// 从管道中接收任务并运行他们的执行程序
pub struct Executor {
    ready_queue: Receiver<Arc<Task>>,
}

/// Spawner 用于在任务管道中创建新的 Future
#[derive(Clone)]
pub struct Spawner {
    task_sender: SyncSender<Arc<Task>>,
}

/// Executor 负责执行Future, Spawner 负责创建新的Future, 通过channel进行通信
pub fn new_executor_and_spawner() -> (Executor, Spawner) {
    const MAX_QUEUED_TASK: usize = 10_000;
    let (task_sender, ready_queue) = sync_channel(MAX_QUEUED_TASK);
    (Executor{ready_queue}, Spawner{task_sender})
}

/// 一个可以重新安排自己被 Executor 调用的 Future
struct Task {
    ///正在运行的future
    future: Mutex<Option<BoxFuture<'static, ()>>>,
    /// 将任务放回到任务队列
    task_sender: SyncSender<Arc<Task>>
}

impl Spawner {
    /// 创建新的 Future， 发送到队列
    pub fn spawn(&self, future: impl Future<Output=()> + 'static + Send) {
        let future = future.boxed();
        let task = Arc::new(Task {
            future: Mutex::new(Some(future)),
            task_sender: self.task_sender.clone(),
        });
        self.task_sender.send(task).expect("too many tasks queued");
    }
}

/// 创建 Waker 的最简单方法是实现 ArcWake 特质，
/// 然后使用 waker_ref 或者 .into_waker() 函数
/// 把 Arc<impl ArcWake> 转变成 Waker。
impl ArcWake for Task {
    fn wake_by_ref(arc_self: &Arc<Self>) {
        // 通过将这个任务发送回任务管道来实现 `wake`，
        // 以便让执行器再次轮询它.
        let cloned = arc_self.clone();
        arc_self.task_sender.send(cloned).expect("too many tasks queued");
    }
}

impl Executor {
    /// 从队列中接收Future并执行
    pub fn run(&self) {
        // 接收 task
        while let Ok(task) = self.ready_queue.recv() {
            // 拿到future
            let mut future_slot = task.future.lock().unwrap();
            // 取出 BoxFuture<>
            if let Some(mut future) = future_slot.take() {
                // 通过ArcWake拿到waker, 再拿到context
                let waker = waker_ref(&task);
                let context = &mut Context::from_waker(&*waker);
                // `BoxFuture<T>` 是 `Pin<Box<dyn Future<Output = T> + Send + 'static>>` 的类型别名.
                // 我们可以调用 `Pin::as_mut` 方法获得 `Pin<&mut dyn Future + Send + 'static>`.
                if let Poll::Pending = future.as_mut().poll(context) {
                    // 我们还没有完成对 `future` 的处理，所以把它再次
                    // 放回它的任务中，以便在某个时段再次运行.
                    *future_slot = Some(future);
                }
            }

        }
    }
}