mod echo_request;
mod middleware;

use axum::{
  body::Body, extract::{ConnectInfo, Query}, http::{HeaderMap, HeaderValue, StatusCode}, response::IntoResponse, routing::{get, post, put}, Router
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
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;
use tracing::Level;
use mimalloc::MiMalloc;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

#[derive(OpenApi)]
#[openapi(
  paths(get_echo),
  components(
    schemas(EchoRequest),
  ),
)]
struct EchoOpenApi;

const X_CLIENT_IP: &'static str = "X-Client-iP";
const X_FORWARD_IP: &'static str = "x-forwarded-for";
const X_REAL_IP: &'static str = "x-real-ip";
const CONTENT_TYPE: &'static str = "content-type";
const USER_AGENT: &'static str = "User-Agent";
const X_CLIENT_USER_AGENT: &'static str = "x-client-user-agent";

fn create_app() -> Router {
  let swagger = SwaggerUi::new("/swagger")
    .url("/api-doc/openapi.json", EchoOpenApi::openapi());

  let app = Router::new()
    .route("/", get(get_echo))
    .route("/", post(post_put_echo))
    .route("/", put(post_put_echo))
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
    .route("/healthz", get(health))
    .merge(swagger);

  app
}

#[tokio::main]
async fn main() {
  env_logger::init();

  let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
  let app = create_app();

  tracing::info!("listening on {}", addr);

  let listener = tokio::net::TcpListener::bind(addr).await.unwrap();

  axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>())
  .await
  .unwrap();
}

pub async fn health() -> StatusCode {
  StatusCode::OK
}

#[utoipa::path(
  get,
  path = "/",
  params(
    ("status" = Option<i32>, Query, description = "Http Status Code"),
    ("delay" = Option<i64>, Query, description = "The delay time in milliseconds"),
    ("timeout" = Option<i64>, Query, description = "The delay time in milliseconds, higher priority than delay"),
  ),
)]
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

  let ua = match headers.get(USER_AGENT) {
    Some(s) => s.to_owned(),
    None => HeaderValue::from_static(""),
  };

  sleep(Duration::from_millis(real_timeout)).await;

  (
    StatusCode::from_u16(status).unwrap(),
    [
      (X_CLIENT_IP, real_ip),
      (X_CLIENT_USER_AGENT, ua),
    ],
    "",
  )
}

#[utoipa::path(
  post,
  path = "/",
  params(
    ("status" = Option<i32>, Query, description = "Http Status Code"),
    ("delay" = Option<i64>, Query, description = "The delay time in milliseconds"),
    ("timeout" = Option<i64>, Query, description = "The delay time in milliseconds, higher priority than delay"),
    ("content-type" = Option<String>, Header, description = "The response content type"),
    // ("body" = Option<String>, Body, description = "The request body"),
  ),
)]
async fn post_put_echo(
  headers: HeaderMap,
  Query(query): Query<EchoRequest>,
  ConnectInfo(addr): ConnectInfo<SocketAddr>,
  body: Body,
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

  let content_type = match headers.get(CONTENT_TYPE) {
    Some(s) => s.to_owned(),
    None => HeaderValue::from_static("text/plain"),
  };

  let ua = match headers.get(USER_AGENT) {
    Some(s) => s.to_owned(),
    None => HeaderValue::from_static(""),
  };

  sleep(Duration::from_millis(real_timeout)).await;

  (
    StatusCode::from_u16(status).unwrap(),
    [
      (X_CLIENT_IP, real_ip),
      (CONTENT_TYPE, content_type),
      (X_CLIENT_USER_AGENT, ua),
    ],
    body,
  )
}


#[cfg(test)]
mod tests {
  use std::net::SocketAddr;

use axum::{
    body::Body,
    http::{Request, StatusCode},
    response::IntoResponse, extract::connect_info::MockConnectInfo,
  };
  use tower::ServiceExt;

  use crate::{create_app, middleware::X_RESPONSE_TIME, X_CLIENT_USER_AGENT};

  #[tokio::test]
  async fn test_health() {
    let response = crate::health().await.into_response();

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
          .header("user-agent", "unittests/0.0")
          .body(Body::empty())
          .unwrap(),
      )
      .await
      .unwrap();

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
    assert_eq!(response.headers().get(X_CLIENT_USER_AGENT).unwrap().to_owned(), "unittests/0.0");
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
