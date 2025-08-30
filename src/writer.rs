use std::{
    fs, io::{stdout, BufWriter, Write}, 
    path::PathBuf, 
    sync::{
        mpsc::{channel, Receiver, Sender
        }, RwLock}, 
    thread::{self, JoinHandle}
};

const DEFAULT_BUFFER_CAPACITY : usize = 100;

#[derive(PartialEq, Clone)]
pub enum WriteTarget {
    StdOut, 
    File
}

enum WriteMode {
    ThisThread,
    SeparateThread,
}

enum MsgType {
    Msg(String),
    Flush,
}

pub struct BufferedWriter {

    ///
    /// The target to write to
    /// 
    target: WriteTarget,

    ///
    /// Whether to write on the caller thread or on a separate thread
    /// 
    mode: WriteMode,

    ///
    /// The file path to write to.
    /// Only meaningful if writing on a file 
    /// 
    file_path: PathBuf,

    ///
    /// The capacity of the buffer.
    /// If set to 0, it will write on the target line by line. 
    /// Otherwise it will buffer and then flush.
    /// 
    buffer_capacity: usize,

    ///
    /// The BufWriters on the target Stdout.
    /// - Option because it's only initialized at init()
    /// - Box so I can use the dynamic features
    /// - RwLock because we need async interior mutability (It's needed for integration with log crate)
    /// 
    buf_writer: Option<Box<RwLock<BufWriter<dyn Write + Send + Sync>>>>,

    ///
    /// The handler of the separate thread, 
    /// only meaningful if the mode is SeparateThread.
    /// 
    thread_handler: Option<JoinHandle<()>>,

    ///
    /// The sender to send messages on the separate thread. 
    /// only meaningful if the mode is SeparateThread.
    /// 
    sender: Option<Sender<MsgType>>,
}


impl BufferedWriter {

    ///
    /// Initializes the Writer with default mode as ThisThread.
    /// 
    pub fn new() -> BufferedWriter {
        BufferedWriter { 
            target: WriteTarget::StdOut,
            mode: WriteMode::ThisThread, 
            file_path: PathBuf::default(), 
            buffer_capacity: DEFAULT_BUFFER_CAPACITY, 
            buf_writer: None,
            thread_handler: None, 
            sender: None 
        }
    }

    pub fn on_stdout(mut self) -> BufferedWriter {
        self.target = WriteTarget::StdOut;
        self
    }

    pub fn on_file(mut self, file_path: PathBuf) -> BufferedWriter {
        self.target = WriteTarget::File;
        self.file_path = file_path;
        self
    }

    /// 
    /// Sets the write mode to ThisThread (default). 
    /// With this mode, the logging operations will happen on the thread which is calling the write().
    /// 
    #[must_use = "You must call init() to initialize the writer"]
    #[allow(dead_code)]
    pub fn with_this_thread(mut self) -> BufferedWriter {
        self.mode = WriteMode::ThisThread;
        self
    }

    /// 
    /// With this mode, the logging operations will happen of a dedicated separate thread.
    /// 
    pub fn with_separate_thread(mut self) -> BufferedWriter {
        self.mode = WriteMode::SeparateThread;
        self
    }

    ///
    /// Sets the capcity of the buffer. 
    /// Not calling this function will use the default capacity. 
    /// Calling this function with capacity = 0, will flush log by log on the target 
    /// 
    pub fn with_buffer_capacity(mut self, capacity: usize) -> BufferedWriter {
        self.buffer_capacity = capacity;
        self
    }

    ///
    /// Initializes the BufferedWriter. To be necessarily called before any write. 
    /// In case of failures returns an error with the description of the error
    /// 
    pub fn init(self) -> Result<BufferedWriter, String> {
        match self.init_writers() {
            Ok(moved_self) => {
                match &moved_self.mode {
                    WriteMode::SeparateThread => return moved_self.init_separate_thread(),
                    _ => return Ok(moved_self),
                }
            },
            Err(error) => return Err(error),
        }
    }

    ///
    /// Writes the message on the target using the configured mode. 
    /// # Panics 
    /// If called before init()
    /// 
    pub fn write(&self, message: &str) {
        match &self.mode {
            WriteMode::ThisThread => BufferedWriter::write_on_this_thread(
                message, self.buf_writer.as_ref().unwrap()),
            WriteMode::SeparateThread => {
                let _ = self.sender.as_ref().unwrap().send(MsgType::Msg(message.to_string()));
            }
        }
    }

    ///
    /// Immediately flushes the buffer. 
    /// # Panics 
    /// If called before init()
    /// 
    pub fn flush(&self) {
        match &self.mode {
            WriteMode::ThisThread => BufferedWriter::flush_on_this_thread(
                self.buf_writer.as_ref().unwrap()),
            WriteMode::SeparateThread => self.sender.as_ref().unwrap().send(MsgType::Flush).unwrap_or_default(),
        }
    }


    // ------------------------------------- Private ------------------------------- //

    ///
    /// Initializes the writers depending on the target.
    /// This routine is common to Single and Multi Thread.
    /// Can panic if the writers are initialized before.
    /// 
    fn init_writers(mut self) -> Result<BufferedWriter, String> {

        // Check if data is not corrupted
        if !self.buf_writer.is_none() {
            panic!("The BufWriter should be None at this point");
        }

        match self.target {
            // Init for stdout
            WriteTarget::StdOut => {
                self.buf_writer = Some(Box::new(
                    RwLock::new(
                        BufWriter::with_capacity(self.buffer_capacity, std::io::stdout())
                    )
                ));
                Ok(self)
            }
            // Init for file
            WriteTarget::File => {
                // Create the folder if it doesn't exists
                if let Some(dir) = &self.file_path.parent() {
                    if let Err(err) = fs::create_dir_all(dir) {
                        return Err(format!("Error while creating directory for logging. Details: {}", err));
                    }
                }

                // Open the file
                match fs::OpenOptions::new()
                    .create(true).append(true)
                    .open(&self.file_path) {
                        Err(err) => {
                            Err(format!("Error while opening log file. Details: {}", err))
                        }

                        // Ok, initialize bufwriter
                        Ok(file_handler) => {
                            self.buf_writer = Some(Box::new(
                                RwLock::new(
                                    BufWriter::with_capacity(self.buffer_capacity, file_handler)
                                )
                            ));
                            Ok(self)
                        }
                    }
            }
        }
    }

    ///
    /// Initializes the separate thread for writing in SeparateThread Mode
    /// Can panic if the data structure is corrupted here
    /// 
    fn init_separate_thread(mut self) -> Result<BufferedWriter, String> {
        // Check for data structure consistency
        if ! self.thread_handler.is_none() {
            panic!("Thread handler should be None at this point");
        }

        if ! self.sender.is_none() {
            panic!("Sender should be None at this point");
        }

        if self.buf_writer.is_none() {
            panic!("BufWriter should be initialized at this point");
        }

        let (sender, receiver) : (Sender<MsgType>, Receiver<MsgType>) = channel();
        self.sender = Some(sender); 

        // Note that after the init, the bufwriter cannot be used anymore because it was moved to the other thread.
        let buf_writer_to_move: Box<RwLock<BufWriter<dyn Write + Send + Sync>>> = self.buf_writer.take().unwrap();

        match thread::Builder::new().spawn(move | | {
            while let Ok(new_message) = receiver.recv() {
                match new_message {
                    MsgType::Msg(msg) => BufferedWriter::write_on_this_thread(&msg, &*buf_writer_to_move),
                    MsgType::Flush => BufferedWriter::flush_on_this_thread(&*buf_writer_to_move),
                }
            }
        }) {
            Err(err) => return Err(format!("Unable to start Writer thread. Details {}", err)),
            Ok(handler) => self.thread_handler = Some(handler),
        }
        
        Ok(self)
    }


    ///
    /// Writes on this thread using the buf_writer passed.
    /// Used to avoid moving of self problem when initializing the separate thread.
    /// # Panics
    /// If the RWLock of the BufWriter is poisoned and cannot be taken for writing.
    /// 
    fn write_on_this_thread(message: &str, buf_writer: &RwLock<BufWriter<dyn Write + Send + Sync>>) {
        if let Ok(mut writer_mut) = buf_writer.write() {
            writer_mut.write(format!("{message}\n").as_bytes()).expect("Unable to write");
        } else {
            panic!("Cannot get writer as mutable. RWLock is poisoned!");
        }
    }

    ///
    /// Flushes the buf_writer passed on this thread.
    /// Used to avoid moving of self problems when initializing the separate thread.
    /// # Panics 
    /// If the RWLock of the BufWriter is poisoned and cannot be taken for writing.
    /// 
    fn flush_on_this_thread(buf_writer: &RwLock<BufWriter<dyn Write + Send + Sync>>) {
        if let Ok(mut writer_mut) = buf_writer.write() {
            writer_mut.flush().expect("Unable to flush");
        } else {
            panic!("Cannot get writer as mutable. RWLock is poisoned!");
        }
    }
}

mod tests {
    use super::*;


}