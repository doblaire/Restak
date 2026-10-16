mod scanner;
mod steamgriddb;
mod process_utils;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct GameDetails {
    pub name: Option<String>,
    pub cover_url: Option<String>,
    #[serde(default)]
    pub all_covers: Vec<String>,
    pub genre: Option<String>,
    pub publisher: Option<String>,
    pub year: Option<String>,
    pub description: Option<String>,
}

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Emitter, Manager};

use scanner::Game;

fn library_file_path() -> Option<PathBuf> {
    scanner::user_home_dir().map(|h| h.join(".restak_library.json"))
}

#[tauri::command]
async fn scan_for_games() -> Result<Vec<Game>, String> {
    Ok(scanner::scan_for_games().await)
}

#[tauri::command]
fn load_library() -> Result<Vec<Game>, String> {
    if let Some(path) = library_file_path() {
        if path.exists() {
            if let Ok(content) = fs::read_to_string(&path) {
                if let Ok(games) = serde_json::from_str::<Vec<Game>>(&content) {
                    return Ok(games);
                }
            }
        }
    }
    Ok(Vec::new())
}

#[tauri::command]
fn save_library(games: Vec<Game>) -> Result<(), String> {
    if let Some(path) = library_file_path() {
        let json = serde_json::to_string_pretty(&games).map_err(|e| e.to_string())?;
        fs::write(path, json).map_err(|e| e.to_string())?;
        return Ok(());
    }
    Err("Could not determine home directory".to_string())
}

#[tauri::command]
fn pick_launcher() -> Result<Option<String>, String> {
    #[cfg(unix)]
    {
        let script = r#"
            try
                set appPath to POSIX path of (choose file of type {"app"} with prompt "Select Game Launcher / Emulator" default location (POSIX file "/Applications"))
                return appPath
            on error
                return ""
            end try
        "#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    #[cfg(windows)]
    {
        let script = r#"
            Add-Type -AssemblyName System.Windows.Forms
            $dialog = New-Object System.Windows.Forms.OpenFileDialog
            $dialog.Filter = "Executable Files (*.exe)|*.exe|All Files (*.*)|*.*"
            $dialog.Title = "Select Game Launcher / Emulator"
            if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
                Write-Output $dialog.FileName
            }
        "#;
        let output = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

#[tauri::command]
fn pick_game_file() -> Result<Option<String>, String> {
    #[cfg(unix)]
    {
        let script = r#"
            try
                set filePath to POSIX path of (choose file with prompt "Select Game File")
                return filePath
            on error
                return ""
            end try
        "#;
        let output = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }

    #[cfg(windows)]
    {
        let script = r#"
            Add-Type -AssemblyName System.Windows.Forms
            $dialog = New-Object System.Windows.Forms.OpenFileDialog
            $dialog.Filter = "All Files (*.*)|*.*|Game Executables (*.exe)|*.exe|ROM Files (*.gba;*.gbc;*.gb;*.nds;*.n64;*.z64;*.v64;*.gcm;*.iso;*.nes;*.snes;*.smc;*.sfc;*.nsp;*.xci;*.3ds;*.cue;*.bin)|*.gba;*.gbc;*.gb;*.nds;*.n64;*.z64;*.v64;*.gcm;*.iso;*.nes;*.snes;*.smc;*.sfc;*.nsp;*.xci;*.3ds;*.cue;*.bin"
            $dialog.Title = "Select Game File"
            if ($dialog.ShowDialog() -eq [System.Windows.Forms.DialogResult]::OK) {
                Write-Output $dialog.FileName
            }
        "#;
        let output = Command::new("powershell")
            .arg("-NoProfile")
            .arg("-Command")
            .arg(script)
            .output()
            .map_err(|e| e.to_string())?;

        let result = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if result.is_empty() {
            Ok(None)
        } else {
            Ok(Some(result))
        }
    }
}

#[tauri::command]
async fn lookup_game_info(title: String) -> Result<Option<GameDetails>, String> {
    let client = reqwest::Client::new();
    // 1. Check local / known metadata first
    let (pub_opt, yr_opt, gen_opt) = scanner::lookup_game_metadata(&title);
    
    // 2. Query SteamGridDB for high-quality vertical 600x900 covers via Cloudflare Worker
    let sgdb_covers = steamgriddb::get_covers(&client, &title).await;

    if pub_opt.is_some() || yr_opt.is_some() || gen_opt.is_some() || !sgdb_covers.is_empty() {
        let primary_cover = sgdb_covers.first().cloned();
        Ok(Some(GameDetails {
            name: Some(title),
            cover_url: primary_cover,
            all_covers: sgdb_covers,
            genre: gen_opt.map(|s| s.to_string()),
            publisher: pub_opt.map(|s| s.to_string()),
            year: yr_opt.map(|s| s.to_string()),
            description: None,
        }))
    } else {
        Ok(None)
    }
}


fn find_app_in_dir(dir: &Path) -> Option<PathBuf> {
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let p = entry.path();
                if p.extension().map_or(false, |ext| ext == "app") {
                    return Some(p);
                }
            }
        }
    }
    None
}

fn find_rom_in_dir(game_name: &str, dir: &Path) -> Option<PathBuf> {
    if dir.is_file() {
        return Some(dir.to_path_buf());
    }
    if dir.is_dir() {
        if let Ok(entries) = fs::read_dir(dir) {
            let rom_exts = [
                ".iso", ".gcm", ".rvz", ".ciso", ".wbfs", ".nkit.iso",
                ".z64", ".n64", ".v64", ".gba", ".gbc", ".gb", ".nds",
                ".nes", ".snes", ".smc", ".sfc", ".nsp", ".xci", ".3ds", ".cue", ".bin"
            ];
            let lower_name = game_name.to_lowercase();
            let mut fallback: Option<PathBuf> = None;

            for entry in entries.flatten() {
                let p = entry.path();
                if p.is_file() {
                    let fname = p.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
                    for ext in &rom_exts {
                        if fname.ends_with(ext) {
                            if !lower_name.is_empty() && (fname.contains(&lower_name) || lower_name.contains(&fname)) {
                                return Some(p);
                            }
                            if fallback.is_none() {
                                fallback = Some(p.clone());
                            }
                        }
                    }
                }
            }
            return fallback;
        }
    }
    None
}

fn find_steam_app_id(game_name: &str, game_path: &str) -> Option<String> {
    let steam_folders = scanner::get_steam_library_folders();
    let path_obj = Path::new(game_path);
    let dir_name = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let lower_name = game_name.to_lowercase();

    for steam_apps in steam_folders {
        if let Ok(entries) = fs::read_dir(&steam_apps) {
            for entry in entries.flatten() {
                let p = entry.path();
                if let Some(fname) = p.file_name().and_then(|s| s.to_str()) {
                    if fname.starts_with("appmanifest_") && fname.ends_with(".acf") {
                        if let Ok(content) = fs::read_to_string(&p) {
                            let mut found_id: Option<String> = None;
                            let mut matches = false;

                            for line in content.lines() {
                                let trimmed = line.trim();
                                if trimmed.starts_with("\"appid\"") {
                                    let parts: Vec<&str> = trimmed.split('"').collect();
                                    if parts.len() >= 4 {
                                        found_id = Some(parts[3].to_string());
                                    }
                                } else if trimmed.starts_with("\"name\"") {
                                    let parts: Vec<&str> = trimmed.split('"').collect();
                                    if parts.len() >= 4 {
                                        if parts[3].to_lowercase() == lower_name {
                                            matches = true;
                                        }
                                    }
                                } else if trimmed.starts_with("\"installdir\"") {
                                    let parts: Vec<&str> = trimmed.split('"').collect();
                                    if parts.len() >= 4 {
                                        let idir = parts[3].to_lowercase();
                                        if !dir_name.is_empty() && (dir_name == idir || dir_name.contains(&idir) || idir.contains(&dir_name)) {
                                            matches = true;
                                        }
                                    }
                                }
                            }

                            if matches {
                                if let Some(id) = found_id {
                                    return Some(id);
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

static IS_BORDERLESS_FS: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);
static PREV_POSITION: std::sync::Mutex<Option<tauri::PhysicalPosition<i32>>> = std::sync::Mutex::new(None);
static PREV_SIZE: std::sync::Mutex<Option<tauri::PhysicalSize<u32>>> = std::sync::Mutex::new(None);

#[tauri::command]
fn toggle_borderless_fullscreen(app_handle: tauri::AppHandle) -> Result<bool, String> {
    if let Some(window) = app_handle.get_webview_window("main") {
        let is_fs = IS_BORDERLESS_FS.load(std::sync::atomic::Ordering::SeqCst);
        if !is_fs {
            // Save current position and size
            if let Ok(pos) = window.outer_position() {
                *PREV_POSITION.lock().unwrap() = Some(pos);
            }
            if let Ok(size) = window.outer_size() {
                *PREV_SIZE.lock().unwrap() = Some(size);
            }

            // If window was in native fullscreen, exit it first
            if window.is_fullscreen().unwrap_or(false) {
                let _ = window.set_fullscreen(false);
                std::thread::sleep(std::time::Duration::from_millis(300));
            }

            if let Ok(Some(monitor)) = window.current_monitor() {
                let m_pos = monitor.position();
                let m_size = monitor.size();
                let _ = window.set_decorations(false);
                let _ = window.set_position(*m_pos);
                let _ = window.set_size(*m_size);
            } else {
                let _ = window.set_decorations(false);
                let _ = window.maximize();
            }
            IS_BORDERLESS_FS.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = window.emit("fullscreen-changed", true);
            Ok(true)
        } else {
            // Exit borderless fullscreen: restore decorations and size
            let _ = window.set_decorations(true);
            if let Some(pos) = PREV_POSITION.lock().unwrap().take() {
                let _ = window.set_position(pos);
            }
            if let Some(size) = PREV_SIZE.lock().unwrap().take() {
                let _ = window.set_size(size);
            } else {
                let _ = window.set_size(tauri::LogicalSize::new(1920, 1080));
                let _ = window.center();
            }
            IS_BORDERLESS_FS.store(false, std::sync::atomic::Ordering::SeqCst);
            let _ = window.emit("fullscreen-changed", false);
            Ok(false)
        }
    } else {
        Err("Main window not found".to_string())
    }
}

#[tauri::command]
fn get_fullscreen_state() -> Result<bool, String> {
    Ok(IS_BORDERLESS_FS.load(std::sync::atomic::Ordering::SeqCst))
}




#[cfg(unix)]
fn trigger_unix_game_launch(
    custom_path: &str,
    launcher_lower: &str,
    full_path: &str,
    game_name: &str,
    game_path_str: &str,
) {
    if !custom_path.is_empty() {
        if custom_path.ends_with(".app") {
            let _ = Command::new("open").arg("-a").arg(custom_path).arg("--args").arg(full_path).spawn();
        } else {
            let _ = Command::new("open").arg(custom_path).arg("--args").arg(full_path).spawn();
        }
    } else if launcher_lower.contains("dolphin") {
        let _ = Command::new("open")
            .arg("-a")
            .arg("Dolphin")
            .arg("--args")
            .arg("-b")
            .arg("-e")
            .arg(full_path)
            .spawn();
    } else if launcher_lower.contains("steam") || full_path.starts_with("steam://") {
        if !process_utils::is_steam_running() {
            let _ = Command::new("open").arg("-a").arg("Steam").spawn();
        }

        let app_in_dir = find_app_in_dir(Path::new(game_path_str))
            .or_else(|| if full_path.ends_with(".app") { Some(PathBuf::from(full_path)) } else { None });

        let app_id: Option<String> = if full_path.starts_with("steam://run/") {
            full_path.strip_prefix("steam://run/").map(|s| s.trim_matches('/').to_string())
        } else if full_path.starts_with("steam://rungameid/") {
            full_path.strip_prefix("steam://rungameid/").map(|s| s.trim_matches('/').to_string())
        } else {
            find_steam_app_id(game_name, game_path_str)
                .or_else(|| find_steam_app_id(game_name, full_path))
        };

        if let Some(ref id) = app_id {
            let url = format!("steam://run/{}", id);
            let _ = Command::new("open").arg(&url).spawn();
        } else if let Some(app_path) = app_in_dir {
            let _ = Command::new("open").arg(&app_path).spawn();
        } else if full_path.ends_with(".app") {
            let _ = Command::new("open").arg(full_path).spawn();
        } else {
            let _ = Command::new("open").arg("steam://open").spawn();
        }
    } else if full_path.ends_with(".app") || launcher_lower == "macos" {
        let _ = Command::new("open").arg(full_path).spawn();
    } else {
        let _ = Command::new("open").arg(full_path).spawn();
    }
}

#[tauri::command]
async fn launch_game(app: tauri::AppHandle, game: Game) -> Result<(), String> {
    let custom_path = game.custom_launcher_path.clone().unwrap_or_default();
    let raw_full_path = game.full_path.clone().unwrap_or_default();
    let game_path_str = game.path.clone();
    let launcher_lower = game.launcher.to_lowercase();
    let game_name = game.name.clone();

    // Dynamically resolve exact ROM file path if full_path is unpopulated or points to a folder
    let full_path = if !raw_full_path.is_empty() && Path::new(&raw_full_path).is_file() {
        raw_full_path
    } else if let Some(rom_p) = find_rom_in_dir(&game_name, Path::new(&game_path_str)) {
        rom_p.to_string_lossy().to_string()
    } else if !raw_full_path.is_empty() {
        raw_full_path
    } else {
        game_path_str.clone()
    };

    #[cfg(unix)]
    let mut startup_targets = Vec::new();

    #[cfg(unix)]
    {
        if !custom_path.is_empty() {
            let p = Path::new(&custom_path);
            if custom_path.ends_with(".app") {
                for exe in process_utils::get_app_bundle_executables(p) {
                    if !startup_targets.contains(&exe) {
                        startup_targets.push(exe);
                    }
                }
            } else if let Some(stem) = p.file_stem().and_then(|s| s.to_str()) {
                startup_targets.push(stem.to_string());
            }
        }

        if full_path.ends_with(".app") {
            for exe in process_utils::get_app_bundle_executables(Path::new(&full_path)) {
                if !startup_targets.contains(&exe) {
                    startup_targets.push(exe);
                }
            }
        }

        if let Some(app_path) = find_app_in_dir(Path::new(&game_path_str)) {
            for exe in process_utils::get_app_bundle_executables(&app_path) {
                if !startup_targets.contains(&exe) {
                    startup_targets.push(exe);
                }
            }
        }

        if !startup_targets.contains(&game_name) {
            startup_targets.push(game_name.clone());
        }
        let no_space = game_name.replace(' ', "");
        if !no_space.is_empty() && !startup_targets.contains(&no_space) {
            startup_targets.push(no_space);
        }
        let no_special: String = game_name.chars().filter(|c| c.is_alphanumeric()).collect();
        if !no_special.is_empty() && !startup_targets.contains(&no_special) {
            startup_targets.push(no_special);
        }
    }

    #[cfg(unix)]
    {
        trigger_unix_game_launch(
            &custom_path,
            &launcher_lower,
            &full_path,
            &game_name,
            &game_path_str,
        );
    }

    #[cfg(windows)]
    {
        let is_steam = launcher_lower.contains("steam") || full_path.starts_with("steam://");
        if is_steam && !full_path.ends_with(".exe") {
            let app_id: Option<String> = if full_path.starts_with("steam://run/") {
                full_path.strip_prefix("steam://run/").map(|s| s.trim_matches('/').to_string())
            } else if full_path.starts_with("steam://rungameid/") {
                full_path.strip_prefix("steam://rungameid/").map(|s| s.trim_matches('/').to_string())
            } else {
                find_steam_app_id(&game_name, &game_path_str)
                    .or_else(|| find_steam_app_id(&game_name, &full_path))
            };

            let url = if let Some(ref id) = app_id {
                format!("steam://run/{}", id)
            } else if full_path.starts_with("steam://") {
                full_path.clone()
            } else {
                "steam://open".to_string()
            };
            let _ = Command::new("cmd").arg("/c").arg("start").arg("").arg(&url).spawn();
        } else if !custom_path.is_empty() {
            let mut cmd = Command::new(&custom_path);
            if !full_path.is_empty() {
                cmd.arg(&full_path);
            }
            let _ = cmd.spawn();
        } else if full_path.ends_with(".exe") {
            let _ = Command::new(&full_path).spawn();
        } else {
            let _ = Command::new("cmd").arg("/c").arg("start").arg("").arg(&full_path).spawn();
        }
    }

    let is_steam = launcher_lower.contains("steam") || full_path.starts_with("steam://");
    
    #[cfg(unix)]
    let custom_path_t = custom_path.clone();
    #[cfg(unix)]
    let launcher_lower_t = launcher_lower.clone();
    #[cfg(unix)]
    let full_path_t = full_path.clone();
    #[cfg(unix)]
    let game_name_t = game_name.clone();
    #[cfg(unix)]
    let game_path_str_t = game_path_str.clone();

    std::thread::spawn(move || {
        #[cfg(unix)]
        {
            if is_steam {
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.set_always_on_top(true);
                }
            }

            let mut target_pid = None;
            let mut retried = false;
            let start_time = std::time::Instant::now();

            // Phase 1: High-frequency polling (30ms) for the game process to appear
            while start_time.elapsed().as_secs() < 20 {
                if let Some(pid) = process_utils::get_target_pid(&startup_targets) {
                    target_pid = Some(pid);
                    break;
                }

                if is_steam {
                    process_utils::hide_steam_window();
                }

                // If after 2.5s the game is not yet running and we haven't retried, retry launch once
                if !retried && start_time.elapsed().as_millis() >= 2500 {
                    retried = true;
                    trigger_unix_game_launch(
                        &custom_path_t,
                        &launcher_lower_t,
                        &full_path_t,
                        &game_name_t,
                        &game_path_str_t,
                    );
                }

                std::thread::sleep(std::time::Duration::from_millis(30));
            }

            if is_steam {
                process_utils::hide_steam_window();
                if let Some(main_win) = app.get_webview_window("main") {
                    let _ = main_win.set_always_on_top(false);
                }
            }

            // Phase 2: Game is running -> 16ms high-frequency exit check
            if let Some(pid) = target_pid {
                loop {
                    if !process_utils::is_pid_running(pid) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(16));
                }
            } else {
                std::thread::sleep(std::time::Duration::from_secs(8));
            }

            // Phase 3: Game exited -> bring Restak immediately to the front
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.show();
                let _ = main_win.set_focus();
                let _ = main_win.set_always_on_top(true);
                std::thread::sleep(std::time::Duration::from_millis(100));
                let _ = main_win.set_always_on_top(false);
            }
        }

        #[cfg(windows)]
        {
            std::thread::sleep(std::time::Duration::from_secs(3));
            if let Some(main_win) = app.get_webview_window("main") {
                let _ = main_win.show();
                let _ = main_win.set_focus();
            }
        }
    });

    Ok(())
}

#[tauri::command]
fn show_file_location(path: String) -> Result<(), String> {
    #[cfg(target_os = "macos")]
    {
        let p = PathBuf::from(&path);
        if p.exists() {
            let _ = Command::new("open").arg("-R").arg(&path).spawn();
        } else if let Some(parent) = p.parent() {
            if parent.exists() {
                let _ = Command::new("open").arg(parent).spawn();
            }
        }
        Ok(())
    }
    #[cfg(target_os = "windows")]
    {
        let _ = Command::new("explorer").arg(format!("/select,\"{}\"", path)).spawn();
        Ok(())
    }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    {
        let _ = path;
        Ok(())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            scan_for_games,
            load_library,
            save_library,
            pick_launcher,
            pick_game_file,
            lookup_game_info,
            launch_game,
            show_file_location,
            toggle_borderless_fullscreen,
            get_fullscreen_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
