/*
logger.rs - A simple logging utility for EpiKodi
*/


/* 
    let logger = Logger::new(LOG_FILE);

    // Log some messages
    logger.info("Application started");
    logger.debug("This is a debug message");
    logger.warning("This is a warning message");
    logger.error("This is an error message");
*/

use std::fs::OpenOptions;
use std::io::Write;
use chrono::Local;

use crate::constants::constants::{DEBUG, LOG_IN_CONSOLE};

// Define the different log levels
#[derive(Debug, Clone, Copy)]
pub enum LogLevel {
    Info,
    Warning,
    Error,
    Debug,
}

impl LogLevel {
    // Convert log level to a string for display
    fn as_str(&self) -> &str {
        match self {
            LogLevel::Info => "INFO",
            LogLevel::Warning => "WARNING",
            LogLevel::Error => "ERROR",
            LogLevel::Debug => "DEBUG",
        }
    }
}

// The Logger struct
pub struct Logger {
    log_file: String,
}

impl Logger {
    // Create a new logger instance
    pub fn new(log_file: &str) -> Self {
        Logger {
            log_file: log_file.to_string(),
        }
    }

    // Main logging method
    pub fn log(&self, level: LogLevel, message: &str) {
        // Get current timestamp
        let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
        
        // Format the log entry
        let log_entry = format!("[{}] [{}] {}\n", timestamp, level.as_str(), message);
        
        // Print to console
        if LOG_IN_CONSOLE {print!("{}", log_entry);}
        
        // Write to file
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_file)
            .expect("Failed to open log file");
        
        file.write_all(log_entry.as_bytes())
            .expect("Failed to write to log file");
    }

    // Convenience methods for each log level
    pub fn info(&self, message: &str) {
        self.log(LogLevel::Info, message);
    }

    pub fn warning(&self, message: &str) {
        self.log(LogLevel::Warning, message);
    }

    pub fn error(&self, message: &str) {
        self.log(LogLevel::Error, message);
    }

    pub fn debug(&self, message: &str) {
        if DEBUG {
            self.log(LogLevel::Debug, message);
        }
    }
}
