use std::path::Path;
use std::process::Command;

#[cfg(unix)]
pub fn is_steam_running() -> bool {
    Command::new("pgrep")
        .arg("-xi")
        .arg("steam_osx")
        .output()
        .map_or(false, |o| o.status.success())
}

#[cfg(unix)]
pub fn hide_steam_window() {
    let script = r#"
        tell application "System Events"
            try
                set visible of (every process whose bundle identifier is "com.valvesoftware.steam" or bundle identifier is "com.valvesoftware.steam.helper" or name is "steam_osx") to false
            end try
        end tell
    "#;
    let _ = Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output();
}

#[cfg(unix)]
pub fn get_app_bundle_executables(app_path: &Path) -> Vec<String> {
    let mut names = Vec::new();
    let macos_dir = app_path.join("Contents/MacOS");
    if macos_dir.is_dir() {
        if let Ok(entries) = std::fs::read_dir(macos_dir) {
            for entry in entries.flatten() {
                if let Some(s) = entry.file_name().to_str() {
                    let s_str = s.to_string();
                    if !names.contains(&s_str) {
                        names.push(s_str);
                    }
                }
            }
        }
    }
    if let Some(stem) = app_path.file_stem().and_then(|s| s.to_str()) {
        let stem_str = stem.to_string();
        if !names.contains(&stem_str) {
            names.push(stem_str);
        }
    }
    names
}

#[cfg(unix)]
pub fn get_target_pid(target_names: &[String]) -> Option<i32> {
    let ignored = [
        "steam", "steam_osx", "steamwebhelper", "restak", "finder", "dock", 
        "launchd", "systemevents", "loginwindow", "windowserver", "mds"
    ];

    let my_pid = std::process::id() as i32;

    for name in target_names {
        let name_clean = name.trim();
        if name_clean.is_empty() {
            continue;
        }

        let name_lower = name_clean.to_lowercase();
        if ignored.iter().any(|&ig| name_lower == ig) {
            continue;
        }

        // 1. Exact match
        if let Ok(output) = Command::new("pgrep").arg("-xi").arg(name_clean).output() {
            if let Ok(s) = String::from_utf8(output.stdout) {
                for line in s.lines() {
                    if let Ok(pid) = line.trim().parse::<i32>() {
                        if pid != my_pid && is_pid_running(pid) {
                            return Some(pid);
                        }
                    }
                }
            }
        }

        // 2. Full command-line substring match if name is reasonably specific (>= 4 chars)
        if name_clean.len() >= 4 {
            if let Ok(output) = Command::new("pgrep").arg("-fi").arg(name_clean).output() {
                if let Ok(s) = String::from_utf8(output.stdout) {
                    for line in s.lines() {
                        if let Ok(pid) = line.trim().parse::<i32>() {
                            if pid != my_pid && is_pid_running(pid) {
                                return Some(pid);
                            }
                        }
                    }
                }
            }
        }
    }
    None
}

#[cfg(unix)]
pub fn is_pid_running(pid: i32) -> bool {
    unsafe { libc::kill(pid, 0) == 0 }
}
