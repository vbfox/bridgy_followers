use color_eyre::eyre::Result;
use reqwest::Url;
use tracing::{debug, instrument};

/// Construct the WebFinger URL for a given domain and resource according to [RFC 7033][rfc7033].
///
/// [rfc7033]: https://datatracker.ietf.org/doc/html/rfc7033
fn get_domain_webfinger_url(domain: &str, resource: &str) -> Result<Url, url::ParseError> {
    let mut url = Url::parse(domain)?;
    url.set_path(".well-known/webfinger");

    let mut query_pairs = url.query_pairs_mut();
    query_pairs.append_pair("resource", &resource);
    drop(query_pairs);

    Ok(url)
}

/// Uses WebFinger (RFC [7565][rfc7565]) to check if an account exists on a given
/// domain using its acct URI.
///
/// [rfc7565]: https://datatracker.ietf.org/doc/html/rfc7565
#[instrument()]
pub async fn account_exists(domain: &str, acct: &str) -> Result<bool> {
    let resource = format!("acct:{}", acct);
    let url = get_domain_webfinger_url(domain, &resource)?;

    debug!("Fetching WebFinger URL: {}", url);
    let response = reqwest::get(url).await?;
    let status = response.status();

    Ok(status.is_success())
}
