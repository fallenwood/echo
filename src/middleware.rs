use axum::{
  extract::Query,
  http::{HeaderValue, Request, StatusCode},
  middleware::{self, Next},
  response::Response,
  routing::get,
  Router,
};
use chrono::Utc;
use uuid::Uuid;

const X_RESPONSE_TIME : &'static str = "x-response-time";
const X_REQUEST_ID : &'static str = "x-request-id";

pub async fn populate_request_id<T>(req: Request<T>, next: Next<T>) -> Response {
  let request_id = match req.headers().get(X_REQUEST_ID) {
    Some(v) => v.to_owned(),
    None => HeaderValue::from_str(Uuid::new_v4().to_string().as_str()).unwrap(),
  };

  let mut response = next.run(req).await;

  response.headers_mut().insert(X_REQUEST_ID, request_id);

  response
}

pub async fn populate_response_time<T>(req: Request<T>, next: Next<T>) -> Response {
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
