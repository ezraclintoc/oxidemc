use futures::StreamExt;
use std::path::PathBuf;
use tokio::io::AsyncWriteExt;
use tokio::sync::mpsc::Sender;

#[derive(Debug, thiserror::Error)]
pub enum DownloadError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("msjm error: {0}")]
    Msjm(String),
}

pub fn get_platforms() -> &'static [&'static str] {
    msjm::PLATFORMS
}

pub async fn get_versions(platform: &str) -> Result<Vec<String>, DownloadError> {
    msjm::get_versions(platform)
        .await
        .map_err(|e| DownloadError::Msjm(e.to_string()))
}

#[derive(Debug, Clone)]
pub enum DownloadProgress {
    /// Fired once when the response arrives and total size is known
    Started { total_bytes: u64 },
    /// Fired after each chunk is written to disk
    Chunk { downloaded: u64, total: u64 },
    /// Fired when all bytes are written
    Finished,
}

pub async fn download_jar(
    platform: &str,
    version: &str,
    dest: &PathBuf,
    progress: Sender<DownloadProgress>,
) -> Result<(), DownloadError> {
    // get the download URL from msjm
    let url = msjm::get_download_url(platform, version)
        .await
        .map_err(|e| DownloadError::Msjm(e.to_string()))?;

    // start the request
    let response = reqwest::get(&url).await?;
    let total_bytes = response.content_length().unwrap_or(0);

    progress
        .send(DownloadProgress::Started { total_bytes })
        .await
        .ok();

    // create the destination file
    if let Some(parent) = dest.parent() {
        tokio::fs::create_dir_all(parent).await?;
    }
    let mut file = tokio::fs::File::create(dest).await?;

    // stream bytes to disk, reporting progress per chunk
    let mut downloaded: u64 = 0;
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        downloaded += chunk.len() as u64;
        file.write_all(&chunk).await?;
        progress
            .send(DownloadProgress::Chunk { downloaded, total: total_bytes })
            .await
            .ok();
    }

    file.flush().await?;
    progress.send(DownloadProgress::Finished).await.ok();

    Ok(())
}

pub fn jar_name(server_type: &str, version: &str) -> String {
    format!("{}-{}.jar", server_type, version)
}
