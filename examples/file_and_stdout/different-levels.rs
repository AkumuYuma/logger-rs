use std::{path::PathBuf, thread};

use rslogger::Logger;
use log::{info, warn, error, trace};

fn main() {
    Logger::new()
        .with_level(log::LevelFilter::Trace)
        .with_local_timestamps()
        .with_thread()
        .add_writer_stdout(true, Some(1000))
        // This will trace only error or less
        .add_writer_file_with_level(PathBuf::from("./LOGS/genercs_1.log"), true, Some(10), log::LevelFilter::Error)
        // This will trace only info or less
        .add_writer_file_with_level(PathBuf::from("./LOGS/genercs_2.log"), true, Some(10000), log::LevelFilter::Info)
        // This will trace only trace or less (default)
        .add_writer_file(PathBuf::from("./LOGS/genercs_3.log"), true, None)
        .add_writer_stdout(false, None)
        .init().unwrap();

    let handler1 = thread::Builder::new().spawn(move | | {
        for i in 0..10 {
            info!("This is a info, i: {i}");
        }
    }).unwrap();

    let handler2 = thread::Builder::new().spawn(move | | {
        for i in 0..10 {
            info!("This is a info, i: {i}");
        }
    }).unwrap();
    
    warn!("Warning on main!");
    error!("This is traced on genercs_1.log");
    info!("This is traced on genercs_2.log but not on genercs_1.log");
    trace!("This is traced on genercs_3.log but not on genercs_1/2.log");

    handler1.join().unwrap();
    handler2.join().unwrap();

    log::logger().flush();
}
