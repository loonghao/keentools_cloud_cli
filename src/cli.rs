use clap::{Parser, Subcommand, ValueEnum};

use crate::output::OutputFormat;

/// Unofficial CLI for the KeenTools Cloud 3D Head Reconstruction API.
///
/// This is an unofficial tool and is not affiliated with or endorsed by KeenTools.
/// Obtain your API token from your KeenTools account.
#[derive(Parser)]
#[command(
    name = "keentools-cloud",
    version,
    about,
    long_about = None,
    after_help = "ENVIRONMENT:\n  KEENTOOLS_API_TOKEN    API authentication token\n  KEENTOOLS_API_URL      API base URL\n\nEXIT CODES:\n  0  Success\n  1  API or runtime error\n  2  Input validation error\n  3  Authentication error"
)]
pub struct Cli {
    /// API authentication token (overrides KEENTOOLS_API_TOKEN env var and config file)
    #[arg(long, global = true, env = "KEENTOOLS_API_TOKEN")]
    pub token: Option<String>,

    /// Output format
    #[arg(long, global = true, value_enum)]
    pub output: Option<OutputFormat>,

    /// KeenTools Cloud API base URL
    #[arg(long, global = true, env = "KEENTOOLS_API_URL")]
    pub api_url: Option<String>,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Initialize a new avatar reconstruction session
    Init(crate::commands::init::InitArgs),

    /// Upload photos to the pre-signed URLs obtained from `init`
    Upload(crate::commands::upload::UploadArgs),

    /// Start reconstruction after all photos are uploaded
    Process(crate::commands::process::ProcessArgs),

    /// Check reconstruction status
    Status(crate::commands::status::StatusArgs),

    /// Download the completed 3D model
    Download(crate::commands::download::DownloadArgs),

    /// Get reconstruction metadata (camera matrices, focal length info, etc.)
    Info(crate::commands::info::InfoArgs),

    /// Run the full pipeline: init → upload → process → wait → download
    Run(crate::commands::run_pipeline::RunArgs),

    /// Ephemeral pipeline — zero data retention, results sent directly to your URLs
    Ephemeral(crate::commands::ephemeral::EphemeralArgs),

    /// Dump CLI and API schema as JSON (useful for agents)
    Schema(crate::schema::SchemaArgs),

    /// Manage authentication token
    Auth(crate::commands::auth_cmd::AuthArgs),
}

/// Focal length handling mode
#[derive(Clone, Debug, ValueEnum)]
pub enum FocalLengthType {
    /// Estimate one shared focal length for all photos (all must have same resolution)
    EstimateCommon,
    /// Estimate individual focal length per photo
    EstimatePerImage,
    /// Provide explicit focal lengths (requires --focal-lengths)
    Manual,
}

impl FocalLengthType {
    pub fn as_api_str(&self) -> &'static str {
        match self {
            FocalLengthType::EstimateCommon => "estimate_common",
            FocalLengthType::EstimatePerImage => "estimate_per_image",
            FocalLengthType::Manual => "manual",
        }
    }
}

/// Mesh output format
#[derive(Clone, Debug, ValueEnum)]
pub enum MeshFormat {
    /// glTF Binary — single file with embedded textures (~40 MB). Recommended.
    Glb,
    /// Wavefront OBJ — always returned as a ZIP archive
    Obj,
}

/// Blendshape groups (GLB only)
#[derive(Clone, Debug, ValueEnum)]
pub enum Blendshape {
    /// 51 ARKit-compatible morph targets
    Arkit,
    /// Numbered expression morphs (requires --expressions during process)
    Expression,
    /// Nose shape controls
    Nose,
}
