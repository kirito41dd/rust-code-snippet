/// 伪代码，大概了解 poll
use crate::simple_future::*;

pub struct Socket;
impl Socket {
    fn has_data_to_read(&self) -> bool {
        return false;
    }
    fn read_buf(&self) -> Vec<u8> {
        vec!(1)
    }
    fn set_readable_callback(&self,f: fn()) {
        f();
    }
}

// 模拟一个future
pub struct SocketRead<'a> {
    pub socket: &'a Socket,
}

impl SimpleFuture for SocketRead<'_> {
    type Output = Vec<u8>;

    fn poll(&mut self, wake: fn()) -> Poll<Self::Output> {
        if self.socket.has_data_to_read() {
            // 执行完成
            Poll::Ready(self.socket.read_buf())
        } else {
            // 未执行完成
            self.socket.set_readable_callback(wake);
            Poll::Pending
        }
    }
}