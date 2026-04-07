mod auth;
mod cli;
mod client;
mod commands;
mod config;
mod output;
mod schema;
mod validate;

use anyhow::Result;
use clap::Parser;
use cli::{Cli, Commands};
use output::OutputFormat;

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = run(cli).await;

    if let Err(e) = result {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    let output_format = cli.output.unwrap_or_else(|| {
        if atty::is(atty::Stream::Stdout) {
            OutputFormat::Human
        } else {
            OutputFormat::Json
        }
    });

    // Schema, Auth, and SelfUpdate don't need an API token
    match cli.command {
        Commands::Schema(args) => return schema::run(args, output_format),
        Commands::Auth(args) => return commands::auth_cmd::run(args, output_format),
        Commands::SelfUpdate(args) => return commands::self_update::run(args, output_format).await,
        _ => {}
    }

    let token = auth::resolve_token(cli.token.as_deref())?;
    let base_url = cli.api_url.ok_or_else(|| {
        anyhow::anyhow!(
            "API base URL is required.\n\
            Set the KEENTOOLS_API_URL environment variable or use --api-url <URL>.\n\
            Example: export KEENTOOLS_API_URL=https://your-api-endpoint.example.com"
        )
    })?;

    let http = client::ApiClient::new(token, base_url)?;
    let ctx = commands::Context {
        client: http,
        output: output_format,
    };

    match cli.command {
        Commands::Init(args) => commands::init::run(args, ctx).await,
        Commands::Upload(args) => commands::upload::run(args, ctx).await,
        Commands::Process(args) => commands::process::run(args, ctx).await,
        Commands::Status(args) => commands::status::run(args, ctx).await,
        Commands::Download(args) => commands::download::run(args, ctx).await,
        Commands::Info(args) => commands::info::run(args, ctx).await,
        Commands::Run(args) => commands::run_pipeline::run(args, ctx).await,
        Commands::Ephemeral(args) => commands::ephemeral::run(args, ctx).await,
        // Already handled above
        Commands::Schema(_) | Commands::Auth(_) | Commands::SelfUpdate(_) => unreachable!(),
    }
}
