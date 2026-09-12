mod scanner;
mod steamgriddb;

use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tauri::{Emitter, Manager};

use scanner::Game;

fn library_file_path() -> Option<PathBuf> {
    std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".restak_library.json"))
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

fn get_bundle_executable(app_path: &std::path::Path) -> Option<String> {
    let plist_path = app_path.join("Contents/Info.plist");
    if let Ok(content) = fs::read_to_string(&plist_path) {
        if let Some(pos) = content.find("CFBundleExecutable") {
            let remainder = &content[pos..];
            if let Some(str_start) = remainder.find("<string>") {
                let after_str = &remainder[str_start + 8..];
                if let Some(str_end) = after_str.find("</string>") {
                    let exec_name = after_str[..str_end].trim().to_string();
                    if !exec_name.is_empty() {
                        return Some(exec_name);
                    }
                }
            }
        }
    }
    None
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

fn _check_steam_manifest_status(app_id: &str) -> Option<(u32, u64, u64)> {
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let p = home
        .join("Library/Application Support/Steam/steamapps")
        .join(format!("appmanifest_{}.acf", app_id));
    if !p.exists() {
        return None;
    }
    let content = fs::read_to_string(p).ok()?;
    let mut state_flags = 4;
    let mut downloaded = 0;
    let mut to_download = 0;
    for line in content.lines() {
        let t = line.trim();
        if t.starts_with("\"StateFlags\"") {
            let parts: Vec<&str> = t.split('"').collect();
            if parts.len() >= 4 {
                state_flags = parts[3].parse().unwrap_or(4);
            }
        } else if t.starts_with("\"BytesDownloaded\"") {
            let parts: Vec<&str> = t.split('"').collect();
            if parts.len() >= 4 {
                downloaded = parts[3].parse().unwrap_or(0);
            }
        } else if t.starts_with("\"BytesToDownload\"") {
            let parts: Vec<&str> = t.split('"').collect();
            if parts.len() >= 4 {
                to_download = parts[3].parse().unwrap_or(0);
            }
        }
    }
    Some((state_flags, downloaded, to_download))
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
    let home = std::env::var_os("HOME").map(PathBuf::from)?;
    let steam_apps = home.join("Library/Application Support/Steam/steamapps");
    if !steam_apps.exists() {
        return None;
    }

    let entries = fs::read_dir(&steam_apps).ok()?;
    let path_obj = Path::new(game_path);
    let dir_name = path_obj.file_name().and_then(|s| s.to_str()).unwrap_or("").to_lowercase();
    let lower_name = game_name.to_lowercase();

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
    None
}

#[cfg(unix)]
fn is_any_target_running(targets: &[String]) -> bool {
    for target in targets {
        if target.is_empty() {
            continue;
        }
        if let Ok(output) = Command::new("pgrep").arg("-xi").arg(target).output() {
            if output.status.success() {
                return true;
            }
        }
    }
    false
}

#[cfg(windows)]
fn is_any_target_running(targets: &[String]) -> bool {
    for target in targets {
        if target.is_empty() {
            continue;
        }
        let target_exe = if target.to_lowercase().ends_with(".exe") {
            target.to_string()
        } else {
            format!("{}.exe", target)
        };
        if let Ok(output) = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("IMAGENAME eq {}", target_exe))
            .arg("/NH")
            .output()
        {
            let out = String::from_utf8_lossy(&output.stdout);
            if out.to_lowercase().contains(&target_exe.to_lowercase()) {
                return true;
            }
        }
    }
    false
}

#[cfg(unix)]
fn get_target_pid(targets: &[String]) -> Option<u32> {
    for target in targets {
        if target.is_empty() {
            continue;
        }
        if let Ok(output) = Command::new("pgrep").arg("-xi").arg(target).output() {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                if let Some(first_line) = stdout.lines().next() {
                    if let Ok(pid) = first_line.trim().parse::<u32>() {
                        return Some(pid);
                    }
                }
            }
        }
    }
    None
}

#[cfg(windows)]
fn get_target_pid(targets: &[String]) -> Option<u32> {
    for target in targets {
        if target.is_empty() {
            continue;
        }
        let target_exe = if target.to_lowercase().ends_with(".exe") {
            target.to_string()
        } else {
            format!("{}.exe", target)
        };
        if let Ok(output) = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("IMAGENAME eq {}", target_exe))
            .arg("/FO")
            .arg("CSV")
            .arg("/NH")
            .output()
        {
            let out = String::from_utf8_lossy(&output.stdout);
            for line in out.lines() {
                let parts: Vec<&str> = line.split(',').collect();
                if parts.len() >= 2 {
                    let pid_str = parts[1].trim_matches('"').trim();
                    if let Ok(pid) = pid_str.parse::<u32>() {
                        return Some(pid);
                    }
                }
            }
        }
    }
    None
}

/// Instant, 0-CPU process check using native OS kill(pid, 0) system call on Unix, or tasklist on Windows.
fn is_pid_running(pid: u32) -> bool {
    #[cfg(unix)]
    unsafe {
        libc::kill(pid as i32, 0) == 0
    }
    #[cfg(windows)]
    {
        if let Ok(output) = Command::new("tasklist")
            .arg("/FI")
            .arg(format!("PID eq {}", pid))
            .arg("/NH")
            .output()
        {
            let out = String::from_utf8_lossy(&output.stdout);
            out.contains(&pid.to_string())
        } else {
            false
        }
    }
}

/// More aggressive check that also searches command-line args (pgrep -f).
/// Used only after the game has already been confirmed running once.
fn is_any_target_running_broad(targets: &[String]) -> bool {
    for target in targets {
        if target.is_empty() {
            continue;
        }
        if let Ok(output) = Command::new("pgrep").arg("-xi").arg(target).output() {
            if output.status.success() {
                return true;
            }
        }
        // Fall back to command-line search for targets that are unlikely to be process names
        if let Ok(output) = Command::new("pgrep").arg("-fi").arg(target).output() {
            if output.status.success() {
                // Filter out matches that are only Steam helper processes
                let pids = String::from_utf8_lossy(&output.stdout);
                for pid_str in pids.lines() {
                    if let Ok(pid) = pid_str.trim().parse::<u32>() {
                        // Check what the actual process is
                        if let Ok(ps_out) = Command::new("ps")
                            .arg("-p")
                            .arg(pid.to_string())
                            .arg("-o")
                            .arg("comm=")
                            .output()
                        {
                            let comm = String::from_utf8_lossy(&ps_out.stdout).trim().to_lowercase();
                            // Skip if this is just Steam itself or a Steam helper
                            if comm.contains("steam") || comm.contains("steamwebhelper") || comm.contains("steam_osx") {
                                continue;
                            }
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

/// Snapshot all running PIDs for before/after comparison.
fn get_running_pids() -> HashSet<String> {
    let mut pids = HashSet::new();
    if let Ok(output) = Command::new("ps").arg("-eo").arg("pid=").output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let pid = line.trim().to_string();
            if !pid.is_empty() {
                pids.insert(pid);
            }
        }
    }
    pids
}

#[derive(Clone, serde::Serialize)]
struct GameStatusPayload {
    status: String,
    detail: String,
}

#[tauri::command]
fn enter_launch_mode(_app_handle: tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn prepare_game_reveal(_app_handle: tauri::AppHandle) -> Result<(), String> {
    Ok(())
}

#[tauri::command]
fn complete_game_reveal(_app_handle: tauri::AppHandle) -> Result<(), String> {
    Ok(())
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
                let _ = window.set_always_on_top(true);
            } else {
                let _ = window.set_decorations(false);
                let _ = window.maximize();
                let _ = window.set_always_on_top(true);
            }
            IS_BORDERLESS_FS.store(true, std::sync::atomic::Ordering::SeqCst);
            let _ = window.emit("fullscreen-changed", true);
            Ok(true)
        } else {
            // Exit borderless fullscreen: restore decorations and size
            let _ = window.set_always_on_top(false);
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

#[tauri::command]
async fn launch_game(app_handle: tauri::AppHandle, game: Game) -> Result<(), String> {
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

    // If in native macOS fullscreen (green button), drop it so we're on the desktop space.
    let is_borderless = IS_BORDERLESS_FS.load(std::sync::atomic::Ordering::SeqCst);
    let was_native_fullscreen = if is_borderless {
        false
    } else if let Some(window) = app_handle.get_webview_window("main") {
        let is_fs = window.is_fullscreen().unwrap_or(false);
        if is_fs {
            let _ = window.set_fullscreen(false);
        }
        is_fs
    } else {
        false
    };

    // KIOSK OVERLAY: Raise Restak above ALL other windows.
    if let Some(window) = app_handle.get_webview_window("main") {
        let _ = window.set_always_on_top(true);
    }

    // ── Build TWO target lists ──────────────────────────────────────────
    //
    // startup_targets: ONLY the actual game executable name.
    //   Used to detect when the real game binary is running.
    //   Does NOT include launcher/emulator names (Steam, Dolphin, etc.)
    //
    // monitor_targets: Broader list including game name variations.
    //   Used after the game is confirmed running to detect when it exits.
    // ────────────────────────────────────────────────────────────────────

    let mut startup_targets: Vec<String> = Vec::new();
    let mut monitor_targets: Vec<String> = Vec::new();

    // 1. Extract the actual game executable from .app bundles
    //    This is the REAL binary that runs when the game is playing.

    // Check inside the game's install directory for a .app bundle
    let game_p = std::path::Path::new(&game_path_str);
    if let Some(app_in_dir) = find_app_in_dir(game_p) {
        if let Some(exec) = get_bundle_executable(&app_in_dir) {
            startup_targets.push(exec.clone());
            monitor_targets.push(exec);
        }
        if let Some(stem) = app_in_dir.file_stem().and_then(|s| s.to_str()) {
            startup_targets.push(stem.to_string());
            monitor_targets.push(stem.to_string());
        }
    }

    // Check full_path if it's a .app
    if full_path.ends_with(".app") {
        let full_p = std::path::Path::new(&full_path);
        if let Some(exec) = get_bundle_executable(full_p) {
            startup_targets.push(exec.clone());
            monitor_targets.push(exec);
        }
        if let Some(stem) = full_p.file_stem().and_then(|s| s.to_str()) {
            startup_targets.push(stem.to_string());
            monitor_targets.push(stem.to_string());
        }
    }

    // Directory stem (e.g., "BloonsTD6" from steamapps/common/BloonsTD6)
    if let Some(stem) = game_p.file_stem().and_then(|s| s.to_str()) {
        if !stem.is_empty() && stem.to_lowercase() != "common" {
            startup_targets.push(stem.to_string());
            monitor_targets.push(stem.to_string());
        }
    }

    // Game name and spaceless variation — only for monitoring, NOT startup
    if !game_name.is_empty() {
        monitor_targets.push(game_name.clone());
        let spaceless = game_name.replace(' ', "");
        if spaceless != game_name {
            startup_targets.push(spaceless.clone());
            monitor_targets.push(spaceless);
        }
    }

    // 2. For emulator games: the emulator IS the game runtime.
    //    But only add to startup_targets — not the custom launcher stem.
    let is_emulator = launcher_lower.contains("dolphin")
        || launcher_lower.contains("retroarch")
        || launcher_lower.contains("mgba")
        || launcher_lower.contains("desmume")
        || launcher_lower.contains("citra")
        || launcher_lower.contains("ryujinx")
        || launcher_lower.contains("yuzu")
        || launcher_lower.contains("openemu");

    if is_emulator {
        // For emulators, the emulator process IS what runs the game
        if launcher_lower.contains("dolphin") {
            startup_targets.push("Dolphin".to_string());
            monitor_targets.push("Dolphin".to_string());
        }
        if launcher_lower.contains("retroarch") {
            startup_targets.push("retroarch".to_string());
            monitor_targets.push("retroarch".to_string());
        }
        if launcher_lower.contains("mgba") {
            startup_targets.push("mGBA".to_string());
            monitor_targets.push("mGBA".to_string());
        }
    }

    // 3. Custom launcher — add ONLY to monitor (not startup).
    //    The launcher opening doesn't mean the game is running.
    if !custom_path.is_empty() {
        let custom_p = std::path::Path::new(&custom_path);
        if let Some(stem) = custom_p.file_stem().and_then(|s| s.to_str()) {
            monitor_targets.push(stem.to_string());
        }
        if custom_path.ends_with(".app") {
            if let Some(exec) = get_bundle_executable(custom_p) {
                monitor_targets.push(exec);
            }
        }
    }

    // Blacklist and deduplicate both lists
    let blacklist = ["steam", "steam_osx", "steamwebhelper", "macos", "open", "restak", "app", "common", ""];
    for list in [&mut startup_targets, &mut monitor_targets] {
        list.retain(|t| {
            let l = t.to_lowercase();
            !blacklist.contains(&l.as_str()) && t.len() >= 3
        });
        list.sort();
        list.dedup();
    }

    let exit_handle = app_handle.clone();
    let exit_game_name = game_name.clone();

    // Take a snapshot of running processes BEFORE launching
    let pre_launch_procs = get_running_pids();

    std::thread::spawn(move || {
        let is_steam = launcher_lower.contains("steam") || full_path.starts_with("steam://");

        let _ = exit_handle.emit("game-status", GameStatusPayload {
            status: "launching".to_string(),
            detail: if is_steam {
                "Launching via Steam...".to_string()
            } else {
                format!("Launching {}...", exit_game_name)
            },
        });

        if was_native_fullscreen {
            std::thread::sleep(std::time::Duration::from_millis(450));
        }

        #[cfg(unix)]
        {
            // Spawn the game process via LaunchServices (open -a) so macOS TCC evaluates the target app's established permissions
            if !custom_path.is_empty() {
                if custom_path.ends_with(".app") {
                    let _ = Command::new("open").arg("-a").arg(&custom_path).arg("--args").arg(&full_path).spawn();
                } else {
                    let _ = Command::new("open").arg(&custom_path).arg("--args").arg(&full_path).spawn();
                }
            } else if launcher_lower.contains("dolphin") {
                // Launch Dolphin via system LaunchServices (`open -a Dolphin --args -b -e ...`)
                let _ = Command::new("open")
                    .arg("-a")
                    .arg("Dolphin")
                    .arg("--args")
                    .arg("-b")
                    .arg("-e")
                    .arg(&full_path)
                    .spawn();
            } else if is_steam {
                // Ensure Steam client is running so SteamAPI_Init() succeeds without crashing the game
                let is_steam_running = Command::new("pgrep").arg("-xi").arg("steam_osx").output().map_or(false, |o| o.status.success());
                if !is_steam_running {
                    let _ = exit_handle.emit("game-status", GameStatusPayload {
                        status: "launching".to_string(),
                        detail: "Starting Steam...".to_string(),
                    });
                    let _ = Command::new("open").arg("-a").arg("Steam").spawn();
                    std::thread::sleep(std::time::Duration::from_secs(4));
                }

                // First check if there is a native .app bundle in the game directory
                let app_in_dir = find_app_in_dir(Path::new(&game_path_str))
                    .or_else(|| if full_path.ends_with(".app") { Some(PathBuf::from(&full_path)) } else { None });

                if let Some(app_path) = app_in_dir {
                    // Direct launch of .app bundle via `open` works reliably on macOS once Steam is running
                    let _ = Command::new("open").arg(&app_path).spawn();
                } else {
                    // Fallback: resolve Steam App ID and launch via URL scheme
                    let app_id: Option<String> = if full_path.starts_with("steam://run/") {
                        full_path.strip_prefix("steam://run/").map(|s| s.trim_matches('/').to_string())
                    } else if full_path.starts_with("steam://rungameid/") {
                        full_path.strip_prefix("steam://rungameid/").map(|s| s.trim_matches('/').to_string())
                    } else {
                        find_steam_app_id(&exit_game_name, &game_path_str)
                            .or_else(|| find_steam_app_id(&exit_game_name, &full_path))
                    };

                    if let Some(ref id) = app_id {
                        let url = format!("steam://run/{}", id);
                        let _ = Command::new("open").arg(&url).spawn();
                    } else if full_path.ends_with(".app") {
                        let _ = Command::new("open").arg(&full_path).spawn();
                    } else {
                        let _ = Command::new("open").arg("steam://open").spawn();
                    }
                }
            } else if full_path.ends_with(".app") || launcher_lower == "macos" {
                let _ = Command::new("open").arg(&full_path).spawn();
            } else {
                let _ = Command::new("open").arg(&full_path).spawn();
            }
        }

        #[cfg(windows)]
        {
            if is_steam && !full_path.ends_with(".exe") {
                let app_id: Option<String> = if full_path.starts_with("steam://run/") {
                    full_path.strip_prefix("steam://run/").map(|s| s.trim_matches('/').to_string())
                } else if full_path.starts_with("steam://rungameid/") {
                    full_path.strip_prefix("steam://rungameid/").map(|s| s.trim_matches('/').to_string())
                } else {
                    find_steam_app_id(&exit_game_name, &game_path_str)
                        .or_else(|| find_steam_app_id(&exit_game_name, &full_path))
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

        // Give process a brief moment to spin up before polling
        if is_steam {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let _ = exit_handle.emit("game-status", GameStatusPayload {
                status: "waiting".to_string(),
                detail: "Waiting for game to load...".to_string(),
            });
        } else {
            std::thread::sleep(std::time::Duration::from_secs(1));
        }

        // ── Detect the actual game executable ─────────────────────────
        // Strategy: look for new processes that match our startup_targets
        // AND that were NOT running before we launched.
        let max_wait = if is_steam { 120 } else { 60 };
        let startup_poll_start = std::time::Instant::now();
        let mut process_found = false;

        while startup_poll_start.elapsed() < std::time::Duration::from_secs(max_wait) {
            // Method 1: Check if any startup_target is running as an exact process name
            if is_any_target_running(&startup_targets) {
                std::thread::sleep(std::time::Duration::from_millis(1500));
                if is_any_target_running(&startup_targets) {
                    process_found = true;
                    break;
                }
            }

            // Method 2: Check for genuinely NEW processes since launch
            // that match our startup targets in their command line
            let current_pids = get_running_pids();
            let new_pids: Vec<String> = current_pids.difference(&pre_launch_procs).cloned().collect();
            if !new_pids.is_empty() {
                for pid in &new_pids {
                    if let Ok(ps_out) = Command::new("ps")
                        .arg("-p").arg(pid)
                        .arg("-o").arg("comm=")
                        .output()
                    {
                        let comm = String::from_utf8_lossy(&ps_out.stdout).trim().to_string();
                        let comm_lower = comm.to_lowercase();
                        // Skip if it's Steam/system
                        if comm_lower.contains("steam") || comm_lower.contains("helper") || comm.is_empty() {
                            continue;
                        }
                        // Check if this new process matches any startup target
                        for target in &startup_targets {
                            if comm.eq_ignore_ascii_case(target)
                                || comm_lower.contains(&target.to_lowercase())
                            {
                                // Confirm it's stable
                                std::thread::sleep(std::time::Duration::from_millis(1500));
                                if is_any_target_running(&startup_targets) {
                                    process_found = true;
                                    break;
                                }
                            }
                        }
                        if process_found { break; }
                    }
                }
            }
            if process_found { break; }

            std::thread::sleep(std::time::Duration::from_millis(1000));

            if startup_poll_start.elapsed() > std::time::Duration::from_secs(15) {
                let _ = exit_handle.emit("game-status", GameStatusPayload {
                    status: "waiting".to_string(),
                    detail: "Waiting for game to finish loading...".to_string(),
                });
            }
        }

        if process_found {
            let detected_pid = get_target_pid(&startup_targets).or_else(|| get_target_pid(&monitor_targets));

            let _ = exit_handle.emit("game-status", GameStatusPayload {
                status: "ready".to_string(),
                detail: format!("{} is ready!", exit_game_name),
            });

            // Allow sufficient time for emulator graphics API (Metal/OpenGL/Vulkan) & window creation to complete
            let init_delay = if is_emulator { 2800 } else if is_steam { 1200 } else { 800 };
            std::thread::sleep(std::time::Duration::from_millis(init_delay));

            // Activate the game process so its window is in active foreground before Restak drops
            #[cfg(unix)]
            if let Some(target) = startup_targets.first() {
                let _ = Command::new("open").arg("-a").arg(target).spawn();
            } else if !custom_path.is_empty() {
                let _ = Command::new("open").arg("-a").arg(&custom_path).spawn();
            }

            std::thread::sleep(std::time::Duration::from_millis(250));

            // Drop overlay and hide Restak so macOS transitions seamlessly from Restak to the game window
            if let Some(window) = exit_handle.get_webview_window("main") {
                let _ = window.set_always_on_top(false);
                let _ = window.hide();
            }

            let _ = exit_handle.emit("game-started", &exit_game_name);

            // Monitor for exit with 16ms (60 FPS) frame-rate precision using native kernel system calls
            let wait_start = std::time::Instant::now();
            while wait_start.elapsed() < std::time::Duration::from_secs(3600 * 24) {
                if let Some(pid) = detected_pid {
                    // Direct PID tracking: kernel returns instantly when process dies
                    if !is_pid_running(pid) {
                        break;
                    }
                    std::thread::sleep(std::time::Duration::from_millis(16));
                } else {
                    // Fallback target list tracking
                    if !is_any_target_running(&monitor_targets) && !is_any_target_running_broad(&monitor_targets) {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                        if !is_any_target_running(&monitor_targets) && !is_any_target_running_broad(&monitor_targets) {
                            break;
                        }
                    }
                    std::thread::sleep(std::time::Duration::from_millis(100));
                }
            }

            // Game exited — bring Restak back to the active foreground
            if let Some(window) = exit_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_always_on_top(true);
                let _ = window.set_focus();
            }
            let _ = Command::new("open").arg("-a").arg("Restak").spawn();

            // For Steam games: suppress steam_osx and Steam Big Picture Mode popups over 2.5s post-game sync window
            if is_steam {
                let script = "tell application \"System Events\"\nif exists (process \"steam_osx\") then set visible of process \"steam_osx\" to false\nif exists (process \"Steam\") then set visible of process \"Steam\" to false\nend tell";
                let _ = Command::new("osascript").arg("-e").arg(script).spawn();

                let steam_suppress_handle = exit_handle.clone();
                std::thread::spawn(move || {
                    let script = "tell application \"System Events\"\nif exists (process \"steam_osx\") then set visible of process \"steam_osx\" to false\nif exists (process \"Steam\") then set visible of process \"Steam\" to false\nend tell";
                    
                    for delay_ms in [200, 500, 1000, 1800, 2500] {
                        std::thread::sleep(std::time::Duration::from_millis(delay_ms));
                        let _ = Command::new("osascript").arg("-e").arg(script).spawn();
                        if let Some(window) = steam_suppress_handle.get_webview_window("main") {
                            let _ = window.set_focus();
                        }
                    }

                    std::thread::sleep(std::time::Duration::from_millis(300));
                    if let Some(window) = steam_suppress_handle.get_webview_window("main") {
                        let _ = window.set_always_on_top(false);
                    }
                });
            } else {
                let release_handle = exit_handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_millis(500));
                    if let Some(window) = release_handle.get_webview_window("main") {
                        let _ = window.set_always_on_top(false);
                    }
                });
            }

            if was_native_fullscreen {
                if let Some(window) = exit_handle.get_webview_window("main") {
                    let _ = window.set_fullscreen(true);
                }
            }
            let _ = exit_handle.emit("game-exited", &exit_game_name);
        } else {
            // Game never appeared — drop overlay and restore Restak to foreground
            if let Some(window) = exit_handle.get_webview_window("main") {
                let _ = window.set_always_on_top(false);
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
            let _ = Command::new("open").arg("-a").arg("Restak").spawn();

            if was_native_fullscreen {
                if let Some(window) = exit_handle.get_webview_window("main") {
                    let _ = window.set_fullscreen(true);
                }
            }
            let _ = exit_handle.emit("game-exited", &exit_game_name);
        }
    });

    Ok(())
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
            launch_game,
            enter_launch_mode,
            prepare_game_reveal,
            complete_game_reveal,
            toggle_borderless_fullscreen,
            get_fullscreen_state
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
