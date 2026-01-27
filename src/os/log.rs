use std::{
    fs::{File, OpenOptions},
    io::{BufWriter, Write as _},
};

use log::{Level, Log};
use parking_lot::Mutex;

pub struct FileLogger {
    writer: Mutex<BufWriter<File>>,
    level: Level,
}

impl FileLogger {
    pub fn new(file_name: &str, level: Level) -> std::io::Result<Self> {
        let mut path = std::env::current_exe()?;
        path.pop();
        path.push(file_name);
        let file = OpenOptions::new().create(true).append(true).open(path)?;

        Ok(Self {
            writer: Mutex::new(BufWriter::new(file)),
            level,
        })
    }

    pub fn init(self) {
        let max_level = self.level.to_level_filter();
        log::set_boxed_logger(Box::new(self)).unwrap();
        log::set_max_level(max_level);
    }

    pub fn write_log(&self, record: &log::Record) {
        let message = format!("[{}] {}\n", record.level(), record.args());
        let mut writer = self.writer.lock();
        let _ = writer.write_all(message.as_bytes());
        let _ = writer.flush();
    }
}

impl Log for FileLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        metadata.level() <= self.level
    }

    fn log(&self, record: &log::Record) {
        if self.enabled(record.metadata()) {
            self.write_log(record);
        }
    }

    fn flush(&self) {
        let _ = self.writer.lock().flush();
    }
}
