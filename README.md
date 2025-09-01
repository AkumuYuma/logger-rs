# flex-logger
This is a simple logger implementing the log crate interface. 

- It is simple to initialize and use, and supports logging directly on the caller thread as well as on a separate thread. 
- It supports logging with different timestamps format (utc/local)
- It supports tracing the thread id of the called as well.
- It supports logging on different targets (stdout/file) and the logging is buffered with a custom buffer size (which should make it faster to avoid a lot of locks on the resource in use.) 

## Note
The `flush` method of the log crate interface was implemented here as cleanup. 
In other words, this call ensures that all the buffered logs are immediately flushed but destroys the logger. 
**Ensure to call this method only at the end of your program**

## Example
```rust
use std::{path::PathBuf, thread};

use rslogger::Logger;
use log::{info, warn, error};

fn main() {
    Logger::new()
        // This is setting the level max to Trace,
        .with_level(log::LevelFilter::Trace)
        // This is setting the timestamps as local
        .with_local_timestamps()
        // This is configuring the logger to log the thread id (if available)
        .with_thread()
        // Add a writer on the stdout. It will use a dedicated thread to log and a buffer capacity of 1000 bytes
        .add_writer_stdout(true, Some(1000))
        // Add a writer on the file. If the file does not exists, it will be created. It will use a dedicated thread to log and a buffer capacity of 10 bytes
        .add_writer_file(PathBuf::from("./LOGS/genercs_1.log"), true, Some(10))
        // Another writer on another file. For this, no buffering will happen
        .add_writer_file(PathBuf::from("./LOGS/genercs_2.log"), true, Some(0))
        // Another writer with default buffer size (100 bytes)
        .add_writer_file(PathBuf::from("./LOGS/genercs_3.log"), true, None)
        // Always call init to allow logging
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
```
And here is the output of the program: 
```
09:34:31:222312-[][main] -> {INFO} This is a info test
09:34:31:222400-[][main] -> {WARN} This is a warn test
09:34:31:222432-[][main] -> {ERROR} This is an error test
09:34:31:222551-[][main] -> {WARN} Warning on main!
09:34:31:222570-[][Thread 1] -> {INFO} This is a info, i: 0
09:34:31:222622-[][Thread 1] -> {INFO} This is a info, i: 1
09:34:31:222616-[][Thread 2] -> {INFO} This is a info, i: 0
09:34:31:222657-[][Thread 1] -> {INFO} This is a info, i: 2
09:34:31:222666-[][Thread 2] -> {INFO} This is a info, i: 1
09:34:31:222685-[][Thread 1] -> {INFO} This is a info, i: 3
09:34:31:222693-[][Thread 2] -> {INFO} This is a info, i: 2
09:34:31:222712-[][Thread 1] -> {INFO} This is a info, i: 4
09:34:31:222714-[][Thread 2] -> {INFO} This is a info, i: 3
09:34:31:222738-[][Thread 1] -> {INFO} This is a info, i: 5
09:34:31:222740-[][Thread 2] -> {INFO} This is a info, i: 4
09:34:31:222769-[][Thread 2] -> {INFO} This is a info, i: 5
09:34:31:222768-[][Thread 1] -> {INFO} This is a info, i: 6
09:34:31:222801-[][Thread 2] -> {INFO} This is a info, i: 6
09:34:31:222802-[][Thread 1] -> {INFO} This is a info, i: 7
09:34:31:222851-[][Thread 1] -> {INFO} This is a info, i: 8
09:34:31:222860-[][Thread 2] -> {INFO} This is a info, i: 7
09:34:31:222883-[][Thread 1] -> {INFO} This is a info, i: 9
09:34:31:222885-[][Thread 2] -> {INFO} This is a info, i: 8
09:34:31:222911-[][Thread 2] -> {INFO} This is a info, i: 9
```
In the [examples](./examples/) directory you will find more examples.