use color_eyre::eyre::Result;

pub async fn account_exists(domain: &str, acct: &str) -> Result<bool> {
    let resource = format!("acct:{}", acct);

    let url = {
        let mut url = reqwest::Url::parse(domain)?
            .join(".well-known")?
            .join("webfinger")?;
        let mut query_pairs = url.query_pairs_mut();
        query_pairs.append_pair("resource", &resource);
        drop(query_pairs);
        url
    };

    let response = reqwest::get(url).await?;

    Ok(response.status().is_success())
}
