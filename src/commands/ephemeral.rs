use anyhow::{bail, Result};
use clap::Args;
use serde::{Deserialize, Serialize};

use crate::{cli::FocalLengthType, output::Printer, validate};

use super::Context;

#[derive(Args, Debug)]
pub struct EphemeralArgs {
    /// Readable HTTPS URLs to input photos (repeat for multiple, 2–15 total)
    #[arg(long = "image-url", required = true)]
    pub image_urls: Vec<String>,

    /// Result mesh destinations. Format: FORMAT:PUT_URL
    /// Example: --result-url glb:https://... --result-url obj:https://...
    #[arg(long = "result-url", required = true)]
    pub result_urls: Vec<String>,

    /// How to determine focal length
    #[arg(long, value_enum, default_value = "estimate-per-image")]
    pub focal_length_type: FocalLengthType,

    /// Comma-separated 35mm focal lengths (required for --focal-length-type=manual)
    #[arg(long, value_delimiter = ',')]
    pub focal_lengths: Option<Vec<f32>>,

    /// Enable facial expression blendshapes
    #[arg(long)]
    pub expressions: bool,

    /// HTTPS callback URL for completion webhook
    #[arg(long)]
    pub callback_url: Option<String>,

    /// Validate request without calling the API
    #[arg(long)]
    pub dry_run: bool,
}

#[derive(Serialize)]
#[serde(tag = "focal_length_type", rename_all = "snake_case")]
enum FocalLengthPayload {
    Manual { focal_length_values: Vec<f32> },
    EstimateCommon,
    EstimatePerImage,
}

#[derive(Serialize)]
struct ResultMeshEntry {
    format: String,
    url: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    blendshapes: Vec<String>,
    edges: bool,
    mesh_lod: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    texture: Option<String>,
}

#[derive(Serialize)]
struct EphemeralRequest {
    image_urls: Vec<String>,
    result_mesh_urls: Vec<ResultMeshEntry>,
    #[serde(flatten)]
    focal_length: FocalLengthPayload,
    expressions_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    callback_url: Option<String>,
}

#[derive(Deserialize, Serialize, Debug)]
struct EphemeralResponse {
    avatar_id: String,
}

pub async fn run(args: EphemeralArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);

    validate::photo_count(args.image_urls.len())?;

    for url in &args.image_urls {
        validate::https_url(url)?;
    }

    if let Some(ref cb) = args.callback_url {
        validate::https_url(cb)?;
    }

    // Parse result_urls: FORMAT:PUT_URL
    let mut result_meshes: Vec<ResultMeshEntry> = Vec::new();
    for entry in &args.result_urls {
        let (fmt, url) = entry.split_once(':').ok_or_else(|| {
            anyhow::anyhow!(
                "Invalid --result-url '{}'. Expected format: FORMAT:HTTPS_URL\n\
                Example: glb:https://your-bucket.s3.amazonaws.com/result.glb?...",
                entry
            )
        })?;
        // Reconstruct full URL (split_once only takes first ':')
        let url = format!("https:{}", url);
        validate::https_url(&url)?;

        result_meshes.push(ResultMeshEntry {
            format: fmt.to_lowercase(),
            url,
            blendshapes: vec![],
            edges: false,
            mesh_lod: "high_poly".to_string(),
            texture: None,
        });
    }

    if result_meshes.is_empty() {
        bail!("At least one --result-url is required");
    }

    let focal_payload = match &args.focal_length_type {
        FocalLengthType::Manual => {
            let values = args.focal_lengths.clone().ok_or_else(|| {
                anyhow::anyhow!("--focal-lengths is required when --focal-length-type=manual")
            })?;
            FocalLengthPayload::Manual {
                focal_length_values: values,
            }
        }
        FocalLengthType::EstimateCommon => FocalLengthPayload::EstimateCommon,
        FocalLengthType::EstimatePerImage => FocalLengthPayload::EstimatePerImage,
    };

    let body = EphemeralRequest {
        image_urls: args.image_urls.clone(),
        result_mesh_urls: result_meshes,
        focal_length: focal_payload,
        expressions_enabled: args.expressions,
        callback_url: args.callback_url.clone(),
    };

    if args.dry_run {
        printer.message("Dry run: would create ephemeral avatar");
        printer.success(&serde_json::to_value(&body)?);
        return Ok(());
    }

    let resp: EphemeralResponse = ctx
        .client
        .post_json("/v1/avatar/ephemeral/create", &body)
        .await?;

    printer.success(&resp);

    if !printer.is_json() {
        printer.message(&format!(
            "Ephemeral avatar created: {}\n\
            Results will be uploaded to your provided URLs when processing completes.\n\
            Note: /info and /download endpoints are NOT available for ephemeral avatars after completion.",
            resp.avatar_id
        ));
    }

    Ok(())
}
