import sys

with open('src-tauri/src/lib.rs', 'r') as f:
    content = f.read()

end_injection = """        } else {
            let _ = Command::new("cmd").arg("/c").arg("start").arg("").arg(&full_path).spawn();
        }
    }

    Ok(())
}"""

polling_code = """        } else {
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
}"""

content = content.replace(end_injection, polling_code)

with open('src-tauri/src/lib.rs', 'w') as f:
    f.write(content)

