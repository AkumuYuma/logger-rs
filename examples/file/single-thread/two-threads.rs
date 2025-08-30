use std::{path::PathBuf, thread};

use rslogger::Logger;
use log::{info, warn, error};

/// With this configuration, each thread is writing on the file separately.
/// The logging is happening respectively on Thread 1 and Thread 2 they will contend the resource 
/// of the file to write on it.This is not optimal in this case.
fn main() {
    Logger::new()
        .with_level(log::LevelFilter::Trace)
        .with_local_timestamps()
        .with_thread()
        .add_writer_file(
            PathBuf::from("./LOGS/two-threads.log"), 
            false, None)
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
