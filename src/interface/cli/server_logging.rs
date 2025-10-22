use std::time::{Duration, Instant};
use tracing::{info, warn, error, debug, Level};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

/// Structured logging for the server
pub struct ServerLogger {
    start_time: Instant,
    request_count: std::sync::atomic::AtomicU64,
}

impl ServerLogger {
    pub fn new() -> Self {
        Self {
            start_time: Instant::now(),
            request_count: std::sync::atomic::AtomicU64::new(0),
        }
    }

    /// Initialize structured logging
    pub fn init(debug: bool, json_logs: bool) -> Result<(), Box<dyn std::error::Error>> {
        let filter = if debug {
            EnvFilter::new("debug")
        } else {
            EnvFilter::new("info")
        };

        if json_logs {
            tracing_subscriber::registry()
                .with(filter)
                .with(tracing_subscriber::fmt::layer().json())
                .init();
        } else {
            tracing_subscriber::registry()
                .with(filter)
                .with(
                    tracing_subscriber::fmt::layer()
                        .with_target(false)
                        .with_thread_ids(true)
                        .with_thread_names(true)
                )
                .init();
        }

        Ok(())
    }

    /// Log server startup
    pub fn log_server_start(&self, host: &str, port: u16, directory: &str, live_reload: bool, cors: bool) {
        info!(
            host = %host,
            port = %port,
            directory = %directory,
            live_reload = %live_reload,
            cors = %cors,
            "Server started"
        );
    }

    /// Log server shutdown
    pub fn log_server_shutdown(&self) {
        let uptime = self.start_time.elapsed();
        let total_requests = self.request_count.load(std::sync::atomic::Ordering::Relaxed);
        
        info!(
            uptime_seconds = %uptime.as_secs(),
            total_requests = %total_requests,
            "Server shutting down"
        );
    }

    /// Log HTTP request
    pub fn log_request(&self, method: &str, path: &str, status: u16, duration: Duration) {
        self.request_count.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        
        let level = if status >= 500 {
            Level::ERROR
        } else if status >= 400 {
            Level::WARN
        } else {
            Level::INFO
        };

        match level {
            Level::ERROR => error!(
                method = %method,
                path = %path,
                status = %status,
                duration_ms = %duration.as_millis(),
                "HTTP request"
            ),
            Level::WARN => warn!(
                method = %method,
                path = %path,
                status = %status,
                duration_ms = %duration.as_millis(),
                "HTTP request"
            ),
            _ => info!(
                method = %method,
                path = %path,
                status = %status,
                duration_ms = %duration.as_millis(),
                "HTTP request"
            ),
        }
    }

    /// Log WebSocket connection
    pub fn log_websocket_connect(&self, client_id: &str, total_clients: usize) {
        info!(
            client_id = %client_id,
            total_clients = %total_clients,
            "WebSocket client connected"
        );
    }

    /// Log WebSocket disconnection
    pub fn log_websocket_disconnect(&self, client_id: &str, total_clients: usize) {
        info!(
            client_id = %client_id,
            total_clients = %total_clients,
            "WebSocket client disconnected"
        );
    }

    /// Log file change detection
    pub fn log_file_change(&self, path: &str, event_type: &str) {
        debug!(
            file_path = %path,
            event_type = %event_type,
            "File change detected"
        );
    }

    /// Log reload broadcast
    pub fn log_reload_broadcast(&self, client_count: usize) {
        info!(
            client_count = %client_count,
            "Reload signal broadcasted"
        );
    }

    /// Log error with context
    pub fn log_error(&self, error: &str, context: &str) {
        error!(
            error = %error,
            context = %context,
            "Server error"
        );
    }

    /// Log warning with context
    pub fn log_warning(&self, warning: &str, context: &str) {
        warn!(
            warning = %warning,
            context = %context,
            "Server warning"
        );
    }

    /// Log performance metrics
    pub fn log_performance_metrics(&self) {
        let uptime = self.start_time.elapsed();
        let total_requests = self.request_count.load(std::sync::atomic::Ordering::Relaxed);
        let requests_per_second = if uptime.as_secs() > 0 {
            total_requests as f64 / uptime.as_secs() as f64
        } else {
            0.0
        };

        info!(
            uptime_seconds = %uptime.as_secs(),
            total_requests = %total_requests,
            requests_per_second = %requests_per_second,
            "Performance metrics"
        );
    }
}

impl Clone for ServerLogger {
    fn clone(&self) -> Self {
        Self {
            start_time: self.start_time,
            request_count: std::sync::atomic::AtomicU64::new(self.request_count.load(std::sync::atomic::Ordering::Relaxed)),
        }
    }
}

impl Default for ServerLogger {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;

    #[test]
    fn test_server_logger_creation() {
        let logger = ServerLogger::new();
        assert_eq!(logger.request_count.load(std::sync::atomic::Ordering::Relaxed), 0);
    }

    #[test]
    fn test_log_request() {
        let logger = ServerLogger::new();
        logger.log_request("GET", "/index.html", 200, Duration::from_millis(10));
        assert_eq!(logger.request_count.load(std::sync::atomic::Ordering::Relaxed), 1);
    }

    #[test]
    fn test_log_websocket_events() {
        let logger = ServerLogger::new();
        logger.log_websocket_connect("client-123", 1);
        logger.log_websocket_disconnect("client-123", 0);
        // These should not panic
    }

    #[test]
    fn test_log_file_change() {
        let logger = ServerLogger::new();
        logger.log_file_change("/path/to/file.html", "modified");
        // Should not panic
    }
}
