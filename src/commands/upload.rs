use anyhow::{bail, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use std::path::PathBuf;

use crate::{output::Printer, validate};

use super::Context;

#[derive(Args, Debug)]
pub struct UploadArgs {
    /// Avatar ID from `init`
    #[arg(long)]
    pub avatar_id: String,

    /// Pre-signed upload URLs (comma-separated). If omitted, reads from init output via --upload-urls.
    #[arg(long, value_delimiter = ',')]
    pub urls: Option<Vec<String>>,

    /// Photo file paths to upload (matched by position to upload URLs)
    #[arg(required = true)]
    pub photos: Vec<PathBuf>,
}

pub async fn run(args: UploadArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);

    validate::avatar_id(&args.avatar_id)?;

    for photo in &args.photos {
        validate::photo_path(photo)?;
    }

    let urls = match args.urls {
        Some(u) => u,
        None => {
            bail!(
                "No upload URLs provided. Pass --urls or pipe from `init --output json`.\n\
                Example:\n  keentools-cloud init -n 3 --output json | \\\n    \
                keentools-cloud upload --avatar-id <ID> --urls <urls> photos/*.jpg"
            );
        }
    };

    if urls.len() != args.photos.len() {
        bail!(
            "Mismatch: {} upload URLs but {} photo(s). They must be equal.",
            urls.len(),
            args.photos.len()
        );
    }

    for url in &urls {
        validate::https_url(url)?;
    }

    let pb = if printer.is_json() {
        None
    } else {
        let pb = ProgressBar::new(args.photos.len() as u64);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    };

    for (photo, url) in args.photos.iter().zip(urls.iter()) {
        if let Some(ref p) = pb {
            p.set_message(format!("Uploading {}", photo.display()));
        }
        ctx.client.put_file(url, photo).await?;
        if printer.is_json() {
            printer.success(&serde_json::json!({
                "uploaded": photo.display().to_string(),
            }));
        }
        if let Some(ref p) = pb {
            p.inc(1);
        }
    }

    if let Some(p) = pb {
        p.finish_with_message("All photos uploaded");
    }

    if !printer.is_json() {
        printer.message(&format!(
            "Uploaded {} photo(s) for avatar {}",
            args.photos.len(),
            args.avatar_id
        ));
    }

    Ok(())
}
