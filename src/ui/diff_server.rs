use std::collections::HashMap;
use std::convert::Infallible;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::oneshot;
use tokio::time::timeout;
use warp::Filter;
use serde_json::json;
use crate::ui::session_manager::{SessionManager, SessionStatus};
use crate::enums::file_change::FileChange;
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
        // Find an available port
        let port = self.find_available_port().await?;
        self.port = Some(port);

        let session_manager = Arc::clone(&self.session_manager);

        // Create shutdown channel
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
        self.shutdown_tx = Some(shutdown_tx);

        // Static files route
        let static_files = warp::path("static")
            .and(warp::fs::dir("src/ui/static"));

        // Main diff viewer route
        let diff_route = warp::path("diff")
            .and(warp::query::<HashMap<String, String>>())
            .and_then(serve_diff_page);

        // API routes
        let api_routes = self.create_api_routes(Arc::clone(&session_manager));

        // Combine all routes
        let routes = static_files
            .or(diff_route)
            .or(api_routes)
            .with(warp::cors().allow_any_origin());

        // Start server
        let addr: SocketAddr = ([127, 0, 0, 1], port).into();
        let (_, server) = warp::serve(routes)
            .bind_with_graceful_shutdown(addr, async {
                shutdown_rx.await.ok();
            });

        // Spawn server task
        tokio::spawn(server);

        log::info!("🌐 Diff server started on port {}", port);
        Ok(port)
    }

    pub async fn create_session(
        &self,
        repository_config: &RepositoryConfig,
        changes: Vec<FileChange>,
    ) -> AicedResult<String> {
        self.session_manager.create_session(repository_config, &changes)
    }

    pub async fn wait_for_completion(&self, session_id: &str, timeout_minutes: u64) -> AicedResult<Vec<String>> {
        let timeout_duration = Duration::from_secs(timeout_minutes * 60);

        let result = timeout(timeout_duration, async {
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
                            tokio::time::sleep(Duration::from_millis(500)).await;
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
        if let Some(shutdown_tx) = self.shutdown_tx.take() {
            let _ = shutdown_tx.send(());
        }
        Ok(())
    }

    fn create_api_routes(
        &self,
        session_manager: Arc<SessionManager>,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        let session_manager_filter = warp::any().map(move || Arc::clone(&session_manager));

        // GET /api/session/{id}
        let get_session = warp::path!("api" / "session" / String)
            .and(warp::get())
            .and(session_manager_filter.clone())
            .and_then(get_session_handler);

        // POST /api/session/{id}/apply
        let apply_change = warp::path!("api" / "session" / String / "apply")
            .and(warp::post())
            .and(warp::body::json())
            .and(session_manager_filter.clone())
            .and_then(apply_change_handler);

        // POST /api/session/{id}/unapply
        let unapply_change = warp::path!("api" / "session" / String / "unapply")
            .and(warp::post())
            .and(warp::body::json())
            .and(session_manager_filter.clone())
            .and_then(unapply_change_handler);

        // POST /api/session/{id}/complete
        let complete_session = warp::path!("api" / "session" / String / "complete")
            .and(warp::post())
            .and(session_manager_filter.clone())
            .and_then(complete_session_handler);

        // POST /api/session/{id}/cancel
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
        for port in 8080..8200 {
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

// Handler functions
async fn serve_diff_page(
    params: HashMap<String, String>,
) -> Result<impl warp::Reply, Infallible> {
    let session_id = params.get("session").unwrap_or(&"".to_string()).clone();

    let html = include_str!("static/index.html")
        .replace("{{SESSION_ID}}", &session_id);

    Ok(warp::reply::html(html))
}

async fn get_session_handler(
    session_id: String,
    session_manager: Arc<SessionManager>,
) -> Result<impl warp::Reply, Infallible> {
    match session_manager.get_session(&session_id) {
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
    if let Some(change_id) = body.get("change_id").and_then(|v| v.as_str()) {
        match session_manager.apply_change(&session_id, change_id) {
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
    if let Some(change_id) = body.get("change_id").and_then(|v| v.as_str()) {
        match session_manager.unapply_change(&session_id, change_id) {
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
    match session_manager.complete_session(&session_id) {
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
    match session_manager.cancel_session(&session_id) {
        Ok(_) => Ok(warp::reply::json(&json!({
            "success": true,
            "message": "Session cancelled"
        }))),
        Err(e) => Ok(warp::reply::json(&json!({
            "error": format!("Failed to cancel session: {}", e)
        }))),
    }
}

