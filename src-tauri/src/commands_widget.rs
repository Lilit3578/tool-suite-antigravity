// Widget window management command

#[tauri::command]
pub async fn show_widget(app: tauri::AppHandle, widget: String) -> Result<(), String> {
    crate::show_widget_window(&app, &widget)
        .map_err(|e| format!("Failed to show widget window: {}", e))
}
