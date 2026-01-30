use color_eyre::eyre::Result;
use tracing::{debug, instrument};

#[instrument()]
pub async fn account_exists(domain: &str, acct: &str) -> Result<bool> {
    let resource = format!("acct:{}", acct);

    let url = {
        let mut url = reqwest::Url::parse(domain)?;
        url.set_path(".well-known/webfinger");

        let mut query_pairs = url.query_pairs_mut();
        query_pairs.append_pair("resource", &resource);
        drop(query_pairs);
        url
    };

    debug!("Fetching WebFinger URL: {}", url);
    let response = reqwest::get(url).await?;
    let status = response.status();

    Ok(status.is_success())
}
