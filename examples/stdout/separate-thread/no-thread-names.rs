use std::thread;

use rslogger::Logger;
use log::{info, warn, error};

fn main() {
    Logger::new()
        .with_level(log::LevelFilter::Trace)
        .with_local_timestamps()
        .add_writer_stdout(false, None)
        .init().unwrap();

    let handler1 = thread::Builder::new().name("Thread 1".to_string()).spawn(move | | {
        for i in 0..10 {
            info!("This is a info, i: {i}");
        }
    }).unwrap();

    let handler2 = thread::Builder::new().name("Thread 2".to_string()).spawn(move | | {
        for i in 0..10 {
            info!("This is a info, i: {i}");
        }
    }).unwrap();
    
    warn!("Warning on main!");

    handler1.join().unwrap();
    handler2.join().unwrap();

    log::logger().flush();
}