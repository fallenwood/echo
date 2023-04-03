mod echo_request;
mod middleware;

use axum::{
  routing::{get},
  http::{StatusCode},
  Router, extract::{Query}
};
use echo_request::EchoRequest;
use tower::{limit::{RateLimitLayer, ConcurrencyLimitLayer}, ServiceBuilder, buffer::BufferLayer};
use std::{net::SocketAddr, cmp::min};
use tokio::time::{sleep, Duration};
use middleware::{populate_request_id, populate_response_time};

const HELP: &'static str = r#"{
  "Query": {
    "Status": "Optional Int, return 500 if status is less than 100 or greater than 600",
    "Timeout": "Optional Uint, in milliseconds, max value is 120s",
    "Delay": "Optional Uint, in milliseconds, has lower priority than Timeout",
    "Headers": {
      "X-Request-Id": "Request Id",
    }
  },
  "Response": {
    "Status": "Status",
    "Headers": {
      "X-Request-Id": "Request Id",
      "X-Response-Time": "Response time, in milliseconds"
    }
  }
}"#;

#[tokio::main]
async fn main() {
  tracing_subscriber::fmt::init();

  let mut app = Router::new()
      .route("/", get(get_echo))
      .route("/help", get(root))
      .layer(axum::middleware::from_fn(populate_request_id))
      .layer(
        ServiceBuilder::new()
          .layer(axum::error_handling::HandleErrorLayer::new(|err: tower::BoxError| async move {
            (
              StatusCode::INTERNAL_SERVER_ERROR,
              format!("Unhandled error: {}", err),
            )
          }))
          .layer(BufferLayer::new(4096))
          .layer(ConcurrencyLimitLayer::new(200))
          // .layer(RateLimitLayer::new(25, Duration::from_secs(5)))
        )
      .layer(axum::middleware::from_fn(populate_response_time));

  let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
  tracing::debug!("listening on {}", addr);
  axum::Server::bind(&addr)
      .serve(app.into_make_service())
      .await
      .unwrap();
}

// basic handler that responds with a static string
async fn root() -> &'static str {
  HELP
}

async fn get_echo(
  Query(query): Query<EchoRequest>
) -> (StatusCode, &'static str) {
  let status = query.status.unwrap_or(200) as u16;
  let timeout = query.timeout.or(query.delay).unwrap_or_default();
  let real_timeout = if timeout < 0 {
    0
  } else {
    min(timeout, 120_000) as u64
  };

  sleep(Duration::from_millis(real_timeout)).await;

  (StatusCode::from_u16(status).unwrap(), "")
}
