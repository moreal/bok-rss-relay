use domain::BokRssProvider;
use poem::{EndpointExt, Route, Server, listener::TcpListener, middleware::Cors};
use poem_openapi::OpenApiService;
use reqwest::Url;

mod api;
mod domain;

#[tokio::main]
async fn main() -> Result<(), anyhow::Error> {
    let api_service = OpenApiService::new(
        api::Api {
            rss_provider: Box::new(BokRssProvider {
                endpoint: Url::parse("https://bok.or.kr")?,
            }),
        },
        "RSS relayer for Bank of Korea RSS feeds.",
        "1.0",
    )
    .server("http://localhost:3000/");

    let ui = api_service.swagger_ui();

    let app = Route::new()
        .nest("/", api_service)
        .nest("/docs", ui)
        .with(Cors::new());

    Server::new(TcpListener::bind("0.0.0.0:3000"))
        .run(app)
        .await
        .map_err(|x| x.into())
}
