use {
    std::{
        future::Future,
        pin::Pin,
        sync::{Arc, Mutex},
        task::{Context, Poll, Waker},
        thread,
        time::Duration,
    },
};

pub struct TimerFuture {
    shared_state: Arc<Mutex<SharedState>>
}

struct SharedState {
    /// 是否已经达到休眠时间
    completed: bool,

    /// `TimerFuture` 表示正在运行的 `waker`.
    /// 线程可以在设置完 `completed = true` 之后来通知 `TimerFuture` 任务被唤醒并
    /// 检查 `completed = true`，然后继续执行.
    waker: Option<Waker>
}

impl Future for TimerFuture {
    type Output = ();
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 检查状态
        let mut shared_state = self.shared_state.lock().unwrap();
        if shared_state.completed {
            Poll::Ready(())
        } else {
            // 设置 `waker`， 让线程可以在定时器完成时唤醒当前 `waker`，确保
            // 再次轮询 `future` 并获知 `completed = true`.
            
            // 每次future轮询后，我们必须更新Waker，
            // 这是因为这个future可能会移动到不同的 任务去，带着不同的Waker。
            shared_state.waker = Some(cx.waker().clone());
            Poll::Pending
        }
    }
}

impl TimerFuture {
    // 大写Self是类型名，小写self是变量名
    pub fn new(duration: Duration) -> Self {
        let shared_state = Arc::new(Mutex::new(SharedState{
            completed: false,
            waker: None,
        }));

        // 创建一个新的线程
        let thread_shared_state = shared_state.clone();
        thread::spawn(move || {
            thread::sleep(duration); // 这种方式实现定时并不可取
            let mut shared_state = thread_shared_state.lock().unwrap();
            // 设置状态，表示已经完成
            shared_state.completed = true;
            // 告诉executor可以继续执行
            if let Some(waker) = shared_state.waker.take() {
                waker.wake();
            }
        });

        TimerFuture {shared_state}
    }
}