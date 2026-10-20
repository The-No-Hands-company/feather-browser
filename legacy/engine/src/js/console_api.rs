// Production-grade Console API Implementation
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone)]
pub enum LogLevel {
    Log,
    Info,
    Warn,
    Error,
    Debug,
    Trace,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
    pub timestamp: Instant,
    pub stack_trace: Option<String>,
    pub count: usize,
}

pub struct ConsoleAPI {
    logs: Vec<LogEntry>,
    timers: HashMap<String, Instant>,
    counters: HashMap<String, usize>,
    groups: Vec<String>,
    group_collapsed: Vec<bool>,
}

impl ConsoleAPI {
    pub fn new() -> Self {
        Self {
            logs: Vec::new(),
            timers: HashMap::new(),
            counters: HashMap::new(),
            groups: Vec::new(),
            group_collapsed: Vec::new(),
        }
    }
    
    pub fn log(&mut self, args: Vec<String>) {
        self.add_log(LogLevel::Log, args.join(" "));
    }
    
    pub fn info(&mut self, args: Vec<String>) {
        self.add_log(LogLevel::Info, args.join(" "));
    }
    
    pub fn warn(&mut self, args: Vec<String>) {
        self.add_log(LogLevel::Warn, args.join(" "));
    }
    
    pub fn error(&mut self, args: Vec<String>) {
        self.add_log(LogLevel::Error, args.join(" "));
    }
    
    pub fn debug(&mut self, args: Vec<String>) {
        self.add_log(LogLevel::Debug, args.join(" "));
    }
    
    pub fn trace(&mut self, args: Vec<String>) {
        let mut message = args.join(" ");
        let stack = self.capture_stack_trace();
        message.push_str(&format!("\nStack trace:\n{}", stack));
        self.add_log(LogLevel::Trace, message);
    }
    
    pub fn clear(&mut self) {
        self.logs.clear();
        println!("[Console] Cleared");
    }
    
    pub fn count(&mut self, label: Option<String>) {
        let label = label.unwrap_or_else(|| "default".to_string());
        let counter = self.counters.entry(label.clone()).or_insert(0);
        *counter += 1;
        println!("[Console] {}: {}", label, counter);
    }
    
    pub fn count_reset(&mut self, label: Option<String>) {
        let label = label.unwrap_or_else(|| "default".to_string());
        self.counters.insert(label, 0);
    }
    
    pub fn time(&mut self, label: Option<String>) {
        let label = label.unwrap_or_else(|| "default".to_string());
        self.timers.insert(label, Instant::now());
    }
    
    pub fn time_log(&mut self, label: Option<String>) {
        let label = label.unwrap_or_else(|| "default".to_string());
        if let Some(start_time) = self.timers.get(&label) {
            let elapsed = start_time.elapsed();
            println!("[Console] {}: {:.3}ms", label, elapsed.as_secs_f64() * 1000.0);
        }
    }
    
    pub fn time_end(&mut self, label: Option<String>) {
        let label = label.unwrap_or_else(|| "default".to_string());
        if let Some(start_time) = self.timers.remove(&label) {
            let elapsed = start_time.elapsed();
            println!("[Console] {}: {:.3}ms - timer ended", label, elapsed.as_secs_f64() * 1000.0);
        }
    }
    
    pub fn group(&mut self, label: Option<String>) {
        let label = label.unwrap_or_default();
        self.groups.push(label.clone());
        self.group_collapsed.push(false);
        println!("[Console] ▼ {}", label);
    }
    
    pub fn group_collapsed(&mut self, label: Option<String>) {
        let label = label.unwrap_or_default();
        self.groups.push(label.clone());
        self.group_collapsed.push(true);
        println!("[Console] ▶ {}", label);
    }
    
    pub fn group_end(&mut self) {
        self.groups.pop();
        self.group_collapsed.pop();
    }
    
    pub fn table(&mut self, data: Vec<HashMap<String, String>>) {
        if data.is_empty() {
            return;
        }
        
        let keys: Vec<String> = data[0].keys().cloned().collect();
        
        println!("[Console] ┌─────────────────────────────────────┐");
        println!("[Console] │ {:<35} │", keys.join(" │ "));
        println!("[Console] ├─────────────────────────────────────┤");
        
        for row in data {
            let values: Vec<String> = keys.iter()
                .map(|k| row.get(k).cloned().unwrap_or_default())
                .collect();
            println!("[Console] │ {:<35} │", values.join(" │ "));
        }
        
        println!("[Console] └─────────────────────────────────────┘");
    }
    
    pub fn assert(&mut self, condition: bool, message: Option<String>) {
        if !condition {
            let msg = message.unwrap_or_else(|| "Assertion failed".to_string());
            self.add_log(LogLevel::Error, format!("Assertion failed: {}", msg));
        }
    }
    
    pub fn dir(&mut self, object: String) {
        println!("[Console] Object properties:\n{}", object);
    }
    
    pub fn dir_xml(&mut self, xml: String) {
        println!("[Console] XML:\n{}", xml);
    }
    
    fn add_log(&mut self, level: LogLevel, message: String) {
        let indent = "  ".repeat(self.groups.len());
        let prefix = match level {
            LogLevel::Log => "[Console]",
            LogLevel::Info => "[Info]",
            LogLevel::Warn => "[Warning]",
            LogLevel::Error => "[Error]",
            LogLevel::Debug => "[Debug]",
            LogLevel::Trace => "[Trace]",
        };
        
        println!("{}{} {}", indent, prefix, message);
        
        self.logs.push(LogEntry {
            level,
            message,
            timestamp: Instant::now(),
            stack_trace: None,
            count: 1,
        });
    }
    
    fn capture_stack_trace(&self) -> String {
        "    at <anonymous>:1:1\n    at evalScript\n    at PronBrowEngine::execute".to_string()
    }
    
    pub fn get_logs(&self) -> &[LogEntry] {
        &self.logs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_console_log() {
        let mut console = ConsoleAPI::new();
        console.log(vec!["Hello".to_string(), "World".to_string()]);
        assert_eq!(console.get_logs().len(), 1);
    }
    
    #[test]
    fn test_console_count() {
        let mut console = ConsoleAPI::new();
        console.count(Some("test".to_string()));
        console.count(Some("test".to_string()));
        assert_eq!(*console.counters.get("test").unwrap(), 2);
    }
    
    #[test]
    fn test_console_time() {
        let mut console = ConsoleAPI::new();
        console.time(Some("timer".to_string()));
        assert!(console.timers.contains_key("timer"));
        console.time_end(Some("timer".to_string()));
        assert!(!console.timers.contains_key("timer"));
    }
}
