use anyhow::Result;
use clap::Args;
use serde_json::{json, Value};

use crate::output::OutputFormat;

/// Dump CLI and API capabilities as JSON for agent consumption.
///
/// This command implements runtime schema introspection per the Agent DX guidelines —
/// agents don't need to read documentation or call external URLs.
#[derive(Args, Debug)]
pub struct SchemaArgs {
    /// Command name to describe. Omit to list all commands.
    pub command: Option<String>,
}

pub fn run(args: SchemaArgs, _output: OutputFormat) -> Result<()> {
    let schema = build_schema();

    let output = match args.command.as_deref() {
        None => schema,
        Some(cmd) => schema
            .get("commands")
            .and_then(|c| c.get(cmd))
            .cloned()
            .unwrap_or_else(|| json!({ "error": format!("unknown command: {}", cmd) })),
    };

    println!("{}", serde_json::to_string_pretty(&output)?);
    Ok(())
}

fn build_schema() -> Value {
    json!({
        "name": "keentools-cloud",
        "version": env!("CARGO_PKG_VERSION"),
        "description": "Unofficial CLI for the KeenTools Cloud 3D Head Reconstruction API",
        "disclaimer": "This is an unofficial tool, not affiliated with or endorsed by KeenTools.",
        "base_url": "Set via KEENTOOLS_API_URL env var or --api-url flag (required)",
        "auth": {
            "type": "bearer",
            "env_var": "KEENTOOLS_API_TOKEN",
            "flag": "--token",
            "config_file": "~/.config/keentools-cloud/config.toml"
        },
        "global_flags": {
            "--token": { "type": "string", "description": "API token" },
            "--output": { "type": "enum", "values": ["human", "json"], "description": "Output format (default: human on TTY, json otherwise)" },
            "--api-url": { "type": "string", "env": "KEENTOOLS_API_URL", "required": true, "description": "API base URL" }
        },
        "exit_codes": {
            "0": "success",
            "1": "API or runtime error",
            "2": "input validation error",
            "3": "authentication error"
        },
        "commands": {
            "init": {
                "description": "Initialize a new avatar reconstruction session. Returns avatar_id and upload_urls.",
                "flags": {
                    "--count": { "type": "integer", "range": [2, 15], "required": true, "description": "Number of photos to upload" },
                    "--dry-run": { "type": "bool", "description": "Validate locally without calling the API" }
                },
                "output": {
                    "avatar_id": "string",
                    "upload_urls": "array of pre-signed S3 PUT URLs (one per photo)"
                },
                "agent_tip": "Pipe output to `upload` via avatar_id. Use --dry-run first."
            },
            "upload": {
                "description": "Upload photos to pre-signed S3 URLs obtained from `init`.",
                "args": {
                    "PHOTOS": "One or more local image file paths (JPEG, PNG, HEIC)"
                },
                "flags": {
                    "--avatar-id": { "type": "string", "required": true, "description": "Avatar ID from init" },
                    "--urls": { "type": "string", "description": "Comma-separated pre-signed URLs (alternative to reading from init output)" }
                },
                "agent_tip": "Photos and upload_urls are matched by position. Must upload exactly --count files."
            },
            "process": {
                "description": "Start 3D reconstruction after all photos are uploaded.",
                "flags": {
                    "--avatar-id": { "type": "string", "required": true },
                    "--focal-length-type": {
                        "type": "enum",
                        "values": ["estimate-common", "estimate-per-image", "manual"],
                        "required": true,
                        "description": "How to determine focal length. Use estimate-per-image for mixed cameras."
                    },
                    "--focal-lengths": { "type": "string", "description": "Comma-separated 35mm-equivalent values, one per photo. Required when --focal-length-type=manual." },
                    "--expressions": { "type": "bool", "description": "Enable expression blendshapes (makes 'expression' group available in download)" },
                    "--dry-run": { "type": "bool", "description": "Validate request without calling API" }
                },
                "agent_tip": "Use --dry-run before triggering reconstruction. Reconstruction cannot be cancelled."
            },
            "status": {
                "description": "Check reconstruction status. Statuses: not_started, running, completed, failed, deleted.",
                "flags": {
                    "--avatar-id": { "type": "string", "required": true },
                    "--poll": { "type": "bool", "description": "Keep polling until completed or failed" },
                    "--poll-interval": { "type": "integer", "default": 5, "description": "Seconds between polls" }
                },
                "output": {
                    "status": "not_started | running | completed | failed | deleted",
                    "progress": "float 0.0–1.0 (when running)"
                },
                "agent_tip": "Use --poll to wait for completion instead of manually polling in a loop."
            },
            "download": {
                "description": "Download the completed 3D model. Uses polling protocol internally.",
                "flags": {
                    "--avatar-id": { "type": "string", "required": true },
                    "--output-path": { "type": "path", "required": true, "description": "Where to save the file" },
                    "--format": { "type": "enum", "values": ["glb", "obj"], "default": "glb" },
                    "--blendshapes": { "type": "string", "description": "Comma-separated: arkit,expression,nose (GLB only)" },
                    "--texture": { "type": "enum", "values": ["jpg", "png"], "description": "Include texture (GLB only)" },
                    "--edges": { "type": "bool", "description": "Include wireframe edges (GLB only)" },
                    "--poll": { "type": "bool", "description": "Wait for model to be ready" }
                },
                "agent_tip": "OBJ format is always a ZIP archive. Use --blendshapes arkit,nose (comma-separated, NOT repeated flags)."
            },
            "info": {
                "description": "Get reconstruction metadata: camera matrices, focal length used, image URLs.",
                "flags": {
                    "--avatar-id": { "type": "string", "required": true }
                },
                "output": {
                    "camera_positions": "array of 4x4 world-to-camera matrices (row-major)",
                    "camera_projections": "array of 4x4 intrinsic matrices",
                    "focal_length_type": "manual | exif | estimated_common | estimated_per_image",
                    "expressions_enabled": "bool",
                    "img_urls": "array of pre-signed S3 URLs for preprocessed images (null for ephemeral)"
                }
            },
            "run": {
                "description": "Full pipeline shortcut: init → upload → process → wait → download.",
                "args": {
                    "PHOTOS": "Local photo paths (2–15)"
                },
                "flags": {
                    "--output-path": { "type": "path", "required": true },
                    "--focal-length-type": { "type": "enum", "values": ["estimate-common", "estimate-per-image", "manual"], "default": "estimate-per-image" },
                    "--focal-lengths": { "type": "string", "description": "Required when --focal-length-type=manual" },
                    "--expressions": { "type": "bool" },
                    "--format": { "type": "enum", "values": ["glb", "obj"], "default": "glb" },
                    "--blendshapes": { "type": "string", "default": "" },
                    "--texture": { "type": "enum", "values": ["jpg", "png"] },
                    "--edges": { "type": "bool" },
                    "--dry-run": { "type": "bool" }
                },
                "agent_tip": "Preferred command for simple use cases. Use individual commands for more control."
            },
            "ephemeral": {
                "description": "Zero-retention pipeline. Photos never stored server-side; results pushed to your URLs.",
                "flags": {
                    "--image-url": { "type": "string", "multiple": true, "required": true, "description": "Readable HTTPS URL to an input photo (2–15 total)" },
                    "--result-url": { "type": "string", "multiple": true, "required": true, "description": "Format: FORMAT:PUT_URL (e.g. glb:https://...)" },
                    "--focal-length-type": { "type": "enum", "values": ["estimate-common", "estimate-per-image", "manual"], "required": true },
                    "--focal-lengths": { "type": "string" },
                    "--expressions": { "type": "bool" },
                    "--callback-url": { "type": "string", "description": "HTTPS URL for completion webhook" },
                    "--dry-run": { "type": "bool" }
                },
                "agent_tip": "After completion, avatar data is destroyed. /info and /download endpoints will return 404. Results are only available at the URLs you provided."
            },
            "schema": {
                "description": "Dump CLI capabilities as JSON. Use to discover commands and parameters at runtime.",
                "args": {
                    "COMMAND": "Optional: command name to describe"
                },
                "agent_tip": "Always machine-readable JSON regardless of --output flag."
            },
            "auth": {
                "description": "Manage the API token in the config file.",
                "subcommands": {
                    "login": "Save token to config file",
                    "logout": "Remove token from config file",
                    "status": "Show current token source and masked value"
                }
            }
        }
    })
}
