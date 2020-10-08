use std::time::Duration;
mod timer_future;
mod executor;
fn main() {
    let (executor, spawner) = executor::new_executor_and_spawner();

    spawner.spawn(async {
      println!("start!");
      // 两秒定时器
      timer_future::TimerFuture::new(Duration::new(2, 0)).await;
      println!("done!");
    });

    // 释放这个 `spawner`，以便让我们的执行程序知道它已经工作
    // 完成，并且不会接收到更多要运行的任务传入.
    drop(spawner);

    // 运行执行器，直到任务队列为空.
    // 这将输出 "start!", 等待一会, 然后输出 "done!".
    executor.run();
}
