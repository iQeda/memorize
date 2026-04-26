use crate::error::{AppError, AppResult};
use crate::progress::ProgressEmitter;
use crate::state::AppState;
use anki::collection::CollectionBuilder;
use anki::sync::collection::normal::SyncActionRequired;
use anki::sync::login::{sync_login, SyncAuth};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, State};

const KEYRING_SERVICE: &str = "dev.iqeda.memorize";
const KEYRING_ACCOUNT: &str = "ankiweb-credentials";

#[derive(Serialize, Deserialize, Default, Debug, Clone)]
struct StoredCredentials {
    username: String,
    hkey: String,
    endpoint: Option<String>,
}

fn load_credentials() -> AppResult<Option<StoredCredentials>> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("keyring: {e}")))?;
    match entry.get_password() {
        Ok(s) => {
            let creds: StoredCredentials = serde_json::from_str(&s)
                .map_err(|e| AppError::Anyhow(anyhow::anyhow!("credentials parse: {e}")))?;
            Ok(Some(creds))
        }
        Err(keyring::Error::NoEntry) => Ok(None),
        Err(e) => Err(AppError::Anyhow(anyhow::anyhow!("keyring: {e}"))),
    }
}

fn save_credentials(creds: &StoredCredentials) -> AppResult<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("keyring: {e}")))?;
    let payload = serde_json::to_string(creds)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("serialize: {e}")))?;
    entry
        .set_password(&payload)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("keyring set: {e}")))?;
    Ok(())
}

fn delete_credentials() -> AppResult<()> {
    let entry = keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|e| AppError::Anyhow(anyhow::anyhow!("keyring: {e}")))?;
    match entry.delete_credential() {
        Ok(_) | Err(keyring::Error::NoEntry) => Ok(()),
        Err(e) => Err(AppError::Anyhow(anyhow::anyhow!("keyring delete: {e}"))),
    }
}

#[derive(Serialize, Debug)]
pub struct SyncStatus {
    pub logged_in: bool,
    pub username: Option<String>,
}

#[tauri::command]
pub async fn sync_status() -> AppResult<SyncStatus> {
    let creds = load_credentials()?;
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
    save_credentials(&creds)?;

    Ok(SyncStatus {
        logged_in: true,
        username: Some(creds.username),
    })
}

#[tauri::command]
pub async fn sync_logout() -> AppResult<()> {
    delete_credentials()
}

#[derive(Serialize, Debug)]
pub struct SyncReport {
    pub kind: &'static str, // "no_changes" | "normal_done" | "full_required"
    pub upload_ok: bool,
    pub download_ok: bool,
    pub server_message: String,
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
    let creds = load_credentials()?.ok_or_else(|| {
        AppError::Anyhow(anyhow::anyhow!("not logged in"))
    })?;
    let auth = auth_from(&creds)?;
    let _emitter = ProgressEmitter::start(app, state.progress.clone());

    let mut guard = state.col.lock().await;
    let col = guard.as_mut().ok_or(AppError::CollectionNotOpen)?;
    let out = col.normal_sync(auth, state.http.clone()).await?;

    Ok(match out.required {
        SyncActionRequired::NoChanges => SyncReport {
            kind: "no_changes",
            upload_ok: false,
            download_ok: false,
            server_message: out.server_message,
        },
        SyncActionRequired::NormalSyncRequired => SyncReport {
            kind: "normal_done",
            upload_ok: false,
            download_ok: false,
            server_message: out.server_message,
        },
        SyncActionRequired::FullSyncRequired {
            upload_ok,
            download_ok,
        } => SyncReport {
            kind: "full_required",
            upload_ok,
            download_ok,
            server_message: out.server_message,
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
    let creds = load_credentials()?.ok_or_else(|| {
        AppError::Anyhow(anyhow::anyhow!("not logged in"))
    })?;
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
    let result = if upload {
        col.full_upload(auth, state.http.clone()).await
    } else {
        col.full_download(auth, state.http.clone()).await
    };
    result?;

    // Re-open the collection at the same path.
    let col = CollectionBuilder::new(&path)
        .set_shared_progress_state(state.progress.clone())
        .build()?;
    *state.col.lock().await = Some(col);

    Ok(())
}
