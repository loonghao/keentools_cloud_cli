use anyhow::Result;
use clap::{Args, Subcommand};

use crate::{
    config,
    output::{OutputFormat, Printer},
};

#[derive(Args, Debug)]
pub struct AuthArgs {
    #[command(subcommand)]
    pub command: AuthCommand,
}

#[derive(Subcommand, Debug)]
pub enum AuthCommand {
    /// Save an API token to the config file
    Login {
        /// API token to save. If omitted, reads from KEENTOOLS_API_TOKEN env var.
        token: Option<String>,
    },
    /// Remove the stored API token
    Logout,
    /// Show current auth status and token source
    Status,
}

pub fn run(args: AuthArgs, output: OutputFormat) -> Result<()> {
    let printer = Printer::new(output);

    match args.command {
        AuthCommand::Login { token } => {
            let t = token
                .or_else(|| std::env::var("KEENTOOLS_API_TOKEN").ok())
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "No token provided. Pass it as argument or set KEENTOOLS_API_TOKEN."
                    )
                })?;

            let path = config::save_token(&t)?;
            printer.message(&format!("Token saved to {}", path.display()));
        }

        AuthCommand::Logout => {
            config::clear_token()?;
            printer.message("Token removed from config.");
        }

        AuthCommand::Status => {
            let env_token = std::env::var("KEENTOOLS_API_TOKEN").ok();
            let cfg = config::load().unwrap_or_default();

            let (source, masked) = if let Some(t) = &env_token {
                ("KEENTOOLS_API_TOKEN (env var)", mask_token(t))
            } else if let Some(t) = &cfg.auth.token {
                ("config file", mask_token(t))
            } else {
                ("none", "not configured".to_string())
            };

            if printer.is_json() {
                printer.success(&serde_json::json!({
                    "source": source,
                    "token": masked,
                    "config_path": config::config_path().map(|p| p.display().to_string()),
                }));
            } else {
                printer.status_line("Token source", source);
                printer.status_line("Token", &masked);
                if let Some(p) = config::config_path() {
                    printer.status_line("Config file", &p.display().to_string());
                }
            }
        }
    }

    Ok(())
}

fn mask_token(token: &str) -> String {
    if token.len() <= 8 {
        return "*".repeat(token.len());
    }
    format!("{}...{}", &token[..4], &token[token.len() - 4..])
}
