// Future 需要 executor 来执行
use futures::executor::block_on; // block_on 阻塞等待Future执行完成

mod simple_future;
mod socket_read;

use crate::simple_future::SimpleFuture;

// async 函数返回值是个 Future
async fn hello_world() {
    println!("hello, world!");
}

// 演示 .await
struct Song {
    str: String,
}
async fn learn_song() -> Song {
    println!("learn_song");
    Song{
        str: String::from("s: &str"),
    }
}
async fn sing_song(song: Song) {
    println!("sing - {}",song.str)
}
async fn dance() {
    println!("dance")
}

async fn learn_and_sing() {
    let song = learn_song().await; // await 让出cpu，让其他Future执行
    sing_song(song).await
}
async fn async_main() {
    let f1 = learn_and_sing();
    let f2 = dance();

    futures::join!(f1, f2); // 同时 await 多个
}

fn wake() {
    println!("wake");
}

fn main() {
    // 调用
    let future = hello_world();
    block_on(future); // 执行

    // 获取返回值
    println!("\n");
    let song = block_on(learn_song());
    block_on(sing_song(song));
    block_on(dance());

    // await
    println!("\n");
    block_on(async_main());

    let sock = socket_read::Socket;
    let mut read = socket_read::SocketRead{
        socket: &sock,
    };

    read.poll(wake);
}
