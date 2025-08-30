use rslogger::Logger;
use log::{info, warn, error};

fn main() {
    Logger::new()
        .with_level(log::LevelFilter::Trace)
        .with_local_timestamps()
        .add_writer_stdout(true, Some(1000))
        .init().unwrap();

    info!("This is a info test");
    warn!("This is a warn test");
    error!("This is an error test");

    // In this case, even if we flush, we are not able to see the log traces. 
    // Or to be more precise, we don't ALWAYS see the traces, sometimes we do.
    // This is because flush is just sending a flush command to the separate thread, 
    // but the thread was stopped before processing that command. 
    // @TODO Add some handling of this thing.
    log::logger().flush();
}