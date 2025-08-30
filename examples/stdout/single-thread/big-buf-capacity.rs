use rslogger::Logger;
use log::{info, warn, error};

fn main() {
    Logger::new()
        .with_level(log::LevelFilter::Trace)
        .with_local_timestamps()
        .add_writer_stdout(false, Some(1000))
        .init().unwrap();

    info!("This is a info test");
    warn!("This is a warn test");
    error!("This is an error test");

    // In this case, if we remove this line, we would not see the traces
    // You can try it.
    // This is because the flush method is not called automatically
    log::logger().flush();
}