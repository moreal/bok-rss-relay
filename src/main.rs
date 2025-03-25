use std::time::Duration;

use domain::BokRssProvider;
use poem::{
    EndpointExt, Route, Server, endpoint::StaticFilesEndpoint, listener::TcpListener,
    middleware::Cors,
};
use poem_openapi::OpenApiService;
use reqwest::Url;

mod api;
mod domain;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let api_service = OpenApiService::new(
        api::RssApi {
            rss_provider: Box::new(BokRssProvider {
                endpoint: Url::parse("https://bok.or.kr")?,
            }),
        },
        "RSS relayer for Bank of Korea RSS feeds.",
        "1.0",
    )
    .server("http://localhost:3000/rss");

    let ui = api_service.swagger_ui();

    let app = Route::new()
        .nest("/rss", api_service)
        .nest(
            "/",
            StaticFilesEndpoint::new("./static").index_file("index.html"),
        )
        .nest("/docs", ui)
        .with(Cors::new());

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run_with_graceful_shutdown(
            app,
            async move {
                let _ = tokio::signal::ctrl_c().await;
            },
            Some(Duration::from_secs(5)),
        )
        .await
        .map_err(|x| x.into())
}
