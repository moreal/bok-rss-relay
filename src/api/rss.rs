use poem::Result;
use poem_openapi::{
    ApiResponse, OpenApi, Tags,
    param::{Path, Query},
    payload::PlainText,
};

use crate::domain::{RssProvider, unescape_rss};

#[derive(Tags)]
enum ApiTags {
    #[oai(rename = "RSS")]
    Rss,
}

pub struct RssApi {
    pub rss_provider: Box<dyn RssProvider>,
}

#[derive(ApiResponse)]
enum RssResponse {
    #[oai(status = 200, content_type = "application/xml;charset=UTF-8")]
    Ok(PlainText<String>),
}

#[OpenApi]
impl RssApi {
    #[oai(path = "/:id", method = "get", tag = "ApiTags::Rss")]
    async fn get_rss(
        &self,
        id: Path<String>,
        #[oai(name = "menuNo")] menu_no: Query<String>,
    ) -> Result<RssResponse> {
        let channel = self
            .rss_provider
            .get_channel(&id, Some(&menu_no))
            .await
            .map_err(|_| {
                poem::Error::from_string(
                    "Failed to fetch rss.",
                    poem::http::StatusCode::INTERNAL_SERVER_ERROR,
                )
            })?;

        Ok(RssResponse::Ok(PlainText(
            unescape_rss(channel).to_string(),
        )))
    }
}
#[cfg(test)]
mod tests {
    use rss::Channel;
    use std::io::Cursor;

    use super::*;

    struct MockRssProvider {
        success: bool,
    }

    #[async_trait::async_trait]
    impl RssProvider for MockRssProvider {
        async fn get_channel(
            &self,
            id: &str,
            _sub_id: Option<&str>,
        ) -> Result<Channel, anyhow::Error> {
            if self.success {
                Ok(if id == "escaped" {
                    Channel::read_from(Cursor::new("
<?xml version=\"1.0\" encoding=\"UTF-8\"?><rss xmlns:dc=\"http://purl.org/dc/elements/1.1/\" version=\"2.0\">
    <channel>
        <title>Title</title>
        <description>Description</description>
        <item>
            <title>ItemTitle</title>
            <description><![CDATA[Contains &lt;tags&gt; and escape chars]]></description>
        </item>
    </channel>
</rss></xml>"))?
                } else {
                    Channel {
                        title: "Test Channel".to_string(),
                        link: "http://example.com".to_string(),
                        description: "Test Description".to_string(),
                        ..Default::default()
                    }
                })
            } else {
                Err(anyhow::Error::msg("Mocked error"))
            }
        }
    }

    #[tokio::test]
    async fn test_get_rss_success() {
        let api = RssApi {
            rss_provider: Box::new(MockRssProvider { success: true }),
        };

        let result = api
            .get_rss(Path("id".to_string()), Query("123".to_string()))
            .await;

        assert!(result.is_ok());

        if let Ok(RssResponse::Ok(PlainText(content))) = result {
            assert!(content.contains("Test Channel"));
            assert!(content.contains("http://example.com"));
            assert!(content.contains("Test Description"));
        }
    }

    #[tokio::test]
    async fn test_get_rss_error() {
        let api = RssApi {
            rss_provider: Box::new(MockRssProvider { success: false }),
        };

        let result = api
            .get_rss(Path("invalid_id".to_string()), Query("123".to_string()))
            .await;

        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(err.to_string(), "Failed to fetch rss.");
        assert_eq!(err.status(), poem::http::StatusCode::INTERNAL_SERVER_ERROR);
    }

    #[tokio::test]
    async fn test_get_rss_with_escaped_content() {
        let api = RssApi {
            rss_provider: Box::new(MockRssProvider { success: true }),
        };

        let result = api
            .get_rss(Path("escaped".to_string()), Query("123".to_string()))
            .await;

        assert!(result.is_ok());

        if let Ok(RssResponse::Ok(PlainText(content))) = result {
            assert!(content.contains("Contains <tags> and escape chars"));
        }
    }
}
