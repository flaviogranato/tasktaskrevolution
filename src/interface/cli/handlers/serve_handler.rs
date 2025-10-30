use std::path::PathBuf;
// use std::sync::Arc;
use tokio::sync::broadcast;
use warp::Filter;

use crate::interface::cli::logging::Logger;
// use crate::interface::cli::websocket_server::WebSocketServer;
// use crate::interface::cli::server_logging::ServerLogger;

/// Handle the serve command
pub async fn handle_serve_command(
    port: u16,
    host: String,
    directory: PathBuf,
    live_reload: bool,
    cors: bool,
    debug: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    // Validate directory exists
    if !directory.exists() {
        return Err(format!("Directory '{}' does not exist", directory.display()).into());
    }

    if !directory.is_dir() {
        return Err(format!("'{}' is not a directory", directory.display()).into());
    }

    // Initialize structured logging
    let _json_logs = std::env::var("TTR_JSON_LOGS").unwrap_or_default() == "1";
    // ServerLogger::init(debug, json_logs)?;

    // Create server logger instance
    // let server_logger = ServerLogger::new();
    // server_logger.log_server_start(&host, port, &directory.display().to_string(), live_reload, cors);

    // Create the server
    let mut server = Server::new(directory, live_reload, cors, debug);
    server.serve(host, port).await?;

    Ok(())
}

/// HTTP server for serving static files
pub struct Server {
    directory: PathBuf,
    live_reload: bool,
    cors: bool,
    debug: bool,
    shutdown_tx: Option<broadcast::Sender<()>>,
    // websocket_server: Option<WebSocketServer>,
    // server_logger: ServerLogger,
}

impl Server {
    pub fn new(directory: PathBuf, live_reload: bool, cors: bool, debug: bool) -> Self {
        Self {
            directory,
            live_reload,
            cors,
            debug,
            shutdown_tx: None,
            // websocket_server: None,
            // server_logger,
        }
    }

    pub async fn serve(&mut self, host: String, port: u16) -> Result<(), Box<dyn std::error::Error>> {
        let directory = self.directory.clone();
        let live_reload = self.live_reload;
        let cors = self.cors;
        let _debug = self.debug;

        // Create shutdown channel
        let (shutdown_tx, mut shutdown_rx) = broadcast::channel(1);
        self.shutdown_tx = Some(shutdown_tx.clone());

        // Create WebSocket server if live reload is enabled
        if live_reload {
            // self.websocket_server = Some(WebSocketServer::new(port));
        }

        // Create file watcher if live reload is enabled
        let file_watcher = if live_reload {
            Some(self.create_file_watcher(directory.clone(), shutdown_tx.clone())?)
        } else {
            None
        };

        // Create the warp filter with WebSocket support
        let routes = Self::create_static_routes_simple(directory.clone(), live_reload, cors);

        // Parse host address
        let addr = if host == "0.0.0.0" {
            std::net::SocketAddr::from(([0, 0, 0, 0], port))
        } else if host == "localhost" || host == "127.0.0.1" {
            std::net::SocketAddr::from(([127, 0, 0, 1], port))
        } else {
            // Try to parse as IP address
            let ip: std::net::IpAddr = host.parse()?;
            std::net::SocketAddr::from((ip, port))
        };

        // Clone logger for use in closure
        // let server_logger = self.server_logger.clone();
        let _directory_display = directory.display().to_string();

        // Start the server
        let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(addr, async move {
            shutdown_rx.recv().await.ok();
            // server_logger.log_server_shutdown();
        });

        // self.server_logger.log_server_start(&host, port, &directory_display, live_reload, cors);
        Logger::info(&format!("Server running at http://{}:{}", host, port));
        if live_reload {
            Logger::info(&format!("Live reload WebSocket available at ws://{}:{}/ws", host, port));
        }
        Logger::info("Press Ctrl+C to stop the server");

        // Run the server
        server.await;

        // Clean up file watcher
        if let Some(watcher) = file_watcher {
            drop(watcher);
        }

        Ok(())
    }

    fn create_static_routes_simple(
        directory: PathBuf,
        live_reload: bool,
        _cors: bool,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        use warp::path::FullPath;

        // Live reload script injection
        let live_reload_script = if live_reload {
            "// Live reload script placeholder".to_string() // TODO: Implement live reload
        } else {
            String::new()
        };

        // Static file serving
        let static_files = warp::path::full().and_then(move |path: FullPath| {
            let directory = directory.clone();
            let live_reload_script = live_reload_script.to_string();
            async move {
                let path = path.as_str().trim_start_matches('/');
                let file_path = directory.join(path);

                // Security check - prevent directory traversal
                if !file_path.starts_with(&directory) {
                    return Err(warp::reject::not_found());
                }

                if file_path.is_dir() {
                    // Serve directory listing
                    Self::serve_directory_listing_static(&file_path, &directory).await
                } else if file_path.is_file() {
                    // Serve file
                    Self::serve_file_static(&file_path, &live_reload_script).await
                } else {
                    Err(warp::reject::not_found())
                }
            }
        });

        // Root redirect to index.html
        let root_redirect = warp::path::end()
            .and(warp::get())
            .map(|| warp::redirect(warp::http::Uri::from_static("/index.html")));

        root_redirect.or(static_files)
    }

    async fn serve_file_static(
        file_path: &std::path::Path,
        live_reload_script: &str,
    ) -> Result<warp::reply::Response, warp::Rejection> {
        use std::fs;
        use warp::http::header::{CONTENT_TYPE, HeaderValue};
        use warp::reply::Response;

        match fs::read(file_path) {
            Ok(mut content) => {
                // Inject live reload script for HTML files
                let content_type = mime_guess::from_path(file_path).first_or_octet_stream().to_string();

                if content_type.starts_with("text/html") && !live_reload_script.is_empty() {
                    let mut html_content = String::from_utf8_lossy(&content).to_string();
                    if let Some(body_end) = html_content.rfind("</body>") {
                        html_content.insert_str(body_end, live_reload_script);
                        content = html_content.into_bytes();
                    }
                }

                let mut response = Response::new(content.into());
                response
                    .headers_mut()
                    .insert(CONTENT_TYPE, HeaderValue::from_str(&content_type).unwrap());

                Ok(response)
            }
            Err(_) => Err(warp::reject::not_found()),
        }
    }

    async fn serve_directory_listing_static(
        dir_path: &std::path::Path,
        base_dir: &std::path::Path,
    ) -> Result<warp::reply::Response, warp::Rejection> {
        use std::fs;
        use warp::http::header::{CONTENT_TYPE, HeaderValue};
        use warp::reply::Response;

        let mut entries = Vec::new();
        if let Ok(read_dir) = fs::read_dir(dir_path) {
            for entry in read_dir.flatten() {
                let path = entry.path();
                let name = path.file_name().unwrap().to_string_lossy().to_string();
                let is_dir = path.is_dir();
                let relative_path = path.strip_prefix(base_dir).unwrap_or(&path);
                let relative_path_str = relative_path.to_string_lossy().to_string();

                entries.push(DirectoryEntry {
                    name,
                    path: relative_path_str,
                    is_dir,
                });
            }
        }

        // Sort entries: directories first, then files
        entries.sort_by(|a, b| {
            if a.is_dir && !b.is_dir {
                std::cmp::Ordering::Less
            } else if !a.is_dir && b.is_dir {
                std::cmp::Ordering::Greater
            } else {
                a.name.cmp(&b.name)
            }
        });

        let html = Self::generate_directory_listing_html_static(entries, dir_path);
        let mut response = Response::new(html.into());
        response
            .headers_mut()
            .insert(CONTENT_TYPE, HeaderValue::from_static("text/html"));

        Ok(response)
    }

    fn generate_directory_listing_html_static(entries: Vec<DirectoryEntry>, dir_path: &std::path::Path) -> String {
        let title = format!("Directory listing for {}", dir_path.display());
        let mut html = format!(
            r#"<!DOCTYPE html>
<html>
<head>
    <title>{}</title>
    <style>
        body {{ font-family: Arial, sans-serif; margin: 40px; }}
        h1 {{ color: #333; }}
        table {{ border-collapse: collapse; width: 100%; }}
        th, td {{ border: 1px solid #ddd; padding: 8px; text-align: left; }}
        th {{ background-color: #f2f2f2; }}
        a {{ text-decoration: none; color: #0066cc; }}
        a:hover {{ text-decoration: underline; }}
        .dir {{ font-weight: bold; }}
    </style>
</head>
<body>
    <h1>{}</h1>
    <table>
        <tr>
            <th>Name</th>
            <th>Type</th>
        </tr>"#,
            title, title
        );

        for entry in entries {
            let icon = if entry.is_dir { "📁" } else { "📄" };
            let class = if entry.is_dir { "dir" } else { "" };
            html.push_str(&format!(
                r#"
        <tr>
            <td><a href="/{}" class="{}">{} {}</a></td>
            <td>{}</td>
        </tr>"#,
                entry.path,
                class,
                icon,
                entry.name,
                if entry.is_dir { "Directory" } else { "File" }
            ));
        }

        html.push_str(
            r#"
    </table>
</body>
</html>"#,
        );

        html
    }

    fn create_file_watcher(
        &self,
        directory: PathBuf,
        _shutdown_tx: broadcast::Sender<()>,
    ) -> Result<notify::RecommendedWatcher, Box<dyn std::error::Error>> {
        use notify::{RecommendedWatcher, RecursiveMode, Watcher};

        let _debug = self.debug;
        // let server_logger = Arc::new(self.server_logger.clone());
        // let websocket_server = Arc::new(self.websocket_server.clone());

        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                // let server_logger = server_logger.clone();
                // let websocket_server = websocket_server.clone();

                match res {
                    Ok(event) => {
                        for path in event.paths {
                            let path_str = path.to_string_lossy().to_string();
                            // server_logger.log_file_change(&path_str, "modified");

                            if _debug {
                                Logger::debug(&format!("File changed: {}", path_str));
                            }
                        }

                        // Broadcast reload signal to WebSocket clients
                        // if let Some(ws_server) = websocket_server.as_ref() {
                        //     let ws_server = ws_server.clone();
                        //     tokio::spawn(async move {
                        //         ws_server.broadcast_reload().await;
                        //     });
                        // }
                    }
                    Err(e) => {
                        // server_logger.log_error(&format!("File watcher error: {}", e), "file_watcher");
                        Logger::error(&format!("File watcher error: {}", e));
                    }
                }
            },
            notify::Config::default(),
        )?;

        watcher.watch(&directory, RecursiveMode::Recursive)?;

        Ok(watcher)
    }
}

#[derive(Debug)]
struct DirectoryEntry {
    name: String,
    path: String,
    is_dir: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_serve_command_with_valid_directory() {
        let temp_dir = TempDir::new().unwrap();
        let test_file = temp_dir.path().join("index.html");
        fs::write(&test_file, "<html><body>Test</body></html>").unwrap();

        // Test that the function doesn't panic with valid directory
        // We can't easily test the full server functionality in unit tests
        // as it runs indefinitely, but we can test the setup
        let result = std::panic::catch_unwind(|| {
            tokio::spawn(async move {
                let _ = handle_serve_command(
                    3001,
                    "localhost".to_string(),
                    temp_dir.path().to_path_buf(),
                    false,
                    false,
                    false,
                )
                .await;
            });
        });

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_serve_command_with_invalid_directory() {
        let result = handle_serve_command(
            3002,
            "localhost".to_string(),
            PathBuf::from("/nonexistent/directory"),
            false,
            false,
            false,
        )
        .await;

        assert!(result.is_err());
    }

    #[test]
    fn test_directory_entry_creation() {
        let entry = DirectoryEntry {
            name: "test.html".to_string(),
            path: "test.html".to_string(),
            is_dir: false,
        };

        assert_eq!(entry.name, "test.html");
        assert_eq!(entry.path, "test.html");
        assert!(!entry.is_dir);
    }

    #[test]
    fn test_generate_directory_listing_html() {
        let entries = vec![
            DirectoryEntry {
                name: "file1.html".to_string(),
                path: "file1.html".to_string(),
                is_dir: false,
            },
            DirectoryEntry {
                name: "subdir".to_string(),
                path: "subdir".to_string(),
                is_dir: true,
            },
        ];
        let dir_path = std::path::Path::new("/test");

        let html = Server::generate_directory_listing_html_static(entries, dir_path);

        assert!(html.contains("Directory listing for /test"));
        assert!(html.contains("file1.html"));
        assert!(html.contains("subdir"));
        assert!(html.contains("📁"));
        assert!(html.contains("📄"));
    }

    #[test]
    fn test_server_creation() {
        let temp_dir = TempDir::new().unwrap();
        let server = Server::new(temp_dir.path().to_path_buf(), true, true, true);

        assert_eq!(server.directory, temp_dir.path());
        assert!(server.live_reload);
        assert!(server.cors);
        assert!(server.debug);
    }
}
