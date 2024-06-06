extern crate env_logger;
extern crate log;

use axum::{routing::get, serve::Serve, Router};
use constant::{BIND_ADDRESS, BIND_PORT};
use handler::{healthz_handler, metrics_handler};
use log::{debug, error, info, warn};
use std::{
    sync::{Arc, Mutex},
    time::Instant,
};
use tokio::{
    net::TcpListener,
    time::{sleep, Duration},
};
use updater::update_metrics;

mod config;
mod constant;
mod handler;
mod unit_converter;
mod updater;

#[tokio::main]
async fn main() {
    env_logger::init();

    let metrics: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("")));

    let app = Router::new()
        .route("/metrics", get(metrics_handler))
        .route("/healthz", get(healthz_handler))
        .with_state(metrics.clone());

    let address = format!("{}:{}", BIND_ADDRESS, BIND_PORT);
    let listener = TcpListener::bind(address).await.unwrap();

    info!("Listening on {}", listener.local_addr().unwrap());

    tokio::join!(
        axum_task(axum::serve(listener, app)),
        update_metrics_task(metrics)
    );
}

async fn axum_task(server: Serve<Router, Router>) {
    if let Err(e) = server.await {
        error!("Server error: {:?}", e);
    }
}

async fn update_metrics_task(metrics: Arc<Mutex<String>>) {
    {
        loop {
            let start_time = Instant::now();

            debug!("Updating metrics...");

            let update_result = update_metrics(metrics.clone()).await;

            if update_result.is_ok() {
                debug!("Metrics updated in {:?}", start_time.elapsed());
            } else {
                warn!("Failed to update metrics: {}", update_result.unwrap_err());
            }

            let sleep_duration = if Duration::from_secs(15) > start_time.elapsed() {
                Duration::from_secs(15) - start_time.elapsed()
            } else {
                Duration::from_secs(0)
            };

            sleep(sleep_duration).await;
        }
    }
}
