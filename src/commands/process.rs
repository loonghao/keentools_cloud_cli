use anyhow::{bail, Result};
use clap::Args;
use serde::Serialize;

use crate::{cli::FocalLengthType, output::Printer, validate};

use super::Context;

#[derive(Args, Debug)]
pub struct ProcessArgs {
    /// Avatar ID from `init`
    #[arg(long)]
    pub avatar_id: String,

    /// How to determine focal length for reconstruction
    #[arg(long, value_enum, default_value = "estimate-per-image")]
    pub focal_length_type: FocalLengthType,

    /// Comma-separated 35mm-equivalent focal lengths, one per photo.
    /// Required when --focal-length-type=manual.
    #[arg(long, value_delimiter = ',')]
    pub focal_lengths: Option<Vec<f32>>,

    /// Enable facial expression blendshapes in the output mesh
    #[arg(long)]
    pub expressions: bool,

    /// Validate request without starting reconstruction
    #[arg(long)]
    pub dry_run: bool,
}

/// Inner enum serializes as: {"focal_length_type": "estimate_per_image"} etc.
#[derive(Serialize)]
#[serde(tag = "focal_length_type", rename_all = "snake_case")]
enum FocalLengthInner {
    Manual { focal_length_values: Vec<f32> },
    EstimateCommon,
    EstimatePerImage,
}

/// Wrapper produces: {"focal_length_type": {"focal_length_type": "..."}}
#[derive(Serialize)]
struct FocalLengthPayload {
    focal_length_type: FocalLengthInner,
}

#[derive(Serialize)]
struct ProcessRequest {
    focal_length_type: FocalLengthPayload,
    expressions_enabled: bool,
}

pub async fn run(args: ProcessArgs, ctx: Context) -> Result<()> {
    let printer = Printer::new(ctx.output);

    validate::avatar_id(&args.avatar_id)?;

    let focal_payload = match &args.focal_length_type {
        FocalLengthType::Manual => {
            let values = args.focal_lengths.clone().ok_or_else(|| {
                anyhow::anyhow!(
                    "--focal-lengths is required when --focal-length-type=manual.\n\
                    Example: --focal-lengths 24.0,28.0,35.0"
                )
            })?;
            if values.is_empty() {
                bail!("--focal-lengths must not be empty");
            }
            FocalLengthInner::Manual {
                focal_length_values: values,
            }
        }
        FocalLengthType::EstimateCommon => FocalLengthInner::EstimateCommon,
        FocalLengthType::EstimatePerImage => FocalLengthInner::EstimatePerImage,
    };

    let body = ProcessRequest {
        focal_length_type: FocalLengthPayload {
            focal_length_type: focal_payload,
        },
        expressions_enabled: args.expressions,
    };

    if args.dry_run {
        printer.message(&format!(
            "Dry run: would start reconstruction for avatar {} (focal-length-type: {}, expressions: {})",
            args.avatar_id,
            args.focal_length_type.as_api_str(),
            args.expressions,
        ));
        return Ok(());
    }

    ctx.client
        .post_json::<_, serde_json::Value>(&format!("/v1/avatar/{}/process", args.avatar_id), &body)
        .await
        .or_else(|e| {
            // 200 with no body is success; handle parse errors gracefully
            let msg = e.to_string();
            if msg.contains("EOF") || msg.contains("parse") {
                Ok(serde_json::Value::Null)
            } else {
                Err(e)
            }
        })?;

    printer.message(&format!(
        "Reconstruction started for avatar {}. Use `status --avatar-id {} --poll` to wait.",
        args.avatar_id, args.avatar_id
    ));

    Ok(())
}
