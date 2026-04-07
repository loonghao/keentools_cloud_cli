pub mod auth_cmd;
pub mod download;
pub mod ephemeral;
pub mod info;
pub mod init;
pub mod process;
pub mod run_pipeline;
pub mod status;
pub mod upload;

use crate::{client::ApiClient, output::OutputFormat};

pub struct Context {
    pub client: ApiClient,
    pub output: OutputFormat,
}
