use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::{output::Printer, validate};

use super::Context;

#[derive(Args, Debug)]
pub struct StatusArgs {
    /// Avatar ID
    #[arg(long)]
    pub avatar_id: String,

    /// Keep polling until reconstruction completes or fails
    #[arg(long)]
    pub poll: bool,

    /// Seconds between poll requests (default: 5)
    #[arg(long, default_value = "5")]
    pub poll_interval: u64,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum StatusResponse {
    NotStarted,
    Running { data: RunningData },
    Completed,
    Failed { data: FailedData },
    Deleted,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct RunningData {
    pub progress: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FailedData {
    pub error_message: String,
}

pub async fn run(args: StatusArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);
    validate::avatar_id(&args.avatar_id)?;

    loop {
        let resp: StatusResponse = ctx
            .client
            .get_json(&format!("/v1/avatar/{}/get-status", args.avatar_id))
            .await?;

        if printer.is_json() {
            printer.success(&resp);
        } else {
            match &resp {
                StatusResponse::NotStarted => {
                    printer.status_line("Status", "not started");
                }
                StatusResponse::Running { data } => {
                    printer.status_line(
                        "Status",
                        &format!("running ({:.0}%)", data.progress * 100.0),
                    );
                }
                StatusResponse::Completed => {
                    printer.status_line("Status", "completed");
                }
                StatusResponse::Failed { data } => {
                    printer.status_line("Status", &format!("FAILED: {}", data.error_message));
                }
                StatusResponse::Deleted => {
                    printer.status_line("Status", "deleted");
                }
            }
        }

        if !args.poll {
            break;
        }

        match &resp {
            StatusResponse::Completed | StatusResponse::Failed { .. } | StatusResponse::Deleted => {
                break;
            }
            _ => {
                tokio::time::sleep(Duration::from_secs(args.poll_interval)).await;
            }
        }
    }

    Ok(())
}
