use rslogger::Logger;
use log::{info, warn, error};

fn main() {
    Logger::new()
        .with_level(log::LevelFilter::Trace)
        .with_utc_timestamps()
        .add_writer_stdout(false, None)
        .init().unwrap();

    info!("This is a info test");
    warn!("This is a warn test");
    error!("This is an error test");


    log::logger().flush();
}