import sys

with open('src-tauri/src/lib.rs', 'r') as f:
    content = f.read()

# Add find_game_window and new IPC commands before launch_game
new_functions = """
#[cfg(windows)]
fn find_game_window(game_name: &str, launcher_lower: &str) -> bool {
    use windows::Win32::Foundation::{HWND, LPARAM, BOOL};
    use windows::Win32::UI::WindowsAndMessaging::{EnumWindows, IsWindowVisible, GetWindowTextW};
    use std::sync::atomic::{AtomicBool, Ordering};

    static FOUND: AtomicBool = AtomicBool::new(false);
    FOUND.store(false, Ordering::SeqCst);

    unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: LPARAM) -> BOOL {
        if IsWindowVisible(hwnd).as_bool() {
            let mut title: [u16; 512] = [0; 512];
            let len = GetWindowTextW(hwnd, &mut title);
            if len > 0 {
                let title_str = String::from_utf16_lossy(&title[..len as usize]);
                let title_lower = title_str.to_lowercase();
                
                let game_name_ptr = lparam.0 as *const String;
                if !game_name_ptr.is_null() {
                    let game_name = &*game_name_ptr;
                    let gn_lower = game_name.to_lowercase();
                    
                    if !title_lower.contains("steam") && 
                       !title_lower.contains("epic games") &&
                       !title_lower.contains("restak") &&
                       title_lower != "starting game..." {
                        
                        if !gn_lower.is_empty() && (title_lower.contains(&gn_lower) || gn_lower.contains(&title_lower)) {
                            FOUND.store(true, Ordering::SeqCst);
                            return BOOL(0);
                        }
                    }
                }
            }
        }
        BOOL(1)
    }

    let game_name_str = game_name.to_string();
    let lparam = LPARAM(&game_name_str as *const String as isize);
    unsafe {
        let _ = EnumWindows(Some(enum_windows_proc), lparam);
    }

    FOUND.load(Ordering::SeqCst)
}

#[tauri::command]
fn close_overlay(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.close();
    }
}

#[tauri::command]
fn abort_launch(app: tauri::AppHandle) {
    if let Some(window) = app.get_webview_window("overlay") {
        let _ = window.close();
    }
}

#[tauri::command]
async fn launch_game(app: tauri::AppHandle, game: Game) -> Result<(), String> {
"""

content = content.replace("#[tauri::command]\nasync fn launch_game(game: Game) -> Result<(), String> {", new_functions)

# Find the end of variable declarations inside launch_game to insert the overlay creation
injection_point = "    let game_name = game.name.clone();\n"
overlay_code = """
    if let Some(existing) = app.get_webview_window("overlay") {
        let _ = existing.close();
    }

    let overlay = tauri::WebviewWindowBuilder::new(
        &app,
        "overlay",
        tauri::WebviewUrl::App("overlay.html".into()),
    )
    .title("Restak Overlay")
    .decorations(false)
    .fullscreen(true)
    .always_on_top(true)
    .transparent(false)
    .build()
    .map_err(|e| e.to_string())?;

    #[cfg(windows)]
    {
        if let Ok(hwnd_ptr) = overlay.hwnd() {
            unsafe {
                use windows::Win32::Foundation::HWND;
                use windows::Win32::UI::WindowsAndMessaging::{SetWindowPos, HWND_TOPMOST, SWP_NOMOVE, SWP_NOSIZE};
                let hwnd = HWND(hwnd_ptr.0 as _);
                let _ = SetWindowPos(hwnd, HWND_TOPMOST, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE);
            }
        }
    }
"""
content = content.replace(injection_point, injection_point + overlay_code)

# Find the end of launch_game to inject the background polling thread
end_injection = """
    #[cfg(windows)]
    {
        let _ = Command::new("cmd").arg("/c").arg("start").arg("").arg(&full_path).spawn();
        }
    }

    Ok(())
}
"""

polling_code = """
    #[cfg(windows)]
    {
        let _ = Command::new("cmd").arg("/c").arg("start").arg("").arg(&full_path).spawn();
        }
    }

    let overlay_clone = overlay.clone();
    let game_name_clone = game_name.clone();
    let launcher_lower_clone = launcher_lower.clone();
    
    std::thread::spawn(move || {
        #[cfg(windows)]
        {
            loop {
                std::thread::sleep(std::time::Duration::from_millis(16));
                let is_open = overlay_clone.app_handle().get_webview_window("overlay").is_some();
                if !is_open { break; }

                if find_game_window(&game_name_clone, &launcher_lower_clone) {
                    let _ = overlay_clone.emit("overlay-fade-out", ());
                    break;
                }
            }
        }
        #[cfg(unix)]
        {
            std::thread::sleep(std::time::Duration::from_secs(3));
            let is_open = overlay_clone.app_handle().get_webview_window("overlay").is_some();
            if is_open {
                let _ = overlay_clone.emit("overlay-fade-out", ());
            }
        }
    });

    Ok(())
}
"""
content = content.replace(end_injection, polling_code)

# Register the new commands in the generate_handler
handler_old = """        .invoke_handler(tauri::generate_handler![
            scan_for_games,
            load_library,
            save_library,
            pick_launcher,
            launch_game,
            toggle_borderless_fullscreen,
            get_fullscreen_state
        ])"""
handler_new = """        .invoke_handler(tauri::generate_handler![
            scan_for_games,
            load_library,
            save_library,
            pick_launcher,
            launch_game,
            toggle_borderless_fullscreen,
            get_fullscreen_state,
            close_overlay,
            abort_launch
        ])"""
content = content.replace(handler_old, handler_new)

with open('src-tauri/src/lib.rs', 'w') as f:
    f.write(content)

