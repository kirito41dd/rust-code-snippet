/// 简单的 Future
/// Future能通过调用poll的方式推进，这会尽可能地推进future到完成状态。
/// 如果future完成了， 那就会返回poll::Ready(result)。
/// 如果future尚未完成，则返回poll::Pending，
/// 并且注册回调函数 wake()，在Future准备好进一步执行时调用
/// 当wake()调用时，驱动Future的执行器会再次poll使得Future有所进展。
/// 没有wake()的话，执行器将无从获知一个future是否能有所进展，
/// 并且会持续轮询（polling） 所有future。
/// 但有了wake()函数，执行器就能知道哪些future已经准备好轮询了。
pub trait SimpleFuture {
    type Output;
    fn poll(&mut self, wake: fn()) -> Poll<Self::Output>;
}

pub enum Poll<T> {
    Ready(T),
    Pending,
}