use crate::error::{AppError, AppResult};
use crate::progress::ProgressEmitter;
use crate::state::AppState;
use anki::collection::CollectionBuilder;
use anki::sync::collection::normal::SyncActionRequired;
use anki::sync::login::{sync_login, SyncAuth};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use tauri::{AppHandle, Manager, State};

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct StoredCredentials {
    username: String,
    hkey: String,
    endpoint: Option<String>,
}

/// File-based credential storage in the app data dir.
/// Plain JSON with mode 0600 on Unix. Keychain isn't used in dev because
/// every recompile produces a binary with a different code signature, which
/// triggers a permission prompt every single launch. When we ship a signed
/// release we can swap this back to a keychain backend.
fn credentials_path(app: &AppHandle) -> AppResult<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("app_data_dir: {e}")))?;
    std::fs::create_dir_all(&dir)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("mkdir app_data_dir: {e}")))?;
    Ok(dir.join("ankiweb-credentials.json"))
}

fn load_credentials(app: &AppHandle) -> AppResult<Option<StoredCredentials>> {
    let path = credentials_path(app)?;
    match std::fs::read_to_string(&path) {
        Ok(s) => {
            let creds: StoredCredentials = serde_json::from_str(&s).map_err(|e| {
                AppError::Anyhow(anyhow::anyhow!("credentials parse: {e}"))
            })?;
            Ok(Some(creds))
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(e) => Err(AppError::Anyhow(anyhow::anyhow!("credentials read: {e}"))),
    }
}

fn save_credentials(app: &AppHandle, creds: &StoredCredentials) -> AppResult<()> {
    let path = credentials_path(app)?;
    let payload = serde_json::to_string_pretty(creds)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("serialize: {e}")))?;
    std::fs::write(&path, payload)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("credentials write: {e}")))?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perm = std::fs::metadata(&path)
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("metadata: {e}")))?
            .permissions();
        perm.set_mode(0o600);
        std::fs::set_permissions(&path, perm)
            .map_err(|e| AppError::Anyhow(anyhow::anyhow!("chmod 0600: {e}")))?;
    }
    Ok(())
}

fn delete_credentials(app: &AppHandle) -> AppResult<()> {
    let path = credentials_path(app)?;
    match std::fs::remove_file(&path) {
        Ok(_) => Ok(()),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(e) => Err(AppError::Anyhow(anyhow::anyhow!("credentials delete: {e}"))),
    }
}

#[derive(Serialize, Debug)]
pub struct SyncStatus {
    pub logged_in: bool,
    pub username: Option<String>,
}

#[tauri::command]
pub async fn sync_status(app: AppHandle) -> AppResult<SyncStatus> {
    let creds = load_credentials(&app)?;
    Ok(SyncStatus {
        logged_in: creds.is_some(),
        username: creds.map(|c| c.username),
    })
}

#[derive(Deserialize, Debug)]
pub struct LoginInput {
    pub username: String,
    pub password: String,
    pub endpoint: Option<String>,
}

#[tauri::command]
pub async fn sync_login_cmd(
    input: LoginInput,
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<SyncStatus> {
    let auth = sync_login(
        input.username.clone(),
        input.password.clone(),
        input.endpoint.clone(),
        state.http.clone(),
    )
    .await?;

    let creds = StoredCredentials {
        username: input.username,
        hkey: auth.hkey,
        endpoint: input.endpoint,
    };
    save_credentials(&app, &creds)?;

    Ok(SyncStatus {
        logged_in: true,
        username: Some(creds.username),
    })
}

#[tauri::command]
pub async fn sync_logout(app: AppHandle) -> AppResult<()> {
    delete_credentials(&app)
}

#[derive(Serialize, Debug)]
pub struct SyncReport {
    pub kind: &'static str, // "no_changes" | "normal_done" | "full_required"
    pub upload_ok: bool,
    pub download_ok: bool,
    pub server_message: String,
    pub host_number: u32,
    pub new_endpoint: Option<String>,
    pub local_pending_notes: u32,
    pub local_pending_cards: u32,
}

fn auth_from(creds: &StoredCredentials) -> AppResult<SyncAuth> {
    let endpoint = creds
        .endpoint
        .as_deref()
        .map(|s| {
            reqwest::Url::parse(s)
                .map_err(|e| AppError::Anyhow(anyhow::anyhow!("bad endpoint: {e}")))
        })
        .transpose()?;
    Ok(SyncAuth {
        hkey: creds.hkey.clone(),
        endpoint,
        io_timeout_secs: None,
    })
}

#[tauri::command]
pub async fn sync_now(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<SyncReport> {
    let creds = load_credentials(&app)?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("not logged in")))?;
    let auth = auth_from(&creds)?;
    let _emitter = ProgressEmitter::start(app.clone(), state.progress.clone());

    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;

    tracing::info!("starting normal_sync");
    let out = col.normal_sync(auth, state.http.clone()).await?;
    tracing::info!(
        ?out.required,
        server_message = %out.server_message,
        new_endpoint = ?out.new_endpoint,
        host_number = out.host_number,
        "sync done"
    );

    // If the server told us about a host-specific endpoint (typical:
    // https://sync13.ankiweb.net/), persist it so subsequent full_upload /
    // full_download go to the right shard. Otherwise full sync hits the
    // generic sync.ankiweb.net and gets a 303.
    if let Some(ref endpoint) = out.new_endpoint {
        if let Some(mut updated) = load_credentials(&app)? {
            if updated.endpoint.as_deref() != Some(endpoint.as_str()) {
                tracing::info!(endpoint, "saving discovered endpoint to credentials");
                updated.endpoint = Some(endpoint.clone());
                save_credentials(&app, &updated)?;
            }
        }
    }

    let pending_notes = 0u32;
    let pending_cards = 0u32;

    Ok(match out.required {
        SyncActionRequired::NoChanges => SyncReport {
            kind: "no_changes",
            upload_ok: false,
            download_ok: false,
            server_message: out.server_message,
            host_number: out.host_number,
            new_endpoint: out.new_endpoint,
            local_pending_notes: pending_notes,
            local_pending_cards: pending_cards,
        },
        SyncActionRequired::NormalSyncRequired => SyncReport {
            kind: "normal_done",
            upload_ok: false,
            download_ok: false,
            server_message: out.server_message,
            host_number: out.host_number,
            new_endpoint: out.new_endpoint,
            local_pending_notes: pending_notes,
            local_pending_cards: pending_cards,
        },
        SyncActionRequired::FullSyncRequired {
            upload_ok,
            download_ok,
        } => SyncReport {
            kind: "full_required",
            upload_ok,
            download_ok,
            server_message: out.server_message,
            host_number: out.host_number,
            new_endpoint: out.new_endpoint,
            local_pending_notes: pending_notes,
            local_pending_cards: pending_cards,
        },
    })
}

#[tauri::command]
pub async fn sync_full_upload(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    full_sync(app, state, true).await
}

#[tauri::command]
pub async fn sync_full_download(
    app: AppHandle,
    state: State<'_, AppState>,
) -> AppResult<()> {
    full_sync(app, state, false).await
}

async fn full_sync(app: AppHandle, state: State<'_, AppState>, upload: bool) -> AppResult<()> {
    let creds = load_credentials(&app)?
        .ok_or_else(|| AppError::Anyhow(anyhow::anyhow!("not logged in")))?;
    let auth = auth_from(&creds)?;

    let path = state
        .col_path
        .lock()
        .await
        .clone()
        .ok_or(AppError::CollectionNotOpen)?;

    // Take the collection out (full_upload/download consume self).
    let col = state
        .col
        .lock()
        .await
        .take()
        .ok_or(AppError::CollectionNotOpen)?;

    let _emitter = ProgressEmitter::start(app, state.progress.clone());
    tracing::info!(upload, "starting full_sync");
    let result = if upload {
        col.full_upload(auth, state.http.clone()).await
    } else {
        col.full_download(auth, state.http.clone()).await
    };
    if let Err(ref e) = result {
        tracing::error!(?e, "full_sync failed");
    } else {
        tracing::info!("full_sync ok");
    }

    // Always try to re-open before returning, even if full_upload errored,
    // so the app doesn't end up in a "collection not open" state.
    match CollectionBuilder::new(&path)
        .set_shared_progress_state(state.progress.clone())
        .build()
    {
        Ok(reopened) => *state.col.lock().await = Some(reopened),
        Err(e) => tracing::error!(?e, "failed to re-open after full sync"),
    }

    result?;
    Ok(())
}
