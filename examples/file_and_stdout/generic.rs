use std::{path::PathBuf, thread};

use rslogger::Logger;
use log::{info, warn, error};

fn main() {
    Logger::new()
        .with_level(log::LevelFilter::Trace)
        .with_local_timestamps()
        .with_thread()
        .add_writer_stdout(true, Some(1000))
        .add_writer_file(PathBuf::from("./LOGS/genercs_1.log"), true, Some(10))
        .add_writer_file(PathBuf::from("./LOGS/genercs_2.log"), true, Some(10000))
        .add_writer_file(PathBuf::from("./LOGS/genercs_3.log"), true, None)
        .init().unwrap();

    info!("This is a info test");
    warn!("This is a warn test");
    error!("This is an error test");

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

    // Note: This method must be called only at the end of the program. After that, you can no longer log.
    log::logger().flush();

    // This would panic
    // info!("Test");
}