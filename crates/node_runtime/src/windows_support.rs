#![cfg(all(target_os = "windows", feature = "win-shell-adapters"))]

use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use http_client::Url;
use log::warn;
use reqwest_client::ReqwestClient;
use smol::fs;
use paths;

use crate::node_runtime::ManagedNodeRuntime;

#[derive(Clone, Debug, Default)]
pub struct ProxySettings {
    pub http: Option<String>,
    pub https: Option<String>,
    pub no_proxy: Option<String>,
}

impl ProxySettings {
    pub fn from_env() -> Self {
        Self {
            http: std::env::var("HTTP_PROXY").ok(),
            https: std::env::var("HTTPS_PROXY").ok(),
            no_proxy: std::env::var("NO_PROXY").ok(),
        }
    }
}

pub fn ensure_node(version: &str, proxy: Option<&ProxySettings>) -> Result<PathBuf> {
    smol::block_on(async move {
        let cache_root = paths::data_dir().join("node").join("versions");
        let offline_dir = cache_root.join(version);
        let offline_binary = offline_dir.join("node.exe");

        if offline_binary.exists() {
            return Ok(offline_binary);
        }

        if let Some(parent) = offline_binary.parent() {
            fs::create_dir_all(parent).await.ok();
        }

        let proxy_url = proxy
            .and_then(|settings| settings.https.as_ref().or(settings.http.as_ref()))
            .and_then(|value| Url::parse(value).ok());
        let previous_no_proxy = proxy.and_then(|settings| settings.no_proxy.clone());

        let previous_env = std::env::var("NO_PROXY").ok();
        if let Some(no_proxy) = previous_no_proxy.as_ref() {
            std::env::set_var("NO_PROXY", no_proxy);
        }

        let client = if let Some(url) = proxy_url.clone() {
            ReqwestClient::proxy_and_user_agent(Some(url), "Zed Node Runtime")?
        } else {
            ReqwestClient::new()
        };
        let http_client: Arc<dyn http_client::HttpClient> = Arc::new(client);

        if version != ManagedNodeRuntime::VERSION
            && version.trim_start_matches('v') != ManagedNodeRuntime::VERSION.trim_start_matches('v')
        {
            warn!(
                "requested Node.js version {} differs from bundled {}; using bundled build",
                version,
                ManagedNodeRuntime::VERSION
            );
        }

        let runtime = ManagedNodeRuntime::install_if_needed(&http_client).await?;
        let installed_binary = runtime.installation_path.join(ManagedNodeRuntime::NODE_PATH);

        if !installed_binary.exists() {
            anyhow::bail!(
                "managed Node.js binary missing at {}",
                installed_binary.display()
            );
        }

        fs::copy(&installed_binary, &offline_binary)
            .await
            .with_context(|| format!("copying Node.js binary to {}", offline_binary.display()))?;

        if let Some(value) = previous_env {
            std::env::set_var("NO_PROXY", value);
        }

        Ok(offline_binary)
    })
}
