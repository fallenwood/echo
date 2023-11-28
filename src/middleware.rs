use axum::{
  http::{HeaderValue, Request},
  middleware::Next,
  response::Response, body::Body,
};
use chrono::Utc;
use uuid::Uuid;

pub const X_RESPONSE_TIME: &'static str = "x-response-time";
pub const X_REQUEST_ID: &'static str = "x-request-id";

pub async fn populate_request_id(req: Request<Body>, next: Next) -> Response {
  let request_id = match req.headers().get(X_REQUEST_ID) {
    Some(v) => v.to_owned(),
    None => HeaderValue::from_str(Uuid::new_v4().to_string().as_str()).unwrap(),
  };

  let mut response = next.run(req).await;

  response.headers_mut().insert(X_REQUEST_ID, request_id);

  response
}

pub async fn populate_response_time(req: Request<Body>, next: Next) -> Response {
  let start_time = Utc::now();

  let mut response = next.run(req).await;

  let end_time = Utc::now();

  let interval = end_time - start_time;
  let response_time =
    HeaderValue::from_str(interval.num_milliseconds().to_string().as_str()).unwrap();

  response
    .headers_mut()
    .insert(X_RESPONSE_TIME, response_time);
  response
}
