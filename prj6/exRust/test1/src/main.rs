
// use futures::executor::block_on;

// async fn hello_world() {
//     println!("hello, world!");
// }

// fn main() {
//     let future = hello_world();
//     block_on(future);
// }

// use futures::executor::block_on;


// async fn learn_and_sing() {
//     // Wait until the song has been learned before singing it.
//     // We use `.await` here rather than `block_on` to prevent blocking the
//     // thread, which makes it possible to `dance` at the same time.
//     let song = learn_song().await;
//     sing_song(song).await;
// }
// async fn learn_song() -> String {
//     "I'm singing a song".to_string()
// }
// async fn sing_song(song: String) {
//     println!("{}", song);
// }
// async fn dance() {
//     println!("I'm dancing");
// }

// async fn async_main() {
//     let f1 = learn_and_sing();
//     let f2 = dance();

//     // `join!` is like `.await` but can wait for multiple futures concurrently.
//     // If we're temporarily blocked in the `learn_and_sing` future, the `dance`
//     // future will take over the current thread. If `dance` becomes blocked,
//     // `learn_and_sing` can take back over. If both futures are blocked, then
//     // `async_main` is blocked and will yield to the executor.
//     futures::join!(f1, f2);
// }

// fn main() {
//     block_on(async_main());
// }

// use std::mem::size_of;
// trait SomeTrait { }


// fn main() {
//     println!("======== The size of different pointers in Rust: ========");
//     println!("&dyn Trait:------{}", size_of::<&dyn SomeTrait>());
//     println!("&[&dyn Trait]:---{}", size_of::<&[&dyn SomeTrait]>());
//     println!("Box<dyn Trait>:------{}", size_of::<Box<dyn SomeTrait>>());
//     println!("Box<Box<dyn Trait>>:-{}", size_of::<Box<Box<dyn SomeTrait>>>());
//     println!("&i32:------------{}", size_of::<&i32>());
//     println!("&[i32]:----------{}", size_of::<&[i32]>());
//     println!("Box<i32>:--------{}", size_of::<Box<i32>>());
//     println!("&Box<i32>:-------{}", size_of::<&Box<i32>>());
//     println!("[&dyn Trait;4]:--{}", size_of::<[&dyn SomeTrait; 4]>());
//     println!("[i32;4]:---------{}", size_of::<[i32; 4]>());
// }
use ch4waker::main_ch4;
use ch5Genarator::main_ch5;

mod ch4waker;
mod ch5Genarator;
fn main(){
    main_ch4();
    main_ch5();
}