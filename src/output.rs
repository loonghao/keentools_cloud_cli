use clap::ValueEnum;
use colored::Colorize;
use serde::Serialize;

#[derive(Clone, Debug, ValueEnum, PartialEq, Copy)]
pub enum OutputFormat {
    /// Colorized, human-readable output (default when stdout is a TTY)
    Human,
    /// Machine-readable JSON (default when stdout is not a TTY)
    Json,
}

pub struct Printer {
    pub format: OutputFormat,
}

impl Printer {
    pub fn new(format: OutputFormat) -> Self {
        Self { format }
    }

    /// Print a success value. In JSON mode emits a JSON object; in human mode pretty-prints.
    pub fn success<T: Serialize + std::fmt::Debug>(&self, value: &T) {
        match self.format {
            OutputFormat::Json => {
                println!("{}", serde_json::to_string(value).unwrap_or_default());
            }
            OutputFormat::Human => {
                // Fallback: pretty JSON for structured data
                println!(
                    "{}",
                    serde_json::to_string_pretty(value).unwrap_or_default()
                );
            }
        }
    }

    /// Print a simple message (human) or JSON object with "message" key (JSON).
    pub fn message(&self, msg: &str) {
        match self.format {
            OutputFormat::Json => {
                println!("{}", serde_json::json!({ "message": msg }));
            }
            OutputFormat::Human => {
                println!("{} {}", "✓".green().bold(), msg);
            }
        }
    }

    /// Print an error to stderr.
    pub fn error(&self, msg: &str, code: &str) {
        match self.format {
            OutputFormat::Json => {
                eprintln!("{}", serde_json::json!({ "error": msg, "code": code }));
            }
            OutputFormat::Human => {
                eprintln!("{} [{}] {}", "✗".red().bold(), code.yellow(), msg);
            }
        }
    }

    /// Print a status line (human only; JSON mode prints the full status JSON via success()).
    pub fn status_line(&self, label: &str, value: &str) {
        if self.format == OutputFormat::Human {
            println!("{}: {}", label.cyan().bold(), value);
        }
    }

    pub fn is_json(&self) -> bool {
        self.format == OutputFormat::Json
    }
}
