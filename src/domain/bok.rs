use reqwest::Url;
use rss::Channel;

#[async_trait::async_trait]
pub trait RssProvider: Send + Sync {
    async fn get_channel(&self, id: &str, sub_id: Option<&str>) -> Result<Channel, anyhow::Error>;
}

pub struct BokRssProvider {
    pub endpoint: Url,
}

#[async_trait::async_trait]
impl RssProvider for BokRssProvider {
    async fn get_channel(&self, id: &str, sub_id: Option<&str>) -> Result<Channel, anyhow::Error> {
        let url = self
            .endpoint
            .join(format!("/portal/bbs/{}/news.rss", id).as_str())?
            .join(
                sub_id
                    .map(|x| format!("?menuNo={}", x))
                    .unwrap_or("".to_string())
                    .as_str(),
            )?;

        let text = reqwest::get(url).await?.text().await?;
        let reader = std::io::Cursor::new(text);

        return Ok(Channel::read_from(reader)?);
    }
}
