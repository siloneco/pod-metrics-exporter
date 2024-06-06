pub const METRICS_URL: &str = "https://kubernetes.default.svc/apis/metrics.k8s.io/v1beta1/pods";
pub const TOKEN_PATH: &str = "/var/run/secrets/kubernetes.io/serviceaccount/token";

pub const CPU_METRIC_NAME: &str = "kube_pod_container_resource_usage_cpu";
pub const MEMORY_METRIC_NAME: &str = "kube_pod_container_resource_usage_memory";

pub const BIND_ADDRESS: &str = "0.0.0.0";
pub const BIND_PORT: u16 = 3000;
