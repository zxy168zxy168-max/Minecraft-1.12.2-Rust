use std::fs;
use std::path::Path;
use std::time::Duration;

use rand::distributions::Alphanumeric;
use rand::Rng;
use thiserror::Error;

const SKIN_URL: &str = "https://api.minecraftservices.com/minecraft/profile/skins";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkinVariant {
    Classic,
    Slim,
}
impl SkinVariant {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Classic => "classic",
            Self::Slim => "slim",
        }
    }
}

#[derive(Debug, Error)]
pub enum SkinUploadError {
    #[error("skin file is not a PNG")]
    InvalidPng,
    #[error("failed reading skin file: {0}")]
    Read(#[source] std::io::Error),
    #[error("Minecraft access token is empty")]
    MissingToken,
    #[error("skin upload failed: {0}")]
    Http(String),
}

pub fn upload_skin(
    path: &Path,
    variant: SkinVariant,
    accessToken: &str,
) -> Result<(), SkinUploadError> {
    if accessToken.trim().is_empty() {
        return Err(SkinUploadError::MissingToken);
    }
    let bytes = fs::read(path).map_err(SkinUploadError::Read)?;
    if bytes.len() < 8 || &bytes[..8] != b"\x89PNG\r\n\x1a\n" {
        return Err(SkinUploadError::InvalidPng);
    }
    let boundary: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(24)
        .map(char::from)
        .collect();
    let filename = path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("skin.png");
    let mut body = Vec::new();
    body.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"variant\"\r\n\r\n{}\r\n",
            variant.as_str()
        )
        .as_bytes(),
    );
    body.extend_from_slice(format!("--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: image/png\r\n\r\n").as_bytes());
    body.extend_from_slice(&bytes);
    body.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    match ureq::post(SKIN_URL)
        .set("Accept", "*/*")
        .set("Authorization", &format!("Bearer {accessToken}"))
        .set("User-Agent", "MojangSharp/0.1")
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .timeout(Duration::from_secs(30))
        .send_bytes(&body)
    {
        Ok(response) if matches!(response.status(), 200 | 204) => Ok(()),
        Ok(response) => Err(SkinUploadError::Http(format!("HTTP {}", response.status()))),
        Err(ureq::Error::Status(status, response)) => {
            let body = response.into_string().unwrap_or_default();
            Err(SkinUploadError::Http(if body.trim().is_empty() {
                format!("HTTP {status}")
            } else {
                format!("HTTP {status}: {}", body.trim())
            }))
        }
        Err(ureq::Error::Transport(error)) => Err(SkinUploadError::Http(error.to_string())),
    }
}
