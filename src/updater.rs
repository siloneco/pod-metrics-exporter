extern crate env_logger;
extern crate log;

use axum::http::{HeaderMap, HeaderValue};
use log::{debug, warn};
use reqwest::Client;
use std::{
    error::Error,
    fs,
    sync::{Arc, Mutex},
};

use crate::{
    config::debug_enabled,
    constant::{CPU_METRIC_NAME, MEMORY_METRIC_NAME, METRICS_URL, TOKEN_PATH},
    unit_converter::{parse_cpu_value, parse_memory_value},
};

pub async fn update_metrics(metrics: Arc<Mutex<String>>) -> Result<(), String> {
    // Generate new metrics and handle errors
    let new_metrics = match generate_new_metrics().await {
        Ok(new_metrics) => new_metrics,
        Err(e) => {
            return Err(format!("Failed to generate new metrics: {}", e));
        }
    };

    // Lock the metrics and handle errors
    let mut value = match metrics.lock() {
        Ok(value) => value,
        Err(e) => {
            return Err(format!("Failed to modify cache metrics: {}", e));
        }
    };

    // Update the metrics
    *value = new_metrics;

    Ok(())
}

pub async fn generate_new_metrics() -> Result<String, String> {
    let result = match get_raw_json_data().await {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("Failed to update metrics: {}", e));
        }
    };

    let mut values: Vec<String> = vec![];

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

            let cpu_usage = match parse_cpu_value(cpu_usage) {
                Ok(cpu_usage) => cpu_usage,
                Err(e) => {
                    warn!("Failed to parse CPU value: {}", e);
                    0.0
                }
            };
            let memory_usage = match parse_memory_value(memory_usage) {
                Ok(memory_usage) => memory_usage,
                Err(e) => {
                    warn!("Failed to parse memory value: {}", e);
                    0.0
                }
            };

            let cpu_metrics_str = format!(
                "{}{{namespace=\"{}\", pod=\"{}\", container=\"{}\"}} {}\n",
                CPU_METRIC_NAME, namespace, pod_name, container_name, cpu_usage
            );
            let memory_metrics_str = format!(
                "{}{{namespace=\"{}\", pod=\"{}\", container=\"{}\"}} {}\n",
                MEMORY_METRIC_NAME, namespace, pod_name, container_name, memory_usage
            );

            values.push(cpu_metrics_str);
            values.push(memory_metrics_str);
        }
    }

    Ok(values.join("\n"))
}

async fn get_raw_json_data() -> Result<serde_json::Value, String> {
    if debug_enabled() {
        return Ok(get_test_data());
    }

    let client = match get_reqwest_client() {
        Ok(client) => client,
        Err(e) => {
            return Err(format!("Unable to create reqwest client: {}", e));
        }
    };

    let result = match client.get(METRICS_URL).send().await {
        Ok(result) => result,
        Err(e) => {
            return Err(format!("Unable to fetch result from URL: {}", e));
        }
    };

    let result_json = match result.json::<serde_json::Value>().await {
        Ok(result_json) => result_json,
        Err(e) => {
            return Err(format!("Unable to parse JSON: {}", e));
        }
    };

    Ok(result_json)
}

fn get_reqwest_client() -> Result<Client, Box<dyn Error>> {
    let token = fs::read_to_string(TOKEN_PATH)?;
    let bearer_token: &str = &format!("Bearer {}", token);

    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(bearer_token).unwrap(),
    );

    let client = Client::builder()
        .default_headers(headers)
        .danger_accept_invalid_certs(true)
        .build()?;

    Ok(client)
}

fn get_test_data() -> serde_json::Value {
    serde_json::from_str("{\"items\": []}").unwrap()
}
