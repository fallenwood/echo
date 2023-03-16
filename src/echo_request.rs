use serde::{Deserialize};

#[derive(Deserialize)]
pub struct EchoRequest {
  pub status: Option<i32>,
  pub timeout: Option<i64>,
  pub delay: Option<i64>,
}
