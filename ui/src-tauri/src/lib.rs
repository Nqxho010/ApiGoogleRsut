use gmail_rust::{EmailPreview, Hub, TrashReport};
use tauri::Manager;

// Guarda la conexion a Gmail para que la usen todos los comandos
#[derive(Default)]
struct AppState(tokio::sync::Mutex<Option<Hub>>);

#[tauri::command]
async fn connect(
    app: tauri::AppHandle,
    state: tauri::State<'_, AppState>,
) -> Result<String, String> {
    let dir = app.path().app_data_dir().map_err(|e| e.to_string())?;
    let credentials = dir.join("credentials.json");
    let token = dir.join("token.json");

    let hub = gmail_rust::build_hub(&credentials.to_string_lossy(), &token.to_string_lossy())
        .await
        .map_err(|e| e.to_string())?;

    let email = gmail_rust::profile_email(&hub)
        .await
        .map_err(|e| e.to_string())?;

    *state.0.lock().await = Some(hub);

    Ok(email)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .manage(AppState::default())
        .invoke_handler(tauri::generate_handler![connect, search, previews, trash])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn search(query: String, state: tauri::State<'_, AppState>) -> Result<Vec<String>, String> {
    let hub = state.0.lock().await.clone().ok_or("No se ha conectado a Gmail")?;

    gmail_rust::search_messages(&hub, &query)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
async fn previews(
    ids: Vec<String>,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<EmailPreview>, String> {
    let hub = state.0.lock().await.clone().ok_or("No se ha conectado a Gmail")?;
    let mut emails = Vec::new();

    for id in &ids {
        emails.push(
            gmail_rust::fetch_preview(&hub, id)
                .await
                .map_err(|e| e.to_string())?,
        );
    }

    Ok(emails)
}

#[tauri::command]
async fn trash(ids: Vec<String>, state: tauri::State<'_, AppState>) -> Result<TrashReport, String> {
    let hub = state.0.lock().await.clone().ok_or("No se ha conectado a Gmail")?;

    Ok(gmail_rust::trash_messages(&hub, &ids).await)
}