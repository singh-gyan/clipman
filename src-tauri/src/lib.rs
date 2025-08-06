#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use arboard::Clipboard;
use std::{sync::{Arc, Mutex}, time::Duration, path::PathBuf};
use tauri::{Emitter, Manager};
use tokio::time::sleep;
use crossbeam::channel::bounded;
use rusqlite::{Connection, Result as SqliteResult, params, OptionalExtension};
use chrono::Utc;

#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn copy_to_clipboard(text: String) -> Result<(), String> {
    let mut clipboard = Clipboard::new().map_err(|e| format!("Failed to access clipboard: {}", e))?;
    clipboard.set_text(text).map_err(|e| format!("Failed to copy to clipboard: {}", e))?;
    Ok(())
}

#[derive(serde::Serialize)]
struct JsonValidationResult {
    is_valid: bool,
    error_message: Option<String>,
    formatted_json: Option<String>,
    line_number: Option<usize>,
    column_number: Option<usize>,
}

#[tauri::command]
fn validate_json(json_text: String) -> JsonValidationResult {
    match serde_json::from_str::<serde_json::Value>(&json_text) {
        Ok(parsed) => {
            // Successfully parsed, now format it nicely
            match serde_json::to_string_pretty(&parsed) {
                Ok(formatted) => JsonValidationResult {
                    is_valid: true,
                    error_message: None,
                    formatted_json: Some(formatted),
                    line_number: None,
                    column_number: None,
                },
                Err(_) => JsonValidationResult {
                    is_valid: true,
                    error_message: None,
                    formatted_json: Some(json_text), // Fallback to original
                    line_number: None,
                    column_number: None,
                },
            }
        }
        Err(e) => {
            // Parse the error to extract line and column information
            let error_msg = e.to_string();
            let (line, column) = extract_error_position(&error_msg);
            
            JsonValidationResult {
                is_valid: false,
                error_message: Some(format!("JSON Parse Error: {}", error_msg)),
                formatted_json: None,
                line_number: line,
                column_number: column,
            }
        }
    }
}

fn extract_error_position(error_msg: &str) -> (Option<usize>, Option<usize>) {
    // Extract line and column from serde_json error messages
    // Example: "expected `,` or `}` at line 5 column 10"
    let line_regex = regex::Regex::new(r"line (\d+)").ok();
    let column_regex = regex::Regex::new(r"column (\d+)").ok();
    
    let line = line_regex
        .and_then(|re| re.captures(error_msg))
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());
        
    let column = column_regex
        .and_then(|re| re.captures(error_msg))
        .and_then(|caps| caps.get(1))
        .and_then(|m| m.as_str().parse().ok());
    
    (line, column)
}

#[tauri::command]
fn format_json(json_text: String) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    serde_json::to_string_pretty(&parsed)
        .map_err(|e| format!("Failed to format JSON: {}", e))
}

#[tauri::command]
fn minify_json(json_text: String) -> Result<String, String> {
    let parsed: serde_json::Value = serde_json::from_str(&json_text)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    
    serde_json::to_string(&parsed)
        .map_err(|e| format!("Failed to minify JSON: {}", e))
}

#[derive(serde::Serialize)]
struct DeleteResult {
    success: bool,
    message: String,
}

#[derive(serde::Serialize, Clone)]
struct ClipboardEntry {
    id: i64,
    content: String,
    timestamp: String,
    content_type: String,
}

// Database functions
fn get_db_path(app_handle: &tauri::AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let app_data_dir = app_handle.path().app_data_dir()
        .map_err(|e| format!("Failed to get app data directory: {}", e))?;
    
    // Create the directory if it doesn't exist
    std::fs::create_dir_all(&app_data_dir)
        .map_err(|e| format!("Failed to create app data directory: {}", e))?;
    
    Ok(app_data_dir.join("clipboard_history.db"))
}

fn init_database(app_handle: &tauri::AppHandle) -> SqliteResult<Connection> {
    let db_path = get_db_path(app_handle).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(e.to_string())
        )
    })?;
    
    let conn = Connection::open(db_path)?;
    
    // Create the clipboard_history table
    conn.execute(
        "CREATE TABLE IF NOT EXISTS clipboard_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            content TEXT NOT NULL,
            timestamp TEXT DEFAULT (datetime('now')),
            content_type TEXT DEFAULT 'text'
        )",
        [],
    )?;
    
    // Create an index on timestamp for better query performance
    conn.execute(
        "CREATE INDEX IF NOT EXISTS idx_timestamp ON clipboard_history(timestamp DESC)",
        [],
    )?;
    
    Ok(conn)
}

fn insert_clipboard_entry(app_handle: &tauri::AppHandle, content: &str) -> SqliteResult<()> {
    let db_path = get_db_path(app_handle).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(e.to_string())
        )
    })?;
    
    let conn = Connection::open(db_path)?;
    
    // Don't insert duplicate entries (check last entry)
    let last_content: Option<String> = conn.query_row(
        "SELECT content FROM clipboard_history ORDER BY timestamp DESC LIMIT 1",
        [],
        |row| row.get(0)
    ).optional()?;
    
    if let Some(last) = last_content {
        if last == content {
            return Ok(()); // Skip duplicate
        }
    }
    
    // Determine content type
    let content_type = if content.trim_start().starts_with('{') || content.trim_start().starts_with('[') {
        "json"
    } else if content.starts_with("http://") || content.starts_with("https://") {
        "url"
    } else if content.lines().count() > 1 {
        "multiline"
    } else {
        "text"
    };
    
    let timestamp = Utc::now().to_rfc3339();
    conn.execute(
        "INSERT INTO clipboard_history (content, content_type, timestamp) VALUES (?1, ?2, ?3)",
        params![content, content_type, timestamp],
    )?;
    
    Ok(())
}

fn get_clipboard_history(app_handle: &tauri::AppHandle, limit: usize) -> SqliteResult<Vec<ClipboardEntry>> {
    let db_path = get_db_path(app_handle).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(e.to_string())
        )
    })?;
    
    let conn = Connection::open(db_path)?;
    
    let mut stmt = conn.prepare(
        "SELECT id, content, timestamp, content_type FROM clipboard_history 
         ORDER BY timestamp DESC LIMIT ?1"
    )?;
    
    let entry_iter = stmt.query_map([limit], |row| {
        Ok(ClipboardEntry {
            id: row.get(0)?,
            content: row.get(1)?,
            timestamp: row.get::<_, String>(2).unwrap_or_else(|_| Utc::now().to_rfc3339()),
            content_type: row.get(3)?,
        })
    })?;
    
    let mut entries = Vec::new();
    for entry in entry_iter {
        entries.push(entry?);
    }
    
    Ok(entries)
}

fn delete_clipboard_entry_from_db(app_handle: &tauri::AppHandle, entry_id: i64) -> SqliteResult<()> {
    let db_path = get_db_path(app_handle).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(e.to_string())
        )
    })?;
    
    let conn = Connection::open(db_path)?;
    
    conn.execute(
        "DELETE FROM clipboard_history WHERE id = ?1",
        params![entry_id],
    )?;
    
    Ok(())
}

fn clear_all_clipboard_entries_from_db(app_handle: &tauri::AppHandle) -> SqliteResult<()> {
    let db_path = get_db_path(app_handle).map_err(|e| {
        rusqlite::Error::SqliteFailure(
            rusqlite::ffi::Error::new(rusqlite::ffi::SQLITE_CANTOPEN),
            Some(e.to_string())
        )
    })?;
    
    let conn = Connection::open(db_path)?;
    
    conn.execute("DELETE FROM clipboard_history", [])?;
    
    Ok(())
}

#[tauri::command]
fn delete_clipboard_entry(app: tauri::AppHandle, entry_id: i64) -> DeleteResult {
    // Delete from database
    if let Err(e) = delete_clipboard_entry_from_db(&app, entry_id) {
        return DeleteResult {
            success: false,
            message: format!("Failed to delete entry from database: {}", e),
        };
    }
    
    // Log the deletion for debugging
    println!("Deleted clipboard entry with ID: {}", entry_id);
    
    DeleteResult {
        success: true,
        message: format!("Successfully deleted entry with ID {}", entry_id),
    }
}

#[tauri::command] 
fn clear_all_clipboard_entries(app: tauri::AppHandle) -> DeleteResult {
    // Clear from database
    if let Err(e) = clear_all_clipboard_entries_from_db(&app) {
        return DeleteResult {
            success: false,
            message: format!("Failed to clear entries from database: {}", e),
        };
    }
    
    // Command to clear all clipboard entries
    println!("Cleared all clipboard entries");
    
    DeleteResult {
        success: true,
        message: "All clipboard entries cleared".to_string(),
    }
}

#[tauri::command]
fn get_clipboard_history_entries(app: tauri::AppHandle, limit: Option<usize>) -> Result<Vec<ClipboardEntry>, String> {
    let limit = limit.unwrap_or(20);
    get_clipboard_history(&app, limit).map_err(|e| format!("Failed to get clipboard history: {}", e))
}

#[tauri::command]
fn add_sample_entries(app: tauri::AppHandle) {
    let samples = vec![
        "Hello, this is a sample clipboard entry!".to_string(),
        "const greet = (name: string) => {\n  console.log(`Hello, ${name}!`);\n};".to_string(),
        "https://github.com/tauri-apps/tauri".to_string(),
        r#"{
  "user": {
    "id": 12345,
    "username": "john_doe",
    "email": "john@example.com",
    "profile": {
      "firstName": "John",
      "lastName": "Doe",
      "age": 28,
      "isActive": true,
      "preferences": {
        "theme": "dark",
        "notifications": {
          "email": true,
          "push": false,
          "sms": true
        }
      }
    },
    "roles": ["user", "moderator"],
    "lastLogin": "2024-01-15T10:30:00Z",
    "metadata": null
  }
}"#.to_string(),
        r#"{
  "apiResponse": {
    "status": "success",
    "data": [
      {
        "id": 1,
        "title": "Sample Product",
        "price": 29.99,
        "currency": "USD",
        "inStock": true,
        "tags": ["electronics", "gadget", "popular"],
        "dimensions": {
          "width": 10.5,
          "height": 5.2,
          "depth": 2.1,
          "unit": "cm"
        }
      },
      {
        "id": 2,
        "title": "Another Product",
        "price": 15.50,
        "currency": "USD",
        "inStock": false,
        "tags": ["accessory", "limited"],
        "dimensions": {
          "width": 8.0,
          "height": 3.5,
          "depth": 1.5,
          "unit": "cm"
        }
      }
    ],
    "pagination": {
      "page": 1,
      "totalPages": 5,
      "totalItems": 50,
      "hasNext": true
    }
  }
}"#.to_string(),
        r#"{
  "config": {
    "appName": "Clipboard Manager",
    "version": "2.0.0",
    "environment": "production",
    "features": {
      "jsonViewer": true,
      "searchEnabled": true,
      "autoValidation": true,
      "exportFormats": ["json", "csv", "xml"]
    },
    "database": {
      "host": "localhost",
      "port": 5432,
      "name": "clipman_db",
      "ssl": false
    },
    "logging": {
      "level": "info",
      "outputs": ["console", "file"],
      "maxFileSize": "10MB"
    }
  }
}"#.to_string(),
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Sed do eiusmod tempor incididunt ut labore et dolore magna aliqua.".to_string(),
    ];

    for sample in samples {
        app.emit("clipboard-update", sample).ok();
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|app| {
            let handle = app.handle();

            // Initialize database
            if let Err(e) = init_database(&handle) {
                eprintln!("Failed to initialize database: {}", e);
                return Err(Box::new(e));
            }

            // Load and emit initial clipboard history
            if let Ok(history) = get_clipboard_history(&handle, 20) {
                for entry in history {
                    let _ = handle.emit("clipboard-history", &entry);
                }
            }

            // Channel to signal clipboard updates
            let (tx, rx) = bounded::<String>(10);
            let last_clipboard = Arc::new(Mutex::new(String::new()));

            // Spawn background clipboard watcher
            let watcher_handle = handle.clone();
            let last_clipboard_clone = last_clipboard.clone();
            std::thread::spawn(move || {
                let mut clipboard = Clipboard::new().unwrap();
                loop {
                    if let Ok(current) = clipboard.get_text() {
                        let mut last = last_clipboard_clone.lock().unwrap();
                        if *last != current {
                            *last = current.clone();
                            let _ = tx.send(current);
                        }
                    }
                    std::thread::sleep(Duration::from_millis(1000));
                }
            });

            // Spawn async event emitter using tokio
            let db_handle = handle.clone();
            tauri::async_runtime::spawn(async move {
                while let Ok(entry) = rx.recv() {
                    // Save to database asynchronously
                    let db_handle_clone = db_handle.clone();
                    let entry_clone = entry.clone();
                    tokio::task::spawn_blocking(move || {
                        if let Err(e) = insert_clipboard_entry(&db_handle_clone, &entry_clone) {
                            eprintln!("Failed to save clipboard entry to database: {}", e);
                        }
                    });
                    
                    watcher_handle.emit("clipboard-update", entry).unwrap();
                    sleep(Duration::from_millis(200)).await;
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![greet, add_sample_entries, copy_to_clipboard, validate_json, format_json, minify_json, delete_clipboard_entry, clear_all_clipboard_entries, get_clipboard_history_entries])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
