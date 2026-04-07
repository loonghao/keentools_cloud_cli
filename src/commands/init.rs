use anyhow::Result;
use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{output::Printer, validate};

use super::Context;

#[derive(Args, Debug)]
pub struct InitArgs {
    /// Number of photos to upload (2–15)
    #[arg(long, short = 'n')]
    pub count: usize,

    /// Validate locally without calling the API
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Serialize, Deserialize)]
struct InitRequest {
    image_count: usize,
}

#[derive(Serialize, Deserialize)]
pub struct InitResponse {
    pub avatar_id: String,
    pub img_urls: Vec<String>,
}

#[derive(Serialize, Debug)]
struct Output {
    avatar_id: String,
    upload_urls: Vec<String>,
}

pub async fn run(args: InitArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);

    validate::photo_count(args.count)?;

    if args.dry_run {
        printer.message(&format!(
            "Dry run: would initialize avatar with {} photos",
            args.count
        ));
        return Ok(());
    }

    let resp: InitResponse = ctx
        .client
        .post_json("/v1/avatar/init", &InitRequest { image_count: args.count })
        .await?;

    let out = Output {
        avatar_id: resp.avatar_id,
        upload_urls: resp.img_urls,
    };

    printer.success(&out);
    Ok(())
}
