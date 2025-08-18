use chrono::DateTime;
use reqwest::header::CONTENT_TYPE;
use rss::{ChannelBuilder, Item};
use axum::{
    body::Body, handler::HandlerWithoutStateExt, http::StatusCode, response::{IntoResponse, Response}, routing::get, Router
};

use crate::{csoonline::CSOSource, darkreading::DarkReadingSource, krebs::KrebsSource};

mod krebs;
mod darkreading;
mod csoonline;

trait RSSSource {
    async fn get(&self) -> Result<Vec<Item>, anyhow::Error>;
}

// Make our own error that wraps `anyhow::Error`.
struct AppError(anyhow::Error);

// Tell axum how to convert `AppError` into a response.
impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Something went wrong: {}", self.0),
        )
            .into_response()
    }
}

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
impl<E> From<E> for AppError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

async fn get_channel() -> Result<rss::Channel, Box<dyn std::error::Error>> {
    let cso_source = CSOSource::default();
    let darkreading_source = DarkReadingSource::default();
    let krebs_source = KrebsSource::default();

    let (cso, darkreading, krebs) = tokio::join!(
        cso_source.get(),
        darkreading_source.get(),
        krebs_source.get()
    );

    let cso = cso?;
    let darkreading = darkreading?;
    let krebs = krebs?;

    let mut items: Vec<Item> = cso.into_iter()
        .chain(darkreading)
        .chain(krebs)
        .collect();

    items.sort_by_key(|el| DateTime::parse_from_rfc2822(el.pub_date().unwrap_or_default()).unwrap_or_default());
    items.reverse();

    let channel = ChannelBuilder::default()
        .title("Rustfully syndicated")
        .items(items)
        .build();

    Ok(channel)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    // tracing_subscriber::fmt::init();

    // build our application with a route
    let app = Router::new()
        // `GET /` goes to `root`
        .route("/", get(root));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await?;
    axum::serve(listener, app).await?;
    Ok(())
}

// basic handler that responds with a static string
#[axum_macros::debug_handler]
async fn root() -> Response {
    match get_channel().await {
        Ok(channel) => {
            let mut response = (StatusCode::OK, channel.to_string()).into_response();
            response.headers_mut().insert(CONTENT_TYPE, "application/xml".parse().unwrap());
            response
    },
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, "Failed to fetch channel".to_string()).into_response(),
    }
}
