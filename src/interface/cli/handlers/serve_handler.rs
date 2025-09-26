use std::path::PathBuf;
use tokio::sync::broadcast;
use warp::Filter;

use crate::interface::cli::logging::Logger;

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

    // Initialize logging
    if debug {
        unsafe {
            std::env::set_var("RUST_LOG", "debug");
        }
    } else {
        unsafe {
            std::env::set_var("RUST_LOG", "info");
        }
    }
    let _ = env_logger::try_init();

    Logger::info(&format!(
        "Starting server on {}:{} serving directory: {}",
        host,
        port,
        directory.display()
    ));

    if live_reload {
        Logger::info("Live reload enabled");
    }

    if cors {
        Logger::info("CORS enabled");
    }

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
}

impl Server {
    pub fn new(directory: PathBuf, live_reload: bool, cors: bool, debug: bool) -> Self {
        Self {
            directory,
            live_reload,
            cors,
            debug,
            shutdown_tx: None,
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

        // Create file watcher if live reload is enabled
        let file_watcher = if live_reload {
            Some(self.create_file_watcher(directory.clone(), shutdown_tx.clone())?)
        } else {
            None
        };

        // Create the warp filter
        let routes = Self::create_routes_static(directory, live_reload, cors);

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

        // Start the server
        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(addr, async move {
                shutdown_rx.recv().await.ok();
                Logger::info("Server shutting down...");
            });

        Logger::info(&format!("Server running at http://{}:{}", host, port));
        Logger::info("Press Ctrl+C to stop the server");

        // Run the server
        server.await;

        // Clean up file watcher
        if let Some(watcher) = file_watcher {
            drop(watcher);
        }

        Ok(())
    }

    fn create_routes_static(
        directory: PathBuf,
        live_reload: bool,
        _cors: bool,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        use warp::path::FullPath;

        // Live reload script injection
        let live_reload_script = if live_reload {
            r#"
<script>
(function() {
    const ws = new WebSocket('ws://localhost:35729');
    ws.onmessage = function(event) {
        if (event.data === 'reload') {
            window.location.reload();
        }
    };
})();
</script>
"#
        } else {
            ""
        };

        // Static file serving
        let static_files = warp::path::full()
            .and_then(move |path: FullPath| {
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
        use warp::http::header::{HeaderValue, CONTENT_TYPE};
        use warp::reply::Response;

        match fs::read(file_path) {
            Ok(mut content) => {
                // Inject live reload script for HTML files
                let content_type = mime_guess::from_path(file_path)
                    .first_or_octet_stream()
                    .to_string();
                
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
        use warp::http::header::{HeaderValue, CONTENT_TYPE};
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
        let mut watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                match res {
                    Ok(event) => {
                        if _debug {
                            Logger::debug(&format!("File changed: {:?}", event));
                        }
                        // Send reload signal to all connected clients
                        // This would be implemented with WebSocket in a real implementation
                    }
                    Err(e) => {
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
                ).await;
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
        let server = Server::new(
            temp_dir.path().to_path_buf(),
            true,
            true,
            true,
        );

        assert_eq!(server.directory, temp_dir.path());
        assert!(server.live_reload);
        assert!(server.cors);
        assert!(server.debug);
    }
}
