
use std::time::Duration;

use renegadex_patcher::NamedUrl;

use crate::{as_string::AsString, error::Error};

#[derive(Clone)]
pub struct VersionInformation {
    pub launcher: LauncherVersion,
    pub software: SoftwareVersion
}

#[derive(Clone)]
pub struct SoftwareVersion {
    pub version: String,
    pub version_number: u64,
    pub name: String,
    pub(crate) instructions_hash: String,
    pub(crate) mirrors: Vec<NamedUrl>
}

#[derive(Clone)]
pub struct LauncherVersion {
    pub version: String,
    pub url: String,
    pub hash: String,
}

impl VersionInformation {
    pub async fn retrieve(url: &str) -> Result<Self, Error> {

        let mut downloader = download_async::Downloader::new();
        downloader.use_uri(url.parse::<download_async::http::Uri>()?);
        let headers = downloader.headers().expect("Couldn't unwrap download_async headers option");
        headers.append("User-Agent", format!("RenX-Patcher ({})", env!("CARGO_PKG_VERSION")).parse().unwrap());
        let mut buffer = vec![];
        downloader.allow_http();
        let response = downloader.download(download_async::Body::empty(), &mut buffer);
    
        let _ = tokio::time::timeout(Duration::from_secs(10), response).await??;

        let file = String::from_utf8(buffer)?;
        let parsed_json = json::parse(&file)?;
        let mirrors : Vec<NamedUrl> = parsed_json["game"]["mirrors"].members().map(|json| NamedUrl {
            name: json["name"].as_string(),
            url: json["url"].as_string(),
        }).collect();
        Ok(Self {
            launcher: LauncherVersion {
                version: parsed_json["launcher"]["version_name"].as_string(),
                url: parsed_json["launcher"]["patch_url"].as_string(),
                hash: parsed_json["launcher"]["patch_hash"].as_string(),
            },
            software: SoftwareVersion {
                name: parsed_json["game"]["version_name"].as_string(),
                version: parsed_json["game"]["patch_path"].as_string(),
                version_number: parsed_json["game"]["version_number"].as_u64().ok_or::<Error>(Error::None(format!("Cannot parse \"{}\" as u64", parsed_json["game"]["version_number"].dump())))?,
                instructions_hash: parsed_json["game"]["instructions_hash"].as_string(),
                mirrors
            }
        })
    }
}