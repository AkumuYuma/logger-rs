mod writer;
use std::{path::PathBuf, sync::RwLock};

use crate::writer::BufferedWriter;

use log::{Log, SetLoggerError, LevelFilter};
use time::{format_description::FormatItem, OffsetDateTime, UtcDateTime};

const TIMESTMAMP_FORMAT: &[FormatItem] = time::macros::format_description!(
    "[hour]:[minute]:[second]:[subsecond digits:6]"
);


#[derive(PartialEq)]
enum Timestamps {
    None, 
    Local,
    Utc,
}

pub struct Logger {
    /// The default log level for all the logs.
    log_level: LevelFilter,
    timestamps: Timestamps,
    thread: bool,
    target: bool,
    ///
    /// The RwLock is needed to provide interior mutability. 
    /// That bitch of the Log crate decided to declare flush method as flush(&self) and not 
    /// flush(&mut self) and there is no way to call a method of the Logger struct that is not 
    /// in the Log Trait (because when you set the boxed logger, they convert it into &'static).
    /// So in order to ensure that the multi threaded BufferedWriter can flush and stop the thread
    /// we need a mutable reference to it inside the flush method.
    /// Also, it is an RwLock and not an Rc because this structure must be Sync + Send.
    writers: Vec<RwLock<BufferedWriter>>,
}

impl Logger {

    /// Initializes the global logger with an RLogger instance with
    /// default log level set to `Level::Trace`.
    ///
    /// ```no_run
    /// use rslogger::Logger;
    /// Logger::new().init().unwrap();
    /// log::warn!("This is an example message.");
    /// ```
    ///
    /// [`init`]: #method.init
    #[must_use = "You must call init() to initialize the logger"]
    pub fn new() -> Logger {
        Logger { 
            log_level: LevelFilter::Trace , 
            timestamps: Timestamps::Local, 
            target: false,
            thread: false, 
            writers: Vec::new() }
    }

    /// Sets the global log level of the logger. 
    #[must_use = "You must call init() to initialize the logger"]
    pub fn with_level(mut self, level: LevelFilter) -> Logger {
        self.log_level = level;
        self
    }

    /// Display timestamps in UTC time
    #[must_use = "You must call init() to initialize the logger"]
    pub fn with_utc_timestamps(mut self) -> Logger {
        self.timestamps = Timestamps::Utc;
        self
    }

    /// Display timestamps in Local time
    #[must_use = "You must call init() to initialize the logger"]
    pub fn with_local_timestamps(mut self) -> Logger {
        self.timestamps = Timestamps::Local;
        self
    }

    /// Don't display timestamps
    #[must_use = "You must call init() to initialize the logger"]
    pub fn without_timestamps(mut self) -> Logger {
        self.timestamps = Timestamps::None;
        self
    }

    /// Display thread Id
    #[must_use = "You must call init() to initialize the logger"]
    pub fn with_thread(mut self) -> Logger {
        self.thread = true;
        self
    }

    /// Displays the module name that is logging
    #[must_use = "You must call init() to initialize the logger"]
    pub fn with_target(mut self) -> Logger {
        self.target = true;
        self
    }

    /// Hides the module name that is logging
    #[must_use = "You must call init() to initialize the logger"]
    pub fn without_target(mut self) -> Logger {
        self.target = false;
        self
    }

    ///
    /// Adds a stdout writer. 
    /// # Param
    /// * `multi_thread` - If set to true, the writer will be multi thread, otherwise single thread
    /// * `capacity` - If Some(capacity), specified the buffer capacity of the writer. If None, initializes it with the default capacity.
    /// 
    #[must_use = "You must call init() to initialize the logger"]
    pub fn add_writer_stdout(mut self, multi_thread: bool, capacity: Option<usize>) -> Logger {
        let mut writer = BufferedWriter::new().on_stdout();

        if multi_thread { writer = writer.with_separate_thread(); }
        if let Some(buf_cap) = capacity { writer = writer.with_buffer_capacity(buf_cap) }

        match writer.init() {
            Ok(initialized_writer) => self.writers.push(RwLock::new(initialized_writer)),
            Err(error) => println!("Error while initializing writer. Details: {}", error),
        }

        self
    }

    ///
    /// Adds a file writer. 
    /// # Param
    /// * `file_path` - The path of the file to write on.
    /// * `multi_thread` - If set to true, the writer will be multi thread, otherwise single thread
    /// * `capacity` - If Some(capacity), specified the buffer capacity of the writer. If None, initializes it with the default capacity.
    /// 
    #[must_use = "You must call init() to initialize the logger"]
    pub fn add_writer_file(mut self, file_path: PathBuf, multi_thread: bool, capacity: Option<usize>) -> Logger {
        let mut writer = BufferedWriter::new().on_file(file_path);
        
        if multi_thread { writer = writer.with_separate_thread(); }
        if let Some(buf_cap) = capacity { writer = writer.with_buffer_capacity(buf_cap) }

        match writer.init() {
            Ok(initialized_writer) => self.writers.push(RwLock::new(initialized_writer)),
            Err(error) => println!("Error while initializing writer. Details: {}", error),
        }

        self
    }

    pub fn init(self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.log_level);
        log::set_boxed_logger(Box::new(self))
    }

    pub fn log_level(&self) -> LevelFilter {
        self.log_level
    }
}

impl Default for Logger {
    fn default() -> Self {
        Logger::new()
    }
}

impl Log for Logger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level().to_level_filter() <= self.log_level
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let mut target = "";

        if self.target {
            target = if !record.target().is_empty() {
                    record.target()
                } else {
                    record.module_path().unwrap_or_default()
                };
        }


        let thread = if self.thread {
            if let Some(thread_name) = std::thread::current().name() {
                format!("{}", thread_name)
            } else {
                format!("{:?}", std::thread::current().id())
            }
        } else {
            "".to_string()
        };
        
        let timestamp = match self.timestamps {
            Timestamps::None => "".to_string(),
            Timestamps::Local => format!(
                "{}",
                OffsetDateTime::now_local()
                    .expect(concat!(
                        "Could not determine the UTC offset on this system. ",
                        "Consider displaying UTC time instead. ",
                        "Possible causes are that the time crate does not implement \"local_offset_at\" ",
                        "on your system, or that you are running in a multi-threaded environment and ",
                        "the time crate is returning \"None\" from \"local_offset_at\" to avoid unsafe ",
                        "behaviour. See the time crate's documentation for more information. ",
                        "(https://time-rs.github.io/internal-api/time/index.html#feature-flags)"
                    ))
                    .format(TIMESTMAMP_FORMAT)
                    .unwrap()
            ),
            Timestamps::Utc => format!( "{}", UtcDateTime::now().format(TIMESTMAMP_FORMAT).unwrap()),
        };

        let message = format!("{timestamp}-[{target}][{thread}] -> {{{}}} {}", record.level().to_string(), record.args());

        for writer in &self.writers {
            if let Ok(writer_mut) = writer.write() {
                writer_mut.write(message.as_str());
            } else {
                panic!("Cannot get writer as mutable. RWLock is poisoned!");
            }
        }
    }

    ///
    /// Flushes to ensure that all possible buffered data are logged. 
    /// This method should be called right before closing the program or when you don't need to 
    /// log anything else. 
    /// This is because in case of separate thread, the logger thread will be stopped.
    /// # Example
    /// ```
    /// use rslogger::Logger;
    /// use log::{info, warn, error};
    /// Logger::new()
    ///     .with_level(log::LevelFilter::Trace)
    ///     .with_local_timestamps()
    ///     .add_writer_stdout(true, Some(1000))
    ///     .init().unwrap();
    /// info!("This is a info test");
    /// warn!("This is a warn test");
    /// error!("This is an error test");
    /// log::logger().flush();
    /// let result = std::panic::catch_unwind(|| info!("Test"));
    /// assert!(result.is_err())
    /// ```
    /// 
    fn flush(&self) {
        for writer in &self.writers {
            if let Ok(mut writer_mut) = writer.write() {
                writer_mut.flush_and_cleanup();
            } else {
                panic!("Cannot get writer as mutable. RWLock is poisoned!");
            }
        }
    }

}

#[cfg(test)]
mod tests {
    use log::{Metadata, Level};

    use super::*;

    #[test]
    fn test_default_level() {
        let builder = Logger::new();
        assert_eq!(builder.log_level(), LevelFilter::Trace);
    }

    #[test]
    fn test_creation_level() {
        let builder = Logger::new().with_level(LevelFilter::Debug);
        assert_eq!(builder.log_level(), LevelFilter::Debug);
    }

    #[test]
    fn test_logger_enabled() {
        let logger = Logger::new().with_level(LevelFilter::Debug);
        assert_eq!(logger.log_level(), LevelFilter::Debug);
        assert!(logger.enabled(&create_log("test_enabled", Level::Debug)));
    }

    #[test]
    fn test_timestamp_default() {
        let builder = Logger::new();
        assert!(builder.timestamps == Timestamps::Local);
    }

    #[test]
    fn test_utc_timestamp() {
        let builder = Logger::new().with_utc_timestamps();
        assert!(builder.timestamps == Timestamps::Utc);
    }


    fn create_log(name: &str, level: Level) -> Metadata {
        let mut builder = Metadata::builder();
        builder.level(level);
        builder.target(name);
        builder.build()
    }
}