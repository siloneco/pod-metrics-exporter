use std::sync::{Arc, Mutex};

use axum::{extract::State, response::Response};
use reqwest::StatusCode;

pub async fn healthz_handler() -> Result<Response, StatusCode> {
    Ok(Response::new("Health OK".into()))
}

pub async fn metrics_handler(
    State(metrics): State<Arc<Mutex<String>>>,
) -> Result<Response, StatusCode> {
    let val = metrics.lock().unwrap();
    Ok(Response::new(format!("{}", *val).into()))
}
