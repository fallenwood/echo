mod echo_request;

use axum::{
  routing::{get},
  http::{StatusCode, HeaderValue, Request},
  response::{Response},
  Router, extract::{Query}, middleware::{Next, self},
};
use chrono::Utc;
use echo_request::EchoRequest;
use uuid::Uuid;
use std::net::SocketAddr;
use tokio::time::{sleep, Duration};

const X_RESPONSE_TIME : &'static str = "x-response-time";
const X_REQUEST_ID : &'static str = "x-request-id";

const HELP: &'static str = r#"{
  "Query": {
    "Status": "Optional Int, return 500 if status is less than 100 or greater than 600",
    "Timeout": "Optional Uint, in milliseconds",
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

  let app = Router::new()
      .route("/", get(get_echo))
      .route("/help", get(root))
      .route_layer(middleware::from_fn(populate_response_time))
      .route_layer(middleware::from_fn(populate_request_id));
 
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
    timeout as u64
  };
  
  sleep(Duration::from_millis(real_timeout)).await;

  (StatusCode::from_u16(status).unwrap(), "")
}

async fn populate_request_id<T>(req: Request<T>, next: Next<T>) -> Response {
  let request_id = match req.headers().get(X_REQUEST_ID) {
    Some(v) => v.to_owned(),
    None => HeaderValue::from_str(Uuid::new_v4().to_string().as_str()).unwrap(),
  };

  let mut response = next.run(req).await;

  response.headers_mut().insert(X_REQUEST_ID, request_id);

  response
}

async fn populate_response_time<T>(req: Request<T>, next: Next<T>) -> Response {
  let start_time = Utc::now();

  let mut response = next.run(req).await;

  let end_time = Utc::now();

  let interval = end_time - start_time;
  let response_time = HeaderValue::from_str(interval.num_milliseconds().to_string().as_str()).unwrap();

  response.headers_mut().insert(X_RESPONSE_TIME, response_time);
  response
}
