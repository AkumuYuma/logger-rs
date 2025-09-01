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

    // Note: This method must be called only at the end of the program. After that, you can no longer log.
    log::logger().flush();

    // This would panic
    // info!("Test");
}