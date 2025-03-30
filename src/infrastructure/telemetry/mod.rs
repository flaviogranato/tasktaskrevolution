use metrics_exporter_prometheus::PrometheusBuilder;
use tracing_subscriber::{fmt, EnvFilter};

/// Inicializa o sistema de logging e métricas
pub fn init() {
    // Configuração do logging
    let filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| EnvFilter::new("info"));
    
    fmt()
        .with_env_filter(filter)
        .with_target(false)
        .with_thread_ids(false)
        .with_thread_names(false)
        .with_ansi(true)
        .init();

    // Configuração das métricas
    PrometheusBuilder::new()
        .with_http_listener("127.0.0.1:9090")
        .install()
        .expect("Falha ao instalar o exportador de métricas");
}

/// Registra métricas de performance
pub fn record_operation_duration(operation: &str, duration: std::time::Duration) {
    metrics::histogram!(
        "operation_duration_seconds",
        duration.as_secs_f64(),
        "operation" => operation.to_string()
    );
}

/// Registra contadores de operações
pub fn increment_operation_counter(operation: &str) {
    metrics::counter!("operation_total", 1, "operation" => operation.to_string());
} 