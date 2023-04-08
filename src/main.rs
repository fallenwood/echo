mod echo_request;
mod middleware;

use axum::{
  extract::{ConnectInfo, Query},
  http::{HeaderMap, HeaderValue, StatusCode},
  response::IntoResponse,
  routing::get,
  Router,
};
use echo_request::EchoRequest;
use middleware::{populate_request_id, populate_response_time};
use std::{cmp::min, net::SocketAddr};
use tokio::time::{sleep, Duration};
use tower::{buffer::BufferLayer, limit::ConcurrencyLimitLayer, ServiceBuilder};
use tower_http::{
  trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
  LatencyUnit,
};
use tracing::Level;

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
      "X-Client-IP: "Client IP Addrss",
      "X-Response-Time": "Response time, in milliseconds"
    }
  }
}"#;

const X_CLIENT_IP: &'static str = "X-Client-iP";
const X_FORWARD_IP: &'static str = "x-forwarded-for";
const X_REAL_IP: &'static str = "x-real-ip";

fn create_app() -> Router {
  let app = Router::new()
    .route("/", get(get_echo))
    .route("/help", get(help))
    .layer(axum::middleware::from_fn(populate_request_id))
    .layer(
      ServiceBuilder::new()
        .layer(axum::error_handling::HandleErrorLayer::new(
          |err: tower::BoxError| async move {
            (
              StatusCode::INTERNAL_SERVER_ERROR,
              format!("Unhandled error: {}", err),
            )
          },
        ))
        .layer(BufferLayer::new(4096))
        .layer(ConcurrencyLimitLayer::new(200)), // .layer(RateLimitLayer::new(25, Duration::from_secs(5)))
    )
    .layer(
      TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
          DefaultOnResponse::new()
            .level(Level::INFO)
            .latency_unit(LatencyUnit::Micros),
        ),
    )
    .layer(axum::middleware::from_fn(populate_response_time))
    .route("/healthz", get(health));

  app
}

#[tokio::main]
async fn main() {
  env_logger::init();

  let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
  let app = create_app();

  tracing::info!("listening on {}", addr);

  axum::Server::bind(&addr)
    .serve(app.into_make_service_with_connect_info::<SocketAddr>())
    .await
    .unwrap();
}

// basic handler that responds with a static string
async fn help() -> &'static str {
  HELP
}

pub async fn health() -> StatusCode {
  StatusCode::OK
}

async fn get_echo(
  headers: HeaderMap,
  Query(query): Query<EchoRequest>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> impl IntoResponse {
  let status = query.status.unwrap_or(200) as u16;
  let timeout = query.timeout.or(query.delay).unwrap_or_default();
  let real_timeout = if timeout < 0 {
    0
  } else {
    min(timeout, 120_000) as u64
  };

  let real_ip = match headers.get(X_REAL_IP) {
    Some(ip) => ip.to_owned(),
    None => match headers.get(X_FORWARD_IP) {
      Some(ip) => ip.to_owned(),
      None => HeaderValue::from_str(addr.ip().to_string().as_str()).unwrap(),
    },
  };

  sleep(Duration::from_millis(real_timeout)).await;

  (
    StatusCode::from_u16(status).unwrap(),
    [(X_CLIENT_IP, real_ip)],
    "",
  )
}

#[cfg(test)]
mod tests {
  use std::net::SocketAddr;

use axum::{
    body::{Body},
    http::{Request, StatusCode},
    response::IntoResponse, extract::connect_info::MockConnectInfo,
  };
  use tower::ServiceExt;

  use crate::{create_app, middleware::X_RESPONSE_TIME};

  #[tokio::test]
  async fn test_health() {
    let response = crate::health().await.into_response();

    assert!(response.status() == StatusCode::OK);
  }

  #[tokio::test]
  async fn test_help() {
    let response = crate::help().await.into_response();

    assert!(response.status() == StatusCode::OK);
  }

  #[tokio::test]
  async fn test_app_healthz() {
    let app = create_app();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/healthz")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers();
    assert!(header.get(X_RESPONSE_TIME).is_none());
  }

  #[tokio::test]
  async fn test_app_help() {
    let app = create_app();

    let response = app
      .oneshot(
        Request::builder()
          .uri("/help")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers();
    assert!(header.get(X_RESPONSE_TIME).is_some());
  }

  #[tokio::test]
  async fn test_app_200() {
    let app = create_app()
      .layer(MockConnectInfo(SocketAddr::from(([192, 1, 1, 1], 12345))));

    let response = app
      .oneshot(
        Request::builder()
          .uri("/")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
  }

  #[tokio::test]
  async fn test_app_404() {
    let app = create_app()
      .layer(MockConnectInfo(SocketAddr::from(([192, 1, 1, 1], 12345))));

    let response = app
      .oneshot(
        Request::builder()
          .uri("/?status=404")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
  }

  #[tokio::test]
  async fn test_app_delay() {
    let app = create_app()
      .layer(MockConnectInfo(SocketAddr::from(([192, 1, 1, 1], 12345))));

    let response = app
      .oneshot(
        Request::builder()
          .uri("/?delay=100")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::OK);
    let header = response.headers();
    let latency = header.get(X_RESPONSE_TIME).unwrap().to_str().unwrap().parse::<i32>().unwrap();

    assert!(latency >= 100);
  }
}
