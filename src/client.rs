use anyhow::{bail, Context, Result};
use reqwest::{header, Client, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use std::path::Path;

#[derive(Clone)]
pub struct ApiClient {
    http: Client,
    pub base_url: String,
}

impl ApiClient {
    pub fn new(token: String, base_url: String) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        let auth_value = header::HeaderValue::from_str(&format!("Bearer {}", token))
            .context("Invalid token: contains non-ASCII characters")?;
        headers.insert(header::AUTHORIZATION, auth_value);

        let http = Client::builder()
            .default_headers(headers)
            .user_agent(concat!("keentools-cloud-cli/", env!("CARGO_PKG_VERSION")))
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self { http, base_url })
    }

    /// POST JSON body, deserialize JSON response.
    pub async fn post_json<B: Serialize, R: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .json(body)
            .send()
            .await
            .with_context(|| format!("POST {} failed", url))?;

        handle_response(resp).await
    }

    /// POST with no body, expect 200 OK (no response body).
    #[allow(dead_code)]
    pub async fn post_empty(&self, path: &str) -> Result<()> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .post(&url)
            .header(header::CONTENT_LENGTH, "0")
            .send()
            .await
            .with_context(|| format!("POST {} failed", url))?;

        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        bail!("API error {}: {}", status, body);
    }

    /// GET, deserialize JSON response.
    pub async fn get_json<R: DeserializeOwned>(&self, path: &str) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .send()
            .await
            .with_context(|| format!("GET {} failed", url))?;

        handle_response(resp).await
    }

    /// GET with query params, deserialize JSON response.
    #[allow(dead_code)]
    pub async fn get_json_with_query<R: DeserializeOwned, Q: Serialize>(
        &self,
        path: &str,
        query: &Q,
    ) -> Result<R> {
        let url = format!("{}{}", self.base_url, path);
        let resp = self
            .http
            .get(&url)
            .query(query)
            .send()
            .await
            .with_context(|| format!("GET {} failed", url))?;

        handle_response(resp).await
    }

    /// Upload a file via PUT to a pre-signed S3 URL (no auth header needed).
    pub async fn put_file(&self, presigned_url: &str, file_path: &Path) -> Result<()> {
        let bytes = tokio::fs::read(file_path)
            .await
            .with_context(|| format!("Failed to read file: {}", file_path.display()))?;

        let mime = guess_mime(file_path);

        // Use a fresh client without our auth headers for S3 pre-signed URLs
        let s3_client = Client::new();
        let resp = s3_client
            .put(presigned_url)
            .header(header::CONTENT_TYPE, mime)
            .body(bytes)
            .send()
            .await
            .context("PUT to pre-signed URL failed")?;

        let status = resp.status();
        if status.is_success() {
            return Ok(());
        }
        let body = resp.text().await.unwrap_or_default();
        bail!("S3 upload error {}: {}", status, body);
    }

    /// Download bytes from a pre-signed URL, saving to a file.
    pub async fn download_to_file(
        &self,
        url: &str,
        dest: &Path,
        on_progress: Option<&dyn Fn(u64, Option<u64>)>,
    ) -> Result<()> {
        let resp = Client::new()
            .get(url)
            .send()
            .await
            .with_context(|| "Download request failed")?;

        let status = resp.status();
        if !status.is_success() {
            let body = resp.text().await.unwrap_or_default();
            bail!("Download error {}: {}", status, body);
        }

        let total = resp.content_length();
        let mut stream = resp.bytes_stream();
        let mut file = tokio::fs::File::create(dest)
            .await
            .with_context(|| format!("Cannot create file: {}", dest.display()))?;
        let mut downloaded: u64 = 0;

        use futures_util::StreamExt;
        use tokio::io::AsyncWriteExt;

        while let Some(chunk) = stream.next().await {
            let chunk = chunk.context("Error reading download stream")?;
            downloaded += chunk.len() as u64;
            file.write_all(&chunk)
                .await
                .context("Error writing download to disk")?;
            if let Some(cb) = on_progress {
                cb(downloaded, total);
            }
        }
        Ok(())
    }
}

async fn handle_response<R: DeserializeOwned>(resp: reqwest::Response) -> Result<R> {
    let status = resp.status();
    if status.is_success() {
        let json = resp
            .json::<R>()
            .await
            .context("Failed to parse API response")?;
        return Ok(json);
    }
    let body = resp.text().await.unwrap_or_default();
    map_api_error(status, &body)
}

fn map_api_error<R>(status: StatusCode, body: &str) -> Result<R> {
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => {
            bail!("Authentication failed ({}): {}", status, body);
        }
        StatusCode::NOT_FOUND => {
            bail!("Not found (404): {}", body);
        }
        StatusCode::UNPROCESSABLE_ENTITY => {
            bail!("Reconstruction failed (422): {}", body);
        }
        StatusCode::TOO_EARLY => {
            bail!("Avatar not ready yet (425): {}", body);
        }
        _ => {
            bail!("API error {}: {}", status, body);
        }
    }
}

fn guess_mime(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("jpg") | Some("jpeg") => "image/jpeg",
        Some("png") => "image/png",
        Some("heic") | Some("heif") => "image/heic",
        _ => "application/octet-stream",
    }
}
