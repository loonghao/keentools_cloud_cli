use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{output::Printer, validate};

use super::Context;

#[derive(Args, Debug)]
pub struct InfoArgs {
    /// Avatar ID
    #[arg(long)]
    pub avatar_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InfoResponse {
    pub img_urls: Option<Vec<String>>,
    pub camera_positions: Vec<Option<Vec<Vec<f32>>>>,
    pub camera_projections: Vec<Option<Vec<Vec<f32>>>>,
    pub focal_length_type: String,
    pub expressions_enabled: bool,
}

pub async fn run(args: InfoArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);
    validate::avatar_id(&args.avatar_id)?;

    let resp: InfoResponse = ctx
        .client
        .get_json(&format!("/v1/avatar/{}/get-info", args.avatar_id))
        .await?;

    printer.success(&resp);
    Ok(())
}
