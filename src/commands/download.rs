use anyhow::{bail, Context as AnyhowContext, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::PathBuf;
use std::time::Duration;

use crate::{output::Printer, validate};

use super::Context;

#[derive(Args, Debug)]
pub struct DownloadArgs {
    /// Avatar ID
    #[arg(long)]
    pub avatar_id: String,

    /// Destination file path
    #[arg(long, short = 'o')]
    pub output_path: PathBuf,

    /// Mesh format
    #[arg(long, value_enum, default_value = "glb")]
    pub format: crate::cli::MeshFormat,

    /// Blendshape groups to include (GLB only). Comma-separated: arkit,expression,nose
    #[arg(long, value_delimiter = ',')]
    pub blendshapes: Option<Vec<String>>,

    /// Texture format (GLB only)
    #[arg(long, value_enum)]
    pub texture: Option<TextureFormat>,

    /// Include wireframe edges (GLB only)
    #[arg(long)]
    pub edges: bool,

    /// Auto-poll until model is ready for download
    #[arg(long)]
    pub poll: bool,
}

#[derive(clap::ValueEnum, Clone, Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TextureFormat {
    Jpg,
    Png,
}

#[derive(Deserialize)]
#[serde(tag = "event", rename_all = "kebab-case")]
enum ModelResponse {
    RetryAfter { data: RetryData },
    Redirect { data: RedirectData },
}

#[derive(Deserialize)]
struct RetryData {
    time_sec: u64,
}

#[derive(Deserialize)]
struct RedirectData {
    url: String,
}

pub async fn run(args: DownloadArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);
    validate::avatar_id(&args.avatar_id)?;

    let format_str = match args.format {
        crate::cli::MeshFormat::Glb => "glb",
        crate::cli::MeshFormat::Obj => "obj",
    };

    // Build query params manually to handle blendshapes as comma-separated
    let mut query_parts = vec![
        format!("mesh_format={}", format_str),
        "mesh_lod=high_poly".to_string(),
    ];

    if let Some(ref bs) = args.blendshapes {
        if !bs.is_empty() {
            // API requires comma-separated, not repeated params
            query_parts.push(format!("blendshapes={}", bs.join(",")));
        }
    }

    if let Some(ref tex) = args.texture {
        query_parts.push(format!(
            "texture={}",
            match tex {
                TextureFormat::Jpg => "jpg",
                TextureFormat::Png => "png",
            }
        ));
    }

    if args.edges {
        query_parts.push("edges=true".to_string());
    }

    let query_str = query_parts.join("&");
    let path = format!("/v1/avatar/{}/get-3d-model?{}", args.avatar_id, query_str);

    let download_url = loop {
        let resp: ModelResponse = ctx.client.get_json(&path).await?;
        match resp {
            ModelResponse::Redirect { data } => break data.url,
            ModelResponse::RetryAfter { data } => {
                if !args.poll {
                    bail!(
                        "Model is still generating. Use --poll to wait, or retry in {} seconds.",
                        data.time_sec
                    );
                }
                if !printer.is_json() {
                    printer.status_line(
                        "Status",
                        &format!("Generating model, retrying in {}s...", data.time_sec),
                    );
                } else {
                    printer.success(&serde_json::json!({
                        "event": "retry-after",
                        "retry_in_seconds": data.time_sec
                    }));
                }
                tokio::time::sleep(Duration::from_secs(data.time_sec)).await;
            }
        }
    };

    // Progress bar for download
    let pb: Option<ProgressBar> = if printer.is_json() {
        None
    } else {
        let pb = ProgressBar::new(0);
        pb.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec})")
                .unwrap()
                .progress_chars("#>-"),
        );
        Some(pb)
    };

    #[allow(clippy::type_complexity)]
    let progress_cb: Option<Box<dyn Fn(u64, Option<u64>)>> = pb.as_ref().map(|p| {
        let p = p.clone();
        let cb: Box<dyn Fn(u64, Option<u64>)> = Box::new(move |downloaded, total| {
            if let Some(t) = total {
                p.set_length(t);
            }
            p.set_position(downloaded);
        });
        cb
    });

    let is_obj = matches!(args.format, crate::cli::MeshFormat::Obj);

    // Download to a temporary path first, then inspect actual content
    let tmp_dest = args.output_path.with_extension("download");

    ctx.client
        .download_to_file(&download_url, &tmp_dest, progress_cb.as_deref())
        .await?;

    if let Some(p) = pb {
        p.finish_and_clear();
    }

    if is_obj {
        // Read magic bytes to detect whether the server returned a ZIP or a bare OBJ
        let mut header = [0u8; 4];
        {
            let mut f = fs::File::open(&tmp_dest)
                .with_context(|| format!("Cannot open downloaded file: {}", tmp_dest.display()))?;
            f.read_exact(&mut header)
                .with_context(|| format!("Downloaded file too small: {}", tmp_dest.display()))?;
        }

        // ZIP magic: PK\x03\x04 (local file header) or PK\x05\x06 (end of central directory)
        let is_zip = header.starts_with(b"PK\x03\x04") || header.starts_with(b"PK\x05\x06");

        if is_zip {
            // Extract ZIP and report extracted files
            let out_dir = args
                .output_path
                .parent()
                .unwrap_or_else(|| std::path::Path::new("."));
            let zip_file = fs::File::open(&tmp_dest)
                .with_context(|| format!("Cannot open downloaded ZIP: {}", tmp_dest.display()))?;
            let mut archive =
                zip::ZipArchive::new(zip_file).with_context(|| "Failed to read ZIP archive")?;

            let mut extracted: Vec<String> = Vec::new();
            for i in 0..archive.len() {
                let mut entry = archive.by_index(i)?;
                let entry_name = entry.name().to_owned();
                let dest = out_dir.join(&entry_name);
                let mut buf = Vec::new();
                entry.read_to_end(&mut buf)?;
                fs::write(&dest, &buf)
                    .with_context(|| format!("Cannot write extracted file: {}", dest.display()))?;
                extracted.push(dest.display().to_string());
            }

            // Remove the temp download file
            let _ = fs::remove_file(&tmp_dest);

            printer.success(&serde_json::json!({
                "format": format_str,
                "extracted": extracted,
            }));
        } else {
            // Not a ZIP — treat as a bare OBJ file and rename to target path
            fs::rename(&tmp_dest, &args.output_path).with_context(|| {
                format!("Cannot save OBJ file to: {}", args.output_path.display())
            })?;
            printer.success(&serde_json::json!({
                "saved_to": args.output_path.display().to_string(),
                "format": format_str,
            }));
        }
    } else {
        // Non-OBJ formats: rename temp file to the final destination
        fs::rename(&tmp_dest, &args.output_path)
            .with_context(|| format!("Cannot save file to: {}", args.output_path.display()))?;
        printer.success(&serde_json::json!({
            "saved_to": args.output_path.display().to_string(),
            "format": format_str,
        }));
    }

    Ok(())
}
