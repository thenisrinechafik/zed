#![cfg(all(feature = "win-collab", target_os = "windows"))]

use anyhow::{Context, Result};
use http_client::Url;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};
use reqwest_client::ReqwestClient;
use std::time::Duration;

/// Builds a `ReqwestClient` that respects system proxy settings and
/// trusts the Windows certificate store.
pub fn http_client_for_user_agent(agent: &str) -> Result<ReqwestClient> {
    let mut headers = HeaderMap::new();
    headers.insert(USER_AGENT, HeaderValue::from_str(agent)?);

    let mut builder = reqwest::Client::builder()
        .use_preconfigured_tls(http_client_tls::tls_config())
        .default_headers(headers)
        .connect_timeout(Duration::from_secs(10));

    if let Some(proxy) = proxy_from_environment().context("parse proxy from environment")? {
        builder = builder.proxy(proxy);
    }

    let client = builder.build()?;
    Ok(client.into())
}

pub(crate) fn proxy_from_environment() -> Result<Option<reqwest::Proxy>> {
    let proxy = std::env::var("HTTPS_PROXY")
        .or_else(|_| std::env::var("HTTP_PROXY"))
        .ok()
        .and_then(|url| Url::parse(&url).ok());

    if let Some(url) = proxy {
        let proxy = reqwest::Proxy::all(url).context("configure proxy")?;
        Ok(Some(proxy.no_proxy(reqwest::NoProxy::from_env())))
    } else {
        Ok(None)
    }
}
