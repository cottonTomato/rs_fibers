use fibers::{yield_thread, Runtime};

fn main() {
    let mut rt = Runtime::new();
    rt.init();

    rt.spawn(|| {
        println!("Thread 1: Starting");
        let id = 1;
        for i in 0..10 {
            println!("thread: {id}, counter: {i}");
            yield_thread();
        }
        println!("Thread 1: Finished");
    });

    rt.spawn(|| {
        println!("Thread 2: Starting");
        let id = 2;
        for i in 0..15 {
            println!("thread: {id}, counter: {i}");
            yield_thread();
        }
        println!("Thread 2: Finished");
    });

    rt.run();
}
