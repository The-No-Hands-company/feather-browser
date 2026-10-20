// PronBrow CLI - Resource-optimized terminal browser
// 
// Usage:
//   pronbrow-cli --mode minimal https://example.com      (text only, ~5MB RAM)
//   pronbrow-cli --mode standard https://example.com     (ANSI colors, ~15MB RAM)
//   pronbrow-cli --mode full https://example.com         (terminal graphics, ~30MB RAM)

use pronbrow_engine::*;
use std::env;
use std::process;

#[tokio::main]
async fn main() {
    env_logger::init();
    
    let args: Vec<String> = env::args().collect();
    
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }
    
    // Parse CLI arguments
    let mut mode = CLIMode::Standard;
    let mut url = None;
    
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--mode" | "-m" => {
                if i + 1 < args.len() {
                    mode = match args[i + 1].as_str() {
                        "minimal" | "min" => CLIMode::Minimal,
                        "standard" | "std" => CLIMode::Standard,
                        "full" => CLIMode::Full,
                        _ => {
                            eprintln!("Unknown mode: {}", args[i + 1]);
                            process::exit(1);
                        }
                    };
                    i += 2;
                } else {
                    eprintln!("--mode requires an argument");
                    process::exit(1);
                }
            }
            "--help" | "-h" => {
                print_usage();
                process::exit(0);
            }
            arg if arg.starts_with("http://") || arg.starts_with("https://") => {
                url = Some(arg.to_string());
                i += 1;
            }
            _ => {
                eprintln!("Unknown argument: {}", args[i]);
                process::exit(1);
            }
        }
    }
    
    let url = match url {
        Some(u) => u,
        None => {
            eprintln!("No URL provided");
            print_usage();
            process::exit(1);
        }
    };
    
    // Print banner
    print_banner(mode);
    
    // Create CLI browser
    let browser = CLIBrowser::new(mode);
    
    // Browse URL
    match browser.browse(&url).await {
        Ok(output) => {
            println!("{}", output);
        }
        Err(e) => {
            eprintln!("Error browsing {}: {}", url, e);
            process::exit(1);
        }
    }
}

fn print_banner(mode: CLIMode) {
    let mode_str = match mode {
        CLIMode::Minimal => "Minimal (Text-Only)",
        CLIMode::Standard => "Standard (ANSI)",
        CLIMode::Full => "Full (Graphics)",
    };
    
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║           PronBrow CLI - Terminal Web Browser            ║");
    println!("║                  Resource-Optimized Edition               ║");
    println!("╠═══════════════════════════════════════════════════════════╣");
    println!("║ Mode: {:<51} ║", mode_str);
    println!("╚═══════════════════════════════════════════════════════════╝");
    println!();
}

fn print_usage() {
    println!("PronBrow CLI - Resource-optimized terminal browser");
    println!();
    println!("USAGE:");
    println!("    pronbrow-cli [OPTIONS] <URL>");
    println!();
    println!("OPTIONS:");
    println!("    -m, --mode <MODE>    Browsing mode [minimal, standard, full]");
    println!("                         minimal  - Text only (~5MB RAM)");
    println!("                         standard - ANSI colors (~15MB RAM) [default]");
    println!("                         full     - Terminal graphics (~30MB RAM)");
    println!("    -h, --help           Show this help message");
    println!();
    println!("EXAMPLES:");
    println!("    pronbrow-cli https://example.com");
    println!("    pronbrow-cli --mode minimal https://news.ycombinator.com");
    println!("    pronbrow-cli --mode full https://github.com");
    println!();
    println!("ENVIRONMENT:");
    println!("    RUST_LOG=debug       Enable debug logging");
}
