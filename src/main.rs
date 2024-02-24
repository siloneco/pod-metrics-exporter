extern crate env_logger;
extern crate log;

use std::sync::{Arc, Mutex};

use std::fs;

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::Response;
use axum::routing::get;
use axum::Router;
use reqwest::header;

use std::time::Instant;
use tokio::time::{sleep, Duration};

use std::error::Error;

use log::{debug, error, info, warn};

const METRICS_URL: &str = "https://kubernetes.default.svc/apis/metrics.k8s.io/v1beta1/pods";

const CPU_METRIC_NAME: &str = "kube_pod_container_resource_usage_cpu";
const MEMORY_METRIC_NAME: &str = "kube_pod_container_resource_usage_memory";

const BIND_ADDRESS: &str = "0.0.0.0:3000";

const USE_TEST_DATA: bool = false;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();

    let metrics: Arc<Mutex<String>> = Arc::new(Mutex::new(String::from("")));

    let app = Router::new()
        .route("/metrics", get(respond))
        .route("/healthz", get(healthz_handler))
        .with_state(metrics.clone());

    let listener = tokio::net::TcpListener::bind(BIND_ADDRESS).await.unwrap();

    info!("Listening on {}", listener.local_addr().unwrap());

    let server = axum::serve(listener, app);

    let (_result, _) = tokio::join!(
        async {
            if let Err(e) = server.await {
                error!("Server error: {:?}", e);
            }
        },
        timer(15, metrics.clone())
    );

    Ok(())
}

async fn healthz_handler() -> Result<Response, StatusCode> {
    Ok(Response::new("Health OK".into()))
}

async fn respond(State(metrics): State<Arc<Mutex<String>>>) -> Result<Response, StatusCode> {
    let val = metrics.lock().unwrap();
    Ok(Response::new(format!("{}", *val).into()))
}

async fn timer(seconds: u64, metrics: Arc<Mutex<String>>) {
    loop {
        let now = Instant::now();

        debug!("Updating metrics...");
        update_metrics(metrics.clone()).await.unwrap();
        debug!("Metrics updated in {:?}", now.elapsed());

        sleep(Duration::from_secs(seconds) - now.elapsed()).await;
    }
}

async fn update_metrics(metrics: Arc<Mutex<String>>) -> Result<(), Box<dyn Error>> {
    let result = if !USE_TEST_DATA {
        get_reqwest_client()
            .await
            .map_err(|e| {
                error!("Failed to fetch metrics: {:?}", e);
                e
            })
            .unwrap()
            .get(METRICS_URL)
            .send()
            .await
            .map_err(|e| {
                error!("Failed to fetch metrics: {:?}", e);
                e
            })
            .unwrap()
            .json::<serde_json::Value>()
            .await
            .map_err(|e| {
                error!("Failed to fetch metrics: {:?}", e);
                e
            })
            .unwrap()
    } else {
        get_test_data().await
    };

    let mut updated_value = String::from("");

    debug!("{}", result["items"]);
    let pods = result["items"].as_array().unwrap();

    for pod in pods {
        let namespace = pod["metadata"]["namespace"].as_str().unwrap();
        let pod_name = pod["metadata"]["name"].as_str().unwrap();

        let containers = pod["containers"].as_array().unwrap();

        for container in containers {
            let container_name = container["name"].as_str().unwrap();
            let cpu_usage = container["usage"]["cpu"].as_str().unwrap();
            let memory_usage = container["usage"]["memory"].as_str().unwrap();

            // convert to milli seconds
            let cpu_usage = if cpu_usage.ends_with("n") {
                cpu_usage.strip_suffix("n").unwrap().parse::<f64>().unwrap() / 1_000_000.0
            } else if cpu_usage.ends_with("u") {
                cpu_usage.strip_suffix("u").unwrap().parse::<f64>().unwrap() / 1_000.0
            } else if cpu_usage.ends_with("m") {
                cpu_usage.strip_suffix("m").unwrap().parse::<f64>().unwrap()
            } else {
                if cpu_usage.parse::<f64>().unwrap() == 0.0 {
                    0.0
                } else {
                    warn!("Unknown cpu unit: {}", cpu_usage);
                    0.0
                }
            };

            // convert to MiB
            let memory_usage = if memory_usage.ends_with("Ki") {
                memory_usage
                    .strip_suffix("Ki")
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    / 1024.0
            } else if memory_usage.ends_with("Mi") {
                memory_usage
                    .strip_suffix("Mi")
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
            } else if memory_usage.ends_with("Gi") {
                memory_usage
                    .strip_suffix("Gi")
                    .unwrap()
                    .parse::<f64>()
                    .unwrap()
                    * 1024.0
            } else {
                warn!("Unknown memory unit: {}", memory_usage);
                0.0
            };

            updated_value.push_str(&format!(
                "{}{{namespace=\"{}\", pod=\"{}\", container=\"{}\"}} {}\n",
                CPU_METRIC_NAME, namespace, pod_name, container_name, cpu_usage
            ));

            updated_value.push_str(&format!(
                "{}{{namespace=\"{}\", pod=\"{}\", container=\"{}\"}} {}\n",
                MEMORY_METRIC_NAME, namespace, pod_name, container_name, memory_usage
            ));
        }
    }

    let mut val = metrics.lock().unwrap();
    *val = updated_value;

    Ok(())
}

async fn get_reqwest_client() -> Result<reqwest::Client, Box<dyn Error>> {
    let token = fs::read_to_string("/var/run/secrets/kubernetes.io/serviceaccount/token")?;
    let bearer_token: &str = &format!("Bearer {}", token);

    let mut headers = header::HeaderMap::new();
    headers.insert(
        "Authorization",
        header::HeaderValue::from_str(bearer_token).unwrap(),
    );

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()?;

    Ok(client)
}

async fn get_test_data() -> serde_json::Value {
    serde_json::from_str("{\"items\": []}").unwrap()
}
