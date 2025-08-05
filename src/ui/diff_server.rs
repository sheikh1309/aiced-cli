use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio::time::timeout;
use warp::Filter;
use serde_json::json;
use crate::config::constants::{
    DEFAULT_SERVER_PORT_RANGE_START, DEFAULT_SERVER_PORT_RANGE_END, 
    MAX_SESSION_ID_LENGTH, SERVER_SHUTDOWN_GRACE_PERIOD_MS,
    SESSION_CLEANUP_POLL_INTERVAL_MS, timeout_duration, sleep_duration_millis
};
use crate::ui::session_manager::SessionManager;
use crate::enums::file_change::FileChange;
use crate::enums::session_status::SessionStatus;
use crate::structs::config::repository_config::RepositoryConfig;
use crate::errors::{AicedResult, AicedError};

pub struct DiffServer {
    session_manager: Arc<SessionManager>,
    port: Option<u16>,
    shutdown_tx: Option<oneshot::Sender<()>>,
}

impl DiffServer {
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(SessionManager::new()),
            port: None,
            shutdown_tx: None,
        }
    }

    pub async fn start(&mut self) -> AicedResult<u16> {
        let port = self.find_available_port().await?;
        self.port = Some(port);

        let session_manager = Arc::clone(&self.session_manager);

        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Static files route (for /static/* paths)
        let static_files = warp::path("static")
            .and(warp::fs::dir("src/ui/static"));

        // Assets route (for /assets/* paths) - ADD THIS
        let assets_route = warp::path("assets")
            .and(warp::fs::dir("src/ui/static/assets"));

        let diff_route = warp::path::end()  // Matches the root path "/"
            .and(warp::query::<HashMap<String, String>>())
            .and_then(serve_diff_page);

        let api_routes = self.create_api_routes(Arc::clone(&session_manager));

        let routes = static_files
            .or(assets_route) 
            .or(diff_route)
            .or(api_routes)
            .with(warp::cors()
                .allow_origin("http://127.0.0.1")
                .allow_origin("http://localhost")
                .allow_headers(vec!["content-type"])
                .allow_methods(vec!["GET", "POST"]));

        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(addr, async {
                shutdown_rx.await.ok();
            });

        tokio::spawn(server);

        log::info!("🌐 Diff server started on port {}", port);
        Ok(port)
    }

    pub async fn create_session(&self, repository_config: &RepositoryConfig, changes: Vec<FileChange>) -> AicedResult<String> {
        self.session_manager.create_session(repository_config, &changes)
    }

    pub async fn wait_for_completion(&self, session_id: &str, timeout_minutes: u64) -> AicedResult<Vec<String>> {
        let timeout_dur = timeout_duration(timeout_minutes);

        let result = timeout(timeout_dur, async {
            loop {
                if let Some(session) = self.session_manager.get_session(session_id) {
                    match session.status {
                        SessionStatus::Completed => {
                            return Ok(session.applied_changes.into_iter().collect());
                        }
                        SessionStatus::Cancelled => {
                            return Ok(Vec::new());
                        }
                        SessionStatus::Active => {
                            tokio::time::sleep(sleep_duration_millis(SESSION_CLEANUP_POLL_INTERVAL_MS)).await;
                        }
                    }
                } else {
                    return Err(AicedError::validation_error(
                        "Session not found",
                        "Session not found",
                        "Session not found",
                        Some("Session not found"),
                    ));
                }
            }
        }).await;

        match result {
            Ok(applied_changes) => applied_changes,
            Err(_) => {
                log::warn!("⏰ Diff review session timed out after {} minutes", timeout_minutes);
                Ok(Vec::new())
            }
        }
    }

    pub async fn shutdown(&mut self) -> AicedResult<()> {
        log::info!("🛑 Shutting down diff server...");
        
        self.session_manager.cleanup_expired_sessions();
        
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            shutdown_tx.send(()).map_err(|_| 
                AicedError::system_error("shutdown", "Failed to send shutdown signal")
            )?;
        }
        
        tokio::time::sleep(sleep_duration_millis(SERVER_SHUTDOWN_GRACE_PERIOD_MS)).await;
        log::info!("✅ Diff server shutdown complete");
        
        Ok(())
    }

    fn create_api_routes(
        &self,
        session_manager: Arc<SessionManager>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let session_manager_filter = warp::any().map(move || Arc::clone(&session_manager));

        let get_session = warp::path!("api" / "session" / String)
            .and(warp::get())
            .and(session_manager_filter.clone())
            .and_then(get_session_handler);

        let apply_change = warp::path!("api" / "session" / String / "apply")
            .and(warp::post())
            .and(warp::body::json())
            .and(session_manager_filter.clone())
            .and_then(apply_change_handler);

        let unapply_change = warp::path!("api" / "session" / String / "unapply")
            .and(warp::post())
            .and(warp::body::json())
            .and(session_manager_filter.clone())
            .and_then(unapply_change_handler);

        let complete_session = warp::path!("api" / "session" / String / "complete")
            .and(warp::post())
            .and(session_manager_filter.clone())
            .and_then(complete_session_handler);

        let cancel_session = warp::path!("api" / "session" / String / "cancel")
            .and(warp::post())
            .and(session_manager_filter)
            .and_then(cancel_session_handler);

        get_session
            .or(apply_change)
            .or(unapply_change)
            .or(complete_session)
            .or(cancel_session)
    }

    async fn find_available_port(&self) -> AicedResult<u16> {
        for port in DEFAULT_SERVER_PORT_RANGE_START..DEFAULT_SERVER_PORT_RANGE_END {
            if let Ok(listener) = tokio::net::TcpListener::bind(format!("127.0.0.1:{}", port)).await {
                drop(listener);
                return Ok(port);
            }
        }
        Err(AicedError::validation_error(
            "No available ports found",
            "No available ports found",
            "No available ports found",
            Some("No available ports found"),
        ))
    }
}

async fn serve_diff_page(params: HashMap<String, String>) -> Result<impl warp::Reply, Infallible> {
    let session_id = params.get("session")
        .map(|s| sanitize_session_id(s))
        .unwrap_or_default();

    let html = include_str!("static/index.html")
        .replace("{{SESSION_ID}}", &session_id);

    Ok(warp::reply::html(html))
}

fn sanitize_session_id(session_id: &str) -> String {
    session_id.chars()
        .filter(|c| c.is_alphanumeric() || *c == '-' || *c == '_')
        .take(MAX_SESSION_ID_LENGTH)
        .collect()
}

async fn get_session_handler(session_id: String, session_manager: Arc<SessionManager>) -> Result<impl warp::Reply, Infallible> {
    let sanitized_session_id = sanitize_session_id(&session_id);
    if sanitized_session_id.is_empty() {
        return Ok(warp::reply::json(&json!({
            "error": "Invalid session ID"
        })));
    }

    match session_manager.get_session(&sanitized_session_id) {
        Some(session) => Ok(warp::reply::json(&session)),
        None => Ok(warp::reply::json(&json!({
            "error": "Session not found"
        }))),
    }
}

async fn apply_change_handler(
    session_id: String,
    body: serde_json::Value,
    session_manager: Arc<SessionManager>,
) -> Result<impl warp::Reply, Infallible> {
    let sanitized_session_id = sanitize_session_id(&session_id);
    if sanitized_session_id.is_empty() {
        return Ok(warp::reply::json(&json!({
            "error": "Invalid session ID"
        })));
    }

    if let Some(change_id) = body.get("change_id").and_then(|v| v.as_str()) {
        let sanitized_change_id = sanitize_session_id(change_id);
        if sanitized_change_id.is_empty() {
            return Ok(warp::reply::json(&json!({
                "error": "Invalid change ID"
            })));
        }

        match session_manager.apply_change(&sanitized_session_id, &sanitized_change_id) {
            Ok(success) => Ok(warp::reply::json(&json!({
                "success": success,
                "message": if success { "Change applied" } else { "Change not found" }
            }))),
            Err(e) => Ok(warp::reply::json(&json!({
                "error": format!("Failed to apply change: {}", e)
            }))),
        }
    } else {
        Ok(warp::reply::json(&json!({
            "error": "Missing change_id"
        })))
    }
}

async fn unapply_change_handler(
    session_id: String,
    body: serde_json::Value,
    session_manager: Arc<SessionManager>,
) -> Result<impl warp::Reply, Infallible> {
    let sanitized_session_id = sanitize_session_id(&session_id);
    if sanitized_session_id.is_empty() {
        return Ok(warp::reply::json(&json!({
            "error": "Invalid session ID"
        })));
    }

    if let Some(change_id) = body.get("change_id").and_then(|v| v.as_str()) {
        let sanitized_change_id = sanitize_session_id(change_id);
        if sanitized_change_id.is_empty() {
            return Ok(warp::reply::json(&json!({
                "error": "Invalid change ID"
            })));
        }

        match session_manager.unapply_change(&sanitized_session_id, &sanitized_change_id) {
            Ok(success) => Ok(warp::reply::json(&json!({
                "success": success,
                "message": if success { "Change unapplied" } else { "Change not found" }
            }))),
            Err(e) => Ok(warp::reply::json(&json!({
                "error": format!("Failed to unapply change: {}", e)
            }))),
        }
    } else {
        Ok(warp::reply::json(&json!({
            "error": "Missing change_id"
        })))
    }
}

async fn complete_session_handler(
    session_id: String,
    session_manager: Arc<SessionManager>,
) -> Result<impl warp::Reply, Infallible> {
    let sanitized_session_id = sanitize_session_id(&session_id);
    if sanitized_session_id.is_empty() {
        return Ok(warp::reply::json(&json!({
            "error": "Invalid session ID"
        })));
    }

    match session_manager.complete_session(&sanitized_session_id) {
        Ok(applied_changes) => Ok(warp::reply::json(&json!({
            "success": true,
            "applied_changes": applied_changes,
            "message": "Session completed"
        }))),
        Err(e) => Ok(warp::reply::json(&json!({
            "error": format!("Failed to complete session: {}", e)
        }))),
    }
}

async fn cancel_session_handler(
    session_id: String,
    session_manager: Arc<SessionManager>,
) -> Result<impl warp::Reply, Infallible> {
    let sanitized_session_id = sanitize_session_id(&session_id);
    if sanitized_session_id.is_empty() {
        return Ok(warp::reply::json(&json!({
            "error": "Invalid session ID"
        })));
    }

    match session_manager.cancel_session(&sanitized_session_id) {
        Ok(_) => Ok(warp::reply::json(&json!({
            "success": true,
            "message": "Session cancelled"
        }))),
        Err(e) => Ok(warp::reply::json(&json!({
            "error": format!("Failed to cancel session: {}", e)
        }))),
    }
}
