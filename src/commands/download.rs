use anyhow::{bail, Context as AnyhowContext, Result};
use clap::Args;
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::{output::Printer, validate};

use super::Context;

// ── File type detection via magic bytes ──────────────────────────────────────

/// Detected file type from magic bytes inspection.
#[derive(Debug, PartialEq)]
enum DetectedFileType {
    Zip,
    Gzip,
    Glb,
    Obj,
    Unknown,
}

/// Inspect the first few bytes of a file to determine its actual type.
/// This is defensive programming — we never trust the file extension alone.
fn detect_file_type(path: &Path) -> Result<DetectedFileType> {
    let mut header = [0u8; 4];
    let mut f = fs::File::open(path)
        .with_context(|| format!("Cannot open file for type detection: {}", path.display()))?;
    let bytes_read = f
        .read(&mut header)
        .with_context(|| format!("Cannot read file header: {}", path.display()))?;

    if bytes_read < 2 {
        return Ok(DetectedFileType::Unknown);
    }

    // GZIP: 1f 8b
    if header[0] == 0x1f && header[1] == 0x8b {
        return Ok(DetectedFileType::Gzip);
    }

    if bytes_read < 4 {
        // Could be a very small OBJ (e.g. "v 1") — check ASCII
        if header[0..bytes_read].iter().all(|b| b.is_ascii()) {
            return Ok(DetectedFileType::Obj);
        }
        return Ok(DetectedFileType::Unknown);
    }

    // ZIP: PK\x03\x04 (local file header) or PK\x05\x06 (empty archive)
    if (header[0] == b'P' && header[1] == b'K' && header[2] == 0x03 && header[3] == 0x04)
        || (header[0] == b'P' && header[1] == b'K' && header[2] == 0x05 && header[3] == 0x06)
    {
        return Ok(DetectedFileType::Zip);
    }

    // GLB: starts with "glTF" magic (0x676C5446)
    if &header == b"glTF" {
        return Ok(DetectedFileType::Glb);
    }

    // OBJ: ASCII text, typically starts with "v ", "vn", "vt", "f ", "# ", "mtllib", "o ", "g "
    if header.iter().all(|b| b.is_ascii()) {
        return Ok(DetectedFileType::Obj);
    }

    Ok(DetectedFileType::Unknown)
}

/// Decompress a GZIP file in-place: read → gunzip → overwrite same path.
/// This is a fallback for cases where reqwest's auto-gzip didn't activate
/// (e.g., double-gzip, or server using Transfer-Encoding instead of Content-Encoding).
fn decompress_gzip_in_place(path: &Path) -> Result<()> {
    let compressed =
        fs::read(path).with_context(|| format!("Cannot read GZIP file: {}", path.display()))?;
    let mut decoder = flate2::read::GzDecoder::new(&compressed[..]);
    let mut decompressed = Vec::new();
    decoder
        .read_to_end(&mut decompressed)
        .with_context(|| "Failed to decompress GZIP content")?;
    fs::write(path, &decompressed)
        .with_context(|| format!("Cannot write decompressed content to: {}", path.display()))?;
    Ok(())
}

/// Safely extract a ZIP archive, guarding against ZipSlip path traversal.
/// Returns the list of extracted file paths.
fn safe_extract_zip(zip_path: &Path, extract_dir: &Path) -> Result<Vec<String>> {
    let zip_file = fs::File::open(zip_path)
        .with_context(|| format!("Cannot open ZIP: {}", zip_path.display()))?;
    let mut archive =
        zip::ZipArchive::new(zip_file).with_context(|| "Failed to read ZIP archive")?;

    // Canonicalize the target directory for ZipSlip check
    fs::create_dir_all(extract_dir)?;
    let canonical_dir = extract_dir
        .canonicalize()
        .with_context(|| format!("Cannot canonicalize extract dir: {}", extract_dir.display()))?;

    let mut extracted: Vec<String> = Vec::new();
    for i in 0..archive.len() {
        let mut entry = archive.by_index(i)?;
        let entry_name = entry.name().to_owned();

        // ZipSlip protection: reject entries with path traversal
        if entry_name.contains("..") {
            bail!("ZIP entry contains path traversal: {}", entry_name);
        }

        let dest = extract_dir.join(&entry_name);

        // Extra safety: verify the resolved path is still inside extract_dir
        if let Ok(canonical_dest) = dest.canonicalize() {
            if !canonical_dest.starts_with(&canonical_dir) {
                bail!("ZIP entry escapes target directory: {}", entry_name);
            }
        }
        // If canonicalize fails (file doesn't exist yet), that's fine — we'll create it

        // Create parent directories if needed
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut buf = Vec::new();
        entry.read_to_end(&mut buf)?;
        fs::write(&dest, &buf)
            .with_context(|| format!("Cannot write extracted file: {}", dest.display()))?;
        extracted.push(dest.display().to_string());
    }

    Ok(extracted)
}

/// Result of a successful download, carrying the actual output paths.
/// When the server returns a ZIP archive, the output is the extracted files
/// (e.g., neutral.obj + neutral.mtl + textures), NOT the original output_path.
pub struct DownloadResult {
    /// The primary mesh file path (the .obj or .glb file to use).
    pub primary_path: PathBuf,
    /// All extracted file paths (only populated for ZIP archives).
    #[allow(dead_code)]
    pub extracted: Vec<String>,
}

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

pub async fn run(args: DownloadArgs, ctx: Context) -> Result<DownloadResult> {
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

    // Download to a temporary path first, then inspect actual content via magic bytes
    let tmp_dest = args.output_path.with_extension("download");

    ctx.client
        .download_to_file(&download_url, &tmp_dest, progress_cb.as_deref())
        .await?;

    if let Some(p) = pb {
        p.finish_and_clear();
    }

    // ── Post-download: detect actual file type via magic bytes ─────────────
    // The server may return GZIP-wrapped content (even with reqwest gzip enabled,
    // as a fallback for double-gzip or Transfer-Encoding edge cases), a ZIP archive,
    // a bare OBJ/GLB file, or something unexpected. We handle all cases defensively.

    let file_type = detect_file_type(&tmp_dest)?;

    // If GZIP, decompress first then re-detect the inner content type
    if file_type == DetectedFileType::Gzip {
        decompress_gzip_in_place(&tmp_dest)
            .with_context(|| "Failed to decompress GZIP-wrapped download")?;
        // Re-detect after decompression
        let inner_type = detect_file_type(&tmp_dest)?;

        // Recursive GZIP check (extremely unlikely but defensive)
        if inner_type == DetectedFileType::Gzip {
            decompress_gzip_in_place(&tmp_dest)
                .with_context(|| "Failed to decompress double-GZIP content")?;
        }
    }

    // Re-detect type after potential GZIP decompression
    let final_type = detect_file_type(&tmp_dest)?;

    match final_type {
        DetectedFileType::Zip => {
            // Extract ZIP archive (typically OBJ + MTL + textures)
            let out_dir = args.output_path.parent().unwrap_or_else(|| Path::new("."));
            let extracted = safe_extract_zip(&tmp_dest, out_dir)?;

            // Remove the temp download file after successful extraction
            let _ = fs::remove_file(&tmp_dest);

            // Find the primary mesh file (.obj or .glb) from extracted list
            let primary = extracted
                .iter()
                .find(|f| f.to_lowercase().ends_with(".obj") || f.to_lowercase().ends_with(".glb"))
                .cloned()
                .unwrap_or_else(|| extracted.first().cloned().unwrap_or_default());

            printer.success(&serde_json::json!({
                "format": format_str,
                "extracted": extracted,
            }));

            Ok(DownloadResult {
                primary_path: PathBuf::from(&primary),
                extracted,
            })
        }
        DetectedFileType::Glb | DetectedFileType::Obj => {
            // Single file output — rename temp file to target path
            fs::rename(&tmp_dest, &args.output_path)
                .with_context(|| format!("Cannot save file to: {}", args.output_path.display()))?;
            printer.success(&serde_json::json!({
                "saved_to": args.output_path.display().to_string(),
                "format": format_str,
            }));

            Ok(DownloadResult {
                primary_path: args.output_path.clone(),
                extracted: vec![],
            })
        }
        DetectedFileType::Gzip => {
            // Should not reach here after decompression, but handle gracefully
            bail!(
                "Downloaded file is still GZIP after decompression attempts. \
                 The server may be triple-compressing or returning corrupt data."
            );
        }
        DetectedFileType::Unknown => {
            // Unknown type — save as-is and warn
            fs::rename(&tmp_dest, &args.output_path)
                .with_context(|| format!("Cannot save file to: {}", args.output_path.display()))?;
            eprintln!(
                "Warning: downloaded file has unknown format. Saved as-is to {}",
                args.output_path.display()
            );
            printer.success(&serde_json::json!({
                "saved_to": args.output_path.display().to_string(),
                "format": format_str,
                "warning": "unknown_file_type",
            }));

            Ok(DownloadResult {
                primary_path: args.output_path.clone(),
                extracted: vec![],
            })
        }
    }
}
