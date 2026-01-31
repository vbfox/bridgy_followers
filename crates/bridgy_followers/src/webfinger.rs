use color_eyre::eyre::Result;
use reqwest::Url;
use tracing::{debug, instrument};

fn get_domain_webfinger_url(domain: &str, resource: &str) -> Result<Url, url::ParseError> {
    let mut url = Url::parse(domain)?;
    url.set_path(".well-known/webfinger");

    let mut query_pairs = url.query_pairs_mut();
    query_pairs.append_pair("resource", &resource);
    drop(query_pairs);

    Ok(url)
}

#[instrument()]
pub async fn account_exists(domain: &str, acct: &str) -> Result<bool> {
    let resource = format!("acct:{}", acct);
    let url = get_domain_webfinger_url(domain, &resource)?;

    debug!("Fetching WebFinger URL: {}", url);
    let response = reqwest::get(url).await?;
    let status = response.status();

    Ok(status.is_success())
}
