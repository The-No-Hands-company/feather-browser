// Console API implementation
use std::fmt::Display;

pub struct Console;

impl Console {
    pub fn log<T: Display>(message: T) {
        println!("[LOG] {}", message);
    }
    
    pub fn warn<T: Display>(message: T) {
        println!("[WARN] {}", message);
    }
    
    pub fn error<T: Display>(message: T) {
        eprintln!("[ERROR] {}", message);
    }
    
    pub fn debug<T: Display>(message: T) {
        println!("[DEBUG] {}", message);
    }
    
    pub fn info<T: Display>(message: T) {
        println!("[INFO] {}", message);
    }
    
    pub fn clear() {
        print!("\x1B[2J\x1B[1;1H");
    }
    
    pub fn time(label: &str) {
        println!("[TIME] {} started", label);
    }
    
    pub fn time_end(label: &str) {
        println!("[TIME] {} ended", label);
    }
}
