// File:    main.rs
// Author:  apezoo
// Date:    2025-07-17
//
// Description: The entry point and main logic for the web-based interface (GUI) of the OTP application.
//
// License:
// This project is licensed under the terms of the GNU AGPLv3 license.
// See the LICENSE.md file in the project root for full license information.

#![deny(missing_docs)]
//! A web server for the OTP encryption tool, providing a user-friendly interface.

use axum::{
    body::Body,
    extract::{Multipart, State},
    http::{header, StatusCode, Uri},
    response::{IntoResponse, Json, Redirect},
    routing::{delete, get, post},
    Router,
};
use local_ip_address::local_ip;
use otp_core::{pad_generator, state_manager};
use serde::Deserialize;
use serde_json::{json, Value};
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use rust_embed::RustEmbed;
use uuid::Uuid;

#[derive(RustEmbed)]
#[folder = "../static/"]
struct Asset;

/// Shared application state
#[derive(Clone)]
struct AppState {
    vault_path: PathBuf,
}

#[derive(Deserialize)]
struct GeneratePadRequest {
    size: usize,
    count: u32,
}

#[derive(Deserialize)]
struct RequestSegmentRequest {
    pad_id: Option<String>,
    length: usize,
}

#[derive(serde::Serialize)]
struct RequestSegmentResponse {
    pad_id: String,
    start: usize,
    segment_data: Vec<u8>,
}

#[derive(Deserialize)]
struct MarkUsedRequest {
    pad_id: String,
    start: usize,
    end: usize,
}


#[tokio::main]
async fn main() {
    let port = 3000;
    // Set up the vault path from an environment variable or use a default.
    let vault_path =
        env::var("OTP_VAULT_PATH").map_or_else(|_| PathBuf::from("./.otp_vault"), PathBuf::from);

    println!("Using vault at: {}", vault_path.display());

    // Initialize the vault if it doesn't exist.
    if !vault_path.exists() {
        println!("Vault not found. Initializing new vault...");
        if let Err(e) = fs::create_dir_all(&vault_path) {
            eprintln!("Failed to create vault directory: {e}");
            std::process::exit(1);
        }
        if let Err(e) = fs::create_dir_all(vault_path.join("pads/available")) {
            eprintln!("Failed to create pads directory: {e}");
            std::process::exit(1);
        }
        if let Err(e) = fs::create_dir_all(vault_path.join("pads/used")) {
            eprintln!("Failed to create used pads directory: {e}");
            std::process::exit(1);
        }
        let initial_state = state_manager::VaultState::default();
        if let Err(e) = state_manager::save_state(&vault_path, &initial_state) {
            eprintln!("Failed to save initial state: {e}");
            std::process::exit(1);
        }
        println!("Vault initialized successfully.");
    }

    let app_state = Arc::new(AppState { vault_path });

    // Build the Axum router.
    let app = Router::new()
        .route("/api/vault/status", get(get_vault_status))
        .route("/api/pads", get(list_pads_handler))
        .route("/api/pads/:pad_id", delete(delete_pad_handler))
        .route("/api/pads/generate", post(generate_pads_handler))
        .route("/api/pads/upload", post(upload_pads_handler))
        .route("/api/pads/:pad_id/download", get(download_pad_handler))
        .route("/api/pads/request_segment", post(request_segment_handler))
        .route("/api/pads/mark_used", post(mark_used_handler))
        .route("/api/vault/clear", post(clear_vault_handler))
        .route("/", get(|| async { Redirect::permanent("/index.html") }))
        .with_state(app_state)
        .layer(CorsLayer::permissive())
        .fallback(static_path);

    // Run the server.
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    let my_local_ip = local_ip().unwrap_or_else(|e| {
        eprintln!("Failed to get local IP address: {e}");
        "127.0.0.1".parse().unwrap_or_else(|_| {
            eprintln!("Failed to parse hardcoded IP address");
            std::process::exit(1);
        })
    });

    println!("listening on:");
    println!("  - http://{my_local_ip}:{port}/index.html");
    println!("  - http://127.0.0.1:{port}/index.html");

    let listener = match tokio::net::TcpListener::bind(&addr).await {
        Ok(listener) => listener,
        Err(e) => {
            eprintln!("Failed to bind to address: {e}");
            return;
        }
    };
    if let Err(e) = axum::serve(listener, app).await {
        eprintln!("Server error: {e}");
    }
}

/// Returns the status of the OTP vault.
async fn get_vault_status(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    let vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load vault state: {}", e) })),
            );
        }
    };

    let available_pads = vault_state.pads.values().filter(|p| !p.is_fully_used).count();
    let used_pads = vault_state.pads.len() - available_pads;
    let total_pads = vault_state.pads.len();

    let total_storage_bytes: usize = vault_state.pads.values().map(|p| p.size).sum();
    let total_used_bytes: usize = vault_state.pads.values().map(state_manager::Pad::total_used_bytes).sum();
    let remaining_bytes = total_storage_bytes.saturating_sub(total_used_bytes);

    let response = json!({
        "vault_path": state.vault_path,
        "total_pads": total_pads,
        "available_pads": available_pads,
        "used_pads": used_pads,
        "total_storage_bytes": total_storage_bytes,
        "total_used_bytes": total_used_bytes,
        "remaining_bytes": remaining_bytes,
    });

    (StatusCode::OK, Json(response))
}

async fn generate_pads_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<GeneratePadRequest>,
) -> (StatusCode, Json<Value>) {
    let mut vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load vault state: {}", e) })),
            );
        }
    };
    let mut new_pad_ids = Vec::new();

    for _ in 0..payload.count {
        let pad_id = Uuid::new_v4().to_string();
        let file_name = format!("{pad_id}.pad");
        let pad_path = state.vault_path.join("pads/available").join(&file_name);
        let size_in_bytes = payload.size * 1024 * 1024;

        let Some(pad_path_str) = pad_path.to_str() else {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": "Pad path contains invalid UTF-8" })),
            );
        };
        match pad_generator::generate_pad(pad_path_str, size_in_bytes) {
            Ok(()) => {
                vault_state.add_pad(pad_id.clone(), file_name, size_in_bytes);
                new_pad_ids.push(pad_id);
            }
            Err(e) => {
                return (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(json!({ "error": format!("Failed to generate pad: {}", e) })),
                );
            }
        }
    }

    if let Err(e) = state_manager::save_state(&state.vault_path, &vault_state) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to save state: {}", e) })),
        );
    }
    (StatusCode::CREATED, Json(json!({ "pad_ids": new_pad_ids })))
}

async fn list_pads_handler(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    let vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load vault state: {}", e) })),
            );
        }
    };
    let pads: Vec<&state_manager::Pad> = vault_state.pads.values().collect();
    (StatusCode::OK, Json(json!(pads)))
}

async fn delete_pad_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(pad_id): axum::extract::Path<String>,
) -> (StatusCode, Json<Value>) {
    let mut vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load vault state: {}", e) })),
            );
        }
    };

    if let Some(pad_to_delete) = vault_state.pads.get(&pad_id) {
        let pad_dir = if pad_to_delete.is_fully_used { "used" } else { "available" };
        let pad_path = state.vault_path.join("pads").join(pad_dir).join(&pad_to_delete.file_name);

        match fs::remove_file(&pad_path) {
            Ok(()) => {
                vault_state.pads.remove(&pad_id);
                if let Err(e) = state_manager::save_state(&state.vault_path, &vault_state) {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": format!("Failed to save state: {}", e) })),
                    );
                }
                (StatusCode::OK, Json(json!({ "message": "Pad deleted successfully" })))
            }
            Err(e) => {
                if e.kind() == std::io::ErrorKind::NotFound {
                    vault_state.pads.remove(&pad_id);
                    if let Err(e) = state_manager::save_state(&state.vault_path, &vault_state) {
                        return (
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(json!({ "error": format!("Failed to save state: {}", e) })),
                        );
                    }
                     (StatusCode::OK, Json(json!({ "message": "Pad file not found, but removed from state" })))
                } else {
                    (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": format!("Failed to delete pad file: {}", e) })))
                }
            }
        }
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Pad not found" })))
    }
}


async fn request_segment_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<RequestSegmentRequest>,
) -> (StatusCode, Json<Value>) {
    let vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load vault state: {}", e) })),
            );
        }
    };
    let pad_id_to_use = match payload.pad_id {
        Some(id) => id,
        None => {
            match vault_state.pads.values().find(|p| p.find_available_segment(payload.length).is_some()) {
                Some(pad) => pad.id.clone(),
                None => return (StatusCode::BAD_REQUEST, Json(json!({ "error": "No available pad with enough space" }))),
            }
        }
    };

    if let Some(pad) = vault_state.pads.get(&pad_id_to_use) {
        if let Some(start) = pad.find_available_segment(payload.length) {
            let pad_dir = if pad.is_fully_used { "used" } else { "available" };
            let pad_path = state.vault_path.join("pads").join(pad_dir).join(&pad.file_name);
            let pad_data = match fs::read(&pad_path) {
                Ok(data) => data,
                Err(e) => {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": format!("Failed to read pad file: {e}") })),
                    )
                }
            };
            let segment_data = pad_data[start..start + payload.length].to_vec();

            let response = RequestSegmentResponse {
                pad_id: pad_id_to_use,
                start,
                segment_data,
            };
            (StatusCode::OK, Json(json!(response)))
        } else {
            (StatusCode::BAD_REQUEST, Json(json!({ "error": "Not enough contiguous space in selected pad" })))
        }
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Pad not found" })))
    }
}

async fn mark_used_handler(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<MarkUsedRequest>,
) -> (StatusCode, Json<Value>) {
    let mut vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load vault state: {}", e) })),
            );
        }
    };
    if let Some(pad) = vault_state.pads.get_mut(&payload.pad_id) {
        pad.used_segments.push(state_manager::UsedSegment { start: payload.start, end: payload.end });
        pad.is_fully_used = pad.total_used_bytes() >= pad.size;
        let is_full = pad.is_fully_used;
        let file_name_clone = pad.file_name.clone();

        if let Err(e) = state_manager::save_state(&state.vault_path, &vault_state) {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to save state: {}", e) })),
            );
        }

        if is_full {
             let old_pad_path = state.vault_path.join("pads/available").join(&file_name_clone);
            let used_pad_path = state.vault_path.join("pads/used").join(&file_name_clone);
            if old_pad_path.exists() {
                if let Err(e) = fs::rename(old_pad_path, used_pad_path) {
                    return (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(json!({ "error": format!("Failed to move used pad: {e}") })),
                    );
                }
            }
        }
        (StatusCode::OK, Json(json!({ "message": "Pad segment marked as used" })))
    } else {
        (StatusCode::NOT_FOUND, Json(json!({ "error": "Pad not found" })))
    }
}

async fn download_pad_handler(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(pad_id): axum::extract::Path<String>,
) -> impl IntoResponse {
    let vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Failed to load vault state: {e}")).into_response();
        }
    };
    vault_state.pads.get(&pad_id).map_or_else(
        || (StatusCode::NOT_FOUND, "Pad not found in state").into_response(),
        |pad| {
            let pad_dir = if pad.is_fully_used { "used" } else { "available" };
            let pad_path = state.vault_path.join("pads").join(pad_dir).join(&pad.file_name);
            fs::read(&pad_path).map_or_else(
                |_err| (StatusCode::NOT_FOUND, "Pad file not found").into_response(),
                |data| {
                    let headers = [
                        (header::CONTENT_TYPE, "application/octet-stream".to_string()),
                        (
                            header::CONTENT_DISPOSITION,
                            format!("attachment; filename=\"{}\"", pad.file_name),
                        ),
                    ];
                    (headers, Body::from(data)).into_response()
                },
            )
        },
    )
}

async fn upload_pads_handler(
    State(state): State<Arc<AppState>>,
    mut multipart: Multipart,
) -> (StatusCode, Json<Value>) {
    let mut vault_state = match state_manager::load_state(&state.vault_path) {
        Ok(vs) => vs,
        Err(e) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({ "error": format!("Failed to load vault state: {}", e) })),
            );
        }
    };
    let mut imported_pads = Vec::new();

    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = field.file_name().unwrap_or("unknown.pad").to_string();
        let data = match field.bytes().await {
            Ok(data) => data,
            Err(e) => {
                return (
                    StatusCode::BAD_REQUEST,
                    Json(json!({ "error": format!("Failed to get field data: {e}") })),
                );
            }
        };
        let size_in_bytes = data.len();

        // Basic validation: ensure it's a .pad file
        if !std::path::Path::new(&file_name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("pad"))
        {
            continue;
        }
        
        // The pad ID is the file name without the extension.
        let pad_id = file_name.trim_end_matches(".pad").to_string();

        // Prevent overwriting existing pads
        if vault_state.pads.contains_key(&pad_id) {
            return (StatusCode::CONFLICT, Json(json!({ "error": format!("Pad with ID {} already exists.", pad_id) })));
        }

        let pad_path = state.vault_path.join("pads/available").join(&file_name);
        if fs::write(&pad_path, &data).is_ok() {
            vault_state.add_pad(pad_id.clone(), file_name, size_in_bytes);
            imported_pads.push(pad_id);
        }
    }

    if let Err(e) = state_manager::save_state(&state.vault_path, &vault_state) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to save state: {}", e) })),
        );
    }
    (StatusCode::OK, Json(json!({ "imported_pads": imported_pads })))
}

async fn static_path(uri: Uri) -> impl IntoResponse {
    let mut path = uri.path().trim_start_matches('/').to_string();
    if path.is_empty() {
        path = "index.html".to_string();
    }

    match Asset::get(&path) {
        Some(content) => {
            let body = Body::from(content.data);
            let mime = mime_guess::from_path(path).first_or_octet_stream();
            axum::response::Response::builder()
                .header(header::CONTENT_TYPE, mime.as_ref())
                .body(body)
                .unwrap_or_else(|_| {
                    (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response()
                })
        }
        None => axum::response::Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body(Body::empty())
            .unwrap_or_else(|_| {
                (StatusCode::INTERNAL_SERVER_ERROR, "Failed to build response").into_response()
            }),
    }
}

async fn clear_vault_handler(
    State(state): State<Arc<AppState>>,
) -> (StatusCode, Json<Value>) {
    if let Err(e) = fs::remove_dir_all(&state.vault_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to remove vault directory: {e}") })),
        );
    }
    if let Err(e) = fs::create_dir_all(&state.vault_path) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create vault directory: {e}") })),
        );
    }
    if let Err(e) = fs::create_dir_all(state.vault_path.join("pads/available")) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create pads directory: {e}") })),
        );
    }
    if let Err(e) = fs::create_dir_all(state.vault_path.join("pads/used")) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to create used pads directory: {e}") })),
        );
    }
    let initial_state = state_manager::VaultState::default();
    if let Err(e) = state_manager::save_state(&state.vault_path, &initial_state) {
        return (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Failed to save initial state: {}", e) })),
        );
    }
    (StatusCode::OK, Json(json!({ "message": "Vault cleared successfully" })))
}

