use anyhow::{bail, Result};
use clap::Args;
use std::path::PathBuf;
use std::time::Duration;

use crate::{
    cli::{FocalLengthType, MeshFormat},
    output::{emit_ipc, Printer},
    validate,
};

use super::{download::DownloadArgs, Context};

/// Full pipeline shortcut: init → upload → process → wait → download
#[derive(Args, Debug)]
pub struct RunArgs {
    /// Photo files to reconstruct (2–15)
    #[arg(required = true)]
    pub photos: Vec<PathBuf>,

    /// Where to save the resulting 3D model
    #[arg(long, short = 'o')]
    pub output_path: PathBuf,

    /// Focal length handling mode
    #[arg(long, value_enum, default_value = "estimate-per-image")]
    pub focal_length_type: FocalLengthType,

    /// Comma-separated 35mm focal lengths (required for manual mode)
    #[arg(long, value_delimiter = ',')]
    pub focal_lengths: Option<Vec<f32>>,

    /// Enable facial expression blendshapes
    #[arg(long)]
    pub expressions: bool,

    /// Output mesh format
    #[arg(long, value_enum, default_value = "glb")]
    pub format: MeshFormat,

    /// Blendshape groups (GLB only, comma-separated: arkit,expression,nose)
    #[arg(long, value_delimiter = ',')]
    pub blendshapes: Option<Vec<String>>,

    /// Texture format (GLB only)
    #[arg(long, value_enum)]
    pub texture: Option<super::download::TextureFormat>,

    /// Include wireframe edges (GLB only)
    #[arg(long)]
    pub edges: bool,

    /// Validate locally without calling the API
    #[arg(long)]
    pub dry_run: bool,

    /// Seconds between status polls (default: 5)
    #[arg(long, default_value = "5")]
    pub poll_interval: u64,

    /// Emit IPC progress events as NDJSON to stdout (for Qt/web frontends)
    #[arg(long)]
    pub ipc: bool,
}

pub async fn run(args: RunArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);

    validate::photo_count(args.photos.len())?;
    for p in &args.photos {
        validate::photo_path(p)?;
    }

    if args.dry_run {
        printer.message(&format!(
            "Dry run: would run full pipeline with {} photo(s) → {}",
            args.photos.len(),
            args.output_path.display()
        ));
        return Ok(());
    }

    // ── Step 1: init ──────────────────────────────────────────────────────────
    if args.ipc {
        emit_ipc(&serde_json::json!({
            "type": "progress",
            "stage": "init",
            "percent": 0,
            "message": "Initializing session"
        }));
    }
    if !printer.is_json() {
        printer.status_line("Step 1/5", "Initializing avatar session...");
    }

    let init_resp: super::init::InitResponse = ctx
        .client
        .post_json(
            "/v1/avatar/init",
            &serde_json::json!({ "image_count": args.photos.len() }),
        )
        .await?;

    let avatar_id = init_resp.avatar_id.clone();
    let upload_urls = init_resp.img_urls;

    if printer.is_json() {
        printer.success(&serde_json::json!({
            "step": "init",
            "avatar_id": &avatar_id,
        }));
    } else {
        printer.status_line("Avatar ID", &avatar_id);
    }

    // ── Step 2: upload ────────────────────────────────────────────────────────
    if !printer.is_json() {
        printer.status_line("Step 2/5", "Uploading photos...");
    }

    let photo_count = args.photos.len();
    for (i, (photo, url)) in args.photos.iter().zip(upload_urls.iter()).enumerate() {
        if args.ipc {
            // Upload occupies percent range 10–30
            let pct = 10 + (i * 20 / photo_count.max(1));
            emit_ipc(&serde_json::json!({
                "type": "progress",
                "stage": "upload",
                "percent": pct,
                "message": format!("Uploading photo {}/{}", i + 1, photo_count)
            }));
        }
        validate::https_url(url)?;
        ctx.client.put_file(url, photo).await?;
        if printer.is_json() {
            printer.success(&serde_json::json!({
                "step": "upload",
                "file": photo.display().to_string(),
            }));
        }
    }

    // ── Step 3: process ───────────────────────────────────────────────────────
    if args.ipc {
        emit_ipc(&serde_json::json!({
            "type": "progress",
            "stage": "process",
            "percent": 40,
            "message": "Starting reconstruction"
        }));
    }
    if !printer.is_json() {
        printer.status_line("Step 3/5", "Starting reconstruction...");
    }

    let focal_payload = build_focal_payload(&args.focal_length_type, &args.focal_lengths)?;

    ctx.client
        .post_json::<_, serde_json::Value>(
            &format!("/v1/avatar/{}/process", avatar_id),
            &focal_payload,
        )
        .await
        .or_else(|e| {
            let msg = e.to_string();
            if msg.contains("EOF") || msg.contains("parse") {
                Ok(serde_json::Value::Null)
            } else {
                Err(e)
            }
        })?;

    // ── Step 4: poll status ───────────────────────────────────────────────────
    if !printer.is_json() {
        printer.status_line("Step 4/5", "Waiting for reconstruction to complete...");
    }

    loop {
        let status: super::status::StatusResponse = ctx
            .client
            .get_json(&format!("/v1/avatar/{}/get-status", avatar_id))
            .await?;

        match &status {
            super::status::StatusResponse::Completed => {
                if args.ipc {
                    emit_ipc(&serde_json::json!({
                        "type": "progress",
                        "stage": "reconstruct",
                        "percent": 85,
                        "message": "Reconstruction complete"
                    }));
                }
                if printer.is_json() {
                    printer
                        .success(&serde_json::json!({ "step": "status", "status": "completed" }));
                } else {
                    printer.message("Reconstruction completed.");
                }
                break;
            }
            super::status::StatusResponse::Failed { data } => {
                if args.ipc {
                    emit_ipc(&serde_json::json!({
                        "type": "error",
                        "stage": "process",
                        "message": format!("Reconstruction failed: {}", data.error_message)
                    }));
                }
                bail!("Reconstruction failed: {}", data.error_message);
            }
            super::status::StatusResponse::Running { data } => {
                if args.ipc {
                    let pct = 45 + (data.progress * 40.0) as u64; // 45–85
                    emit_ipc(&serde_json::json!({
                        "type": "progress",
                        "stage": "reconstruct",
                        "percent": pct,
                        "message": format!("Reconstructing... {:.0}%", data.progress * 100.0)
                    }));
                }
                if printer.is_json() {
                    printer.success(&serde_json::json!({
                        "step": "status",
                        "status": "running",
                        "progress": data.progress,
                    }));
                } else {
                    printer.status_line("Progress", &format!("{:.0}%", data.progress * 100.0));
                }
            }
            _ => {}
        }

        tokio::time::sleep(Duration::from_secs(args.poll_interval)).await;
    }

    // ── Step 5: download ──────────────────────────────────────────────────────
    if args.ipc {
        emit_ipc(&serde_json::json!({
            "type": "progress",
            "stage": "download",
            "percent": 90,
            "message": "Downloading model"
        }));
    }
    if !printer.is_json() {
        printer.status_line("Step 5/5", "Downloading 3D model...");
    }

    let dl_result = super::download::run(
        DownloadArgs {
            avatar_id: avatar_id.clone(),
            output_path: args.output_path.clone(),
            format: args.format,
            blendshapes: args.blendshapes,
            texture: args.texture,
            edges: args.edges,
            poll: true,
        },
        Context {
            client: ctx.client,
            output: printer.format,
        },
    )
    .await?;

    if args.ipc {
        // Use the actual primary path from DownloadResult, not args.output_path.
        // When the server returns a ZIP (OBJ format), the extracted files have
        // different names (e.g., neutral.obj) than the original output_path.
        emit_ipc(&serde_json::json!({
            "type": "complete",
            "stage": "done",
            "percent": 100,
            "saved_to": dl_result.primary_path.display().to_string()
        }));
    }

    Ok(())
}

fn build_focal_payload(
    fl_type: &FocalLengthType,
    fl_values: &Option<Vec<f32>>,
) -> Result<serde_json::Value> {
    // API expects: {"focal_length_type": {"focal_length_type": "...", ...}, "expressions_enabled": false}
    let inner = match fl_type {
        FocalLengthType::Manual => {
            let values = fl_values.as_ref().ok_or_else(|| {
                anyhow::anyhow!("--focal-lengths required when --focal-length-type=manual")
            })?;
            serde_json::json!({
                "focal_length_type": "manual",
                "focal_length_values": values,
            })
        }
        FocalLengthType::EstimateCommon => {
            serde_json::json!({ "focal_length_type": "estimate_common" })
        }
        FocalLengthType::EstimatePerImage => {
            serde_json::json!({ "focal_length_type": "estimate_per_image" })
        }
    };
    Ok(serde_json::json!({
        "focal_length_type": inner,
        "expressions_enabled": false,
    }))
}
