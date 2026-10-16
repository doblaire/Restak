use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};
use walkdir::WalkDir;

use crate::steamgriddb;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Game {
    pub name: String,
    pub path: String,
    #[serde(rename = "fullPath", default)]
    pub full_path: Option<String>,
    pub launcher: String,
    #[serde(rename = "coverArt", default)]
    pub cover_art: Option<String>,
    #[serde(default)]
    pub genre: Option<String>,
    #[serde(default)]
    pub publisher: Option<String>,
    #[serde(default)]
    pub year: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "customLauncherPath", default)]
    pub custom_launcher_path: Option<String>,
}

const KNOWN_GAME_APPS: &[&str] = &[
    "Minecraft",
    "Roblox",
    "League of Legends",
    "World of Warcraft",
    "Diablo IV",
    "Hearthstone",
    "Overwatch",
    "Fortnite",
    "Valorant",
    "Genshin Impact",
    "Honkai: Star Rail",
    "Civilization VI",
    "Stardew Valley",
    "Terraria",
    "Hollow Knight",
    "Baldur's Gate 3",
];

const EMULATOR_NAMES: &[&str] = &[
    "Dolphin",
    "RetroArch",
    "OpenEmu",
    "mGBA",
    "PCSX2",
    "Ryujinx",
    "Yuzu",
    "Citra",
    "DeSmuME",
    "PPSSPP",
    "DuckStation",
    "RPCS3",
    "Flycast",
    "Cemu",
    "Steam ROM Manager",
    "GCAdapterDriver",
];

pub fn user_home_dir() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
}

pub fn is_app_or_exe_installed(app_name: &str) -> bool {
    #[cfg(unix)]
    {
        let app_file = format!("{}.app", app_name);
        if Path::new("/Applications").join(&app_file).exists() {
            return true;
        }
        if let Some(home) = user_home_dir() {
            if home.join("Applications").join(&app_file).exists() {
                return true;
            }
        }
        false
    }

    #[cfg(windows)]
    {
        let exe_file = format!("{}.exe", app_name.to_lowercase());
        let candidate_dirs = [
            std::env::var("ProgramFiles").ok().map(PathBuf::from),
            std::env::var("ProgramFiles(x86)").ok().map(PathBuf::from),
            std::env::var("LOCALAPPDATA").ok().map(|p| PathBuf::from(p).join("Programs")),
            std::env::var("APPDATA").ok().map(PathBuf::from),
            Some(PathBuf::from("C:\\Program Files")),
            Some(PathBuf::from("C:\\Program Files (x86)")),
            Some(PathBuf::from("C:\\Games")),
            Some(PathBuf::from("D:\\Games")),
        ];

        for dir_opt in candidate_dirs.into_iter().flatten() {
            if dir_opt.exists() {
                if let Ok(entries) = fs::read_dir(&dir_opt) {
                    for entry in entries.flatten() {
                        let p = entry.path();
                        if p.is_file() && p.file_name().and_then(|n| n.to_str()).map(|n| n.to_lowercase()) == Some(exe_file.clone()) {
                            return true;
                        }
                        if p.is_dir() && p.join(&exe_file).exists() {
                            return true;
                        }
                    }
                }
            }
        }
        false
    }
}

fn normalize_title(name: &str) -> String {
    name.replace('-', " ")
        .chars()
        .filter(|c| c.is_alphanumeric() || c.is_whitespace())
        .collect::<String>()
        .to_lowercase()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

pub fn lookup_game_metadata(title: &str) -> (Option<&'static str>, Option<&'static str>, Option<&'static str>) {
    let norm = normalize_title(title);
    
    if norm == "minecraft" {
        return (Some("Mojang Studios"), Some("2011"), Some("Sandbox"));
    }
    if norm == "roblox" {
        return (Some("Roblox Corporation"), Some("2006"), Some("Sandbox"));
    }
    if norm.contains("super mario sunshine") {
        return (Some("Nintendo"), Some("2002"), Some("Platformer"));
    }
    if norm.contains("super smash bros melee") || norm == "melee" {
        return (Some("Nintendo"), Some("2001"), Some("Fighting"));
    }
    if norm.contains("super smash bros brawl") || norm == "brawl" {
        return (Some("Nintendo"), Some("2008"), Some("Fighting"));
    }
    if norm.contains("super smash bros for nintendo 3ds") || norm.contains("smash 3ds") {
        return (Some("Nintendo"), Some("2014"), Some("Fighting"));
    }
    if norm.contains("chibi robo") {
        return (Some("Nintendo"), Some("2005"), Some("Action-Adventure"));
    }
    if norm.contains("bloons td 6") || norm == "btd6" {
        return (Some("Ninja Kiwi"), Some("2018"), Some("Tower Defense"));
    }
    if norm.contains("bloons td battles") || norm == "btdb" {
        return (Some("Ninja Kiwi"), Some("2016"), Some("Tower Defense"));
    }
    if norm.contains("mario kart double dash") {
        return (Some("Nintendo"), Some("2003"), Some("Racing"));
    }
    if norm.contains("the legend of zelda the wind waker") || norm.contains("wind waker") {
        return (Some("Nintendo"), Some("2002"), Some("Action-Adventure"));
    }
    if norm.contains("the legend of zelda twilight princess") || norm.contains("twilight princess") {
        return (Some("Nintendo"), Some("2006"), Some("Action-Adventure"));
    }
    if norm.contains("metroid prime") {
        return (Some("Nintendo"), Some("2002"), Some("Action-Adventure"));
    }
    if norm.contains("luigis mansion") {
        return (Some("Nintendo"), Some("2001"), Some("Action-Adventure"));
    }
    if norm.contains("super mario 64") {
        return (Some("Nintendo"), Some("1996"), Some("Platformer"));
    }
    if norm.contains("ocarina of time") {
        return (Some("Nintendo"), Some("1998"), Some("Action-Adventure"));
    }
    if norm.contains("majoras mask") {
        return (Some("Nintendo"), Some("2000"), Some("Action-Adventure"));
    }
    if norm.contains("mario kart 64") {
        return (Some("Nintendo"), Some("1996"), Some("Racing"));
    }
    if norm.contains("pokemon emerald") {
        return (Some("The Pokémon Company"), Some("2004"), Some("RPG"));
    }
    if norm.contains("pokemon fire red") || norm.contains("pokemon firered") {
        return (Some("The Pokémon Company"), Some("2004"), Some("RPG"));
    }
    if norm.contains("pokemon leaf green") || norm.contains("pokemon leafgreen") {
        return (Some("The Pokémon Company"), Some("2004"), Some("RPG"));
    }
    if norm.contains("pokemon heart gold") || norm.contains("pokemon heartgold") {
        return (Some("The Pokémon Company"), Some("2009"), Some("RPG"));
    }
    if norm.contains("pokemon soul silver") || norm.contains("pokemon soulsilver") {
        return (Some("The Pokémon Company"), Some("2009"), Some("RPG"));
    }
    if norm.contains("pokemon platinum") {
        return (Some("The Pokémon Company"), Some("2008"), Some("RPG"));
    }
    if norm.contains("pokemon diamond") || norm.contains("pokemon pearl") {
        return (Some("The Pokémon Company"), Some("2006"), Some("RPG"));
    }
    if norm.contains("super mario world") {
        return (Some("Nintendo"), Some("1990"), Some("Platformer"));
    }
    if norm.contains("super metroid") {
        return (Some("Nintendo"), Some("1994"), Some("Action-Adventure"));
    }
    if norm.contains("chrono trigger") {
        return (Some("Square"), Some("1995"), Some("RPG"));
    }
    if norm.contains("baldurs gate 3") {
        return (Some("Larian Studios"), Some("2023"), Some("RPG"));
    }
    if norm.contains("hollow knight") {
        return (Some("Team Cherry"), Some("2017"), Some("Metroidvania"));
    }
    if norm.contains("stardew valley") {
        return (Some("ConcernedApe"), Some("2016"), Some("Simulation"));
    }
    if norm.contains("terraria") {
        return (Some("Re-Logic"), Some("2011"), Some("Sandbox"));
    }
    if norm.contains("civilization vi") {
        return (Some("2K Games"), Some("2016"), Some("Strategy"));
    }
    if norm.contains("league of legends") {
        return (Some("Riot Games"), Some("2009"), Some("MOBA"));
    }
    if norm.contains("world of warcraft") {
        return (Some("Blizzard Entertainment"), Some("2004"), Some("MMORPG"));
    }
    if norm.contains("diablo iv") {
        return (Some("Blizzard Entertainment"), Some("2023"), Some("Action RPG"));
    }
    if norm.contains("hearthstone") {
        return (Some("Blizzard Entertainment"), Some("2014"), Some("Card Game"));
    }
    if norm.contains("overwatch") {
        return (Some("Blizzard Entertainment"), Some("2016"), Some("Shooter"));
    }
    if norm.contains("fortnite") {
        return (Some("Epic Games"), Some("2017"), Some("Battle Royale"));
    }
    if norm.contains("valorant") {
        return (Some("Riot Games"), Some("2020"), Some("Shooter"));
    }
    if norm.contains("genshin impact") {
        return (Some("HoYoverse"), Some("2020"), Some("Action RPG"));
    }
    if norm.contains("honkai star rail") {
        return (Some("HoYoverse"), Some("2023"), Some("RPG"));
    }
    if norm.contains("portal 2") {
        return (Some("Valve"), Some("2011"), Some("Puzzle"));
    }
    if norm.contains("portal") {
        return (Some("Valve"), Some("2007"), Some("Puzzle"));
    }
    if norm.contains("half life 2") {
        return (Some("Valve"), Some("2004"), Some("Shooter"));
    }
    if norm.contains("counter strike 2") {
        return (Some("Valve"), Some("2023"), Some("Shooter"));
    }
    if norm.contains("left 4 dead 2") {
        return (Some("Valve"), Some("2009"), Some("Shooter"));
    }
    if norm.contains("team fortress 2") {
        return (Some("Valve"), Some("2007"), Some("Shooter"));
    }
    if norm.contains("dota 2") {
        return (Some("Valve"), Some("2013"), Some("MOBA"));
    }
    if norm.contains("elden ring") {
        return (Some("Bandai Namco"), Some("2022"), Some("Action RPG"));
    }
    if norm.contains("cyberpunk 2077") {
        return (Some("CD Projekt Red"), Some("2020"), Some("Action RPG"));
    }
    if norm.contains("the witcher 3") {
        return (Some("CD Projekt Red"), Some("2015"), Some("RPG"));
    }

    (None, None, None)
}

pub fn get_steam_library_folders() -> Vec<PathBuf> {
    let mut folders = Vec::new();
    let mut root_steam_dirs = Vec::new();

    #[cfg(unix)]
    {
        if let Some(home) = user_home_dir() {
            root_steam_dirs.push(home.join("Library/Application Support/Steam"));
            root_steam_dirs.push(home.join(".steam/steam"));
            root_steam_dirs.push(home.join(".local/share/Steam"));
        }
    }

    #[cfg(windows)]
    {
        if let Ok(pf86) = std::env::var("ProgramFiles(x86)") {
            root_steam_dirs.push(PathBuf::from(pf86).join("Steam"));
        }
        if let Ok(pf) = std::env::var("ProgramFiles") {
            root_steam_dirs.push(PathBuf::from(pf).join("Steam"));
        }
        root_steam_dirs.push(PathBuf::from("C:\\Program Files (x86)\\Steam"));
        root_steam_dirs.push(PathBuf::from("C:\\Program Files\\Steam"));
        root_steam_dirs.push(PathBuf::from("C:\\Steam"));
        root_steam_dirs.push(PathBuf::from("D:\\Steam"));
        root_steam_dirs.push(PathBuf::from("E:\\Steam"));
    }

    for root in root_steam_dirs {
        let steamapps = root.join("steamapps");
        if steamapps.exists() && !folders.contains(&steamapps) {
            folders.push(steamapps.clone());
        }

        // Parse libraryfolders.vdf for secondary libraries across drives
        let lib_vdf = root.join("steamapps/libraryfolders.vdf");
        if lib_vdf.exists() {
            if let Ok(content) = fs::read_to_string(&lib_vdf) {
                for line in content.lines() {
                    let trimmed = line.trim();
                    if trimmed.starts_with("\"path\"") {
                        let parts: Vec<&str> = trimmed.split('"').collect();
                        if parts.len() >= 4 {
                            let custom_root = PathBuf::from(parts[3].replace("\\\\", "\\"));
                            let custom_apps = custom_root.join("steamapps");
                            if custom_apps.exists() && !folders.contains(&custom_apps) {
                                folders.push(custom_apps);
                            }
                        }
                    }
                }
            }
        }
    }

    folders
}

pub fn scan_applications() -> Vec<Game> {
    let mut results = Vec::new();

    #[cfg(unix)]
    {
        let apps_dir = Path::new("/Applications");
        if let Ok(entries) = fs::read_dir(apps_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                    if file_name.ends_with(".app") {
                        let app_name = &file_name[..file_name.len() - 4];
                        let is_game = KNOWN_GAME_APPS.contains(&app_name) || {
                            let plist_path = path.join("Contents/Info.plist");
                            if let Ok(bytes) = fs::read(plist_path) {
                                let needle = b"app-category.game";
                                bytes.windows(needle.len()).any(|w| w == needle)
                            } else {
                                false
                            }
                        };

                        if is_game && !EMULATOR_NAMES.contains(&app_name) {
                            let full_str = path.to_string_lossy().to_string();
                            let (pub_opt, yr_opt, gen_opt) = lookup_game_metadata(app_name);
                            results.push(Game {
                                name: app_name.to_string(),
                                path: full_str.clone(),
                                full_path: Some(full_str),
                                launcher: "macOS".to_string(),
                                cover_art: None,
                                genre: gen_opt.map(|s| s.to_string()),
                                publisher: pub_opt.map(|s| s.to_string()),
                                year: yr_opt.map(|s| s.to_string()),
                                description: None,
                                custom_launcher_path: None,
                            });
                        }
                    }
                }
            }
        }
    }

    #[cfg(windows)]
    {
        let candidate_dirs = [
            std::env::var("ProgramFiles").ok().map(PathBuf::from),
            std::env::var("ProgramFiles(x86)").ok().map(PathBuf::from),
            std::env::var("LOCALAPPDATA").ok().map(|p| PathBuf::from(p).join("Programs")),
            Some(PathBuf::from("C:\\XboxGames")),
            Some(PathBuf::from("C:\\Games")),
            Some(PathBuf::from("D:\\Games")),
            Some(PathBuf::from("E:\\Games")),
        ];

        for base_dir in candidate_dirs.into_iter().flatten() {
            if base_dir.exists() {
                if let Ok(entries) = fs::read_dir(&base_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.is_dir() {
                            let folder_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
                            if KNOWN_GAME_APPS.iter().any(|&k| folder_name.eq_ignore_ascii_case(k)) {
                                let full_str = path.to_string_lossy().to_string();
                                let (pub_opt, yr_opt, gen_opt) = lookup_game_metadata(folder_name);
                                results.push(Game {
                                    name: folder_name.to_string(),
                                    path: full_str.clone(),
                                    full_path: Some(full_str),
                                    launcher: "Windows".to_string(),
                                    cover_art: None,
                                    genre: gen_opt.map(|s| s.to_string()),
                                    publisher: pub_opt.map(|s| s.to_string()),
                                    year: yr_opt.map(|s| s.to_string()),
                                    description: None,
                                    custom_launcher_path: None,
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    results
}

pub fn scan_steam() -> Vec<Game> {
    let mut results = Vec::new();
    let library_folders = get_steam_library_folders();

    for steam_apps_dir in library_folders {
        if let Ok(entries) = fs::read_dir(&steam_apps_dir) {
            for entry in entries.flatten() {
                let file_name = entry.file_name().to_string_lossy().to_string();
                if file_name.starts_with("appmanifest_") && file_name.ends_with(".acf") {
                    if let Ok(content) = fs::read_to_string(entry.path()) {
                        let mut name = None;
                        let mut app_id = None;
                        let mut install_dir = None;

                        for line in content.lines() {
                            let trimmed = line.trim();
                            if trimmed.starts_with("\"name\"") {
                                let parts: Vec<&str> = trimmed.split('"').collect();
                                if parts.len() >= 4 {
                                    name = Some(parts[3].to_string());
                                }
                            } else if trimmed.starts_with("\"appid\"") {
                                let parts: Vec<&str> = trimmed.split('"').collect();
                                if parts.len() >= 4 {
                                    app_id = Some(parts[3].to_string());
                                }
                            } else if trimmed.starts_with("\"installdir\"") {
                                let parts: Vec<&str> = trimmed.split('"').collect();
                                if parts.len() >= 4 {
                                    install_dir = Some(parts[3].to_string());
                                }
                            }
                        }

                        if let Some(game_name) = name {
                            let mut game_path = steam_apps_dir.to_string_lossy().to_string();
                            if let Some(dir) = install_dir {
                                let common = steam_apps_dir.join("common").join(dir);
                                if common.exists() {
                                    game_path = common.to_string_lossy().to_string();
                                }
                            }

                            let full_launch = app_id.map(|id| format!("steam://run/{}", id));
                            let (pub_opt, yr_opt, gen_opt) = lookup_game_metadata(&game_name);

                            results.push(Game {
                                name: game_name,
                                path: game_path,
                                full_path: full_launch,
                                launcher: "Steam".to_string(),
                                cover_art: None,
                                genre: gen_opt.map(|s| s.to_string()),
                                publisher: pub_opt.map(|s| s.to_string()),
                                year: yr_opt.map(|s| s.to_string()),
                                description: None,
                                custom_launcher_path: None,
                            });
                        }
                    }
                }
            }
        }
    }
    results
}

fn clean_rom_name(file_name: &str) -> String {
    let mut base = file_name;
    if let Some(idx) = base.rfind('.') {
        base = &base[..idx];
    }
    if base.ends_with(".nkit") {
        base = &base[..base.len() - 5];
    }

    let mut cleaned = String::new();
    let mut depth_paren = 0;
    let mut depth_bracket = 0;

    for c in base.chars() {
        match c {
            '(' => depth_paren += 1,
            ')' => if depth_paren > 0 { depth_paren -= 1 },
            '[' => depth_bracket += 1,
            ']' => if depth_bracket > 0 { depth_bracket -= 1 },
            _ => {
                if depth_paren == 0 && depth_bracket == 0 {
                    cleaned.push(c);
                }
            }
        }
    }

    let trimmed = cleaned.trim();
    if trimmed.is_empty() {
        file_name.to_string()
    } else {
        trimmed.to_string()
    }
}

pub fn resolve_emulator_for_ext(ext: &str) -> String {
    let ext_lower = ext.to_lowercase();
    match ext_lower.as_str() {
        "iso" | "gcm" | "rvz" | "wbfs" | "ciso" => {
            if is_app_or_exe_installed("Dolphin") {
                "Dolphin Emulator".to_string()
            } else if is_app_or_exe_installed("RetroArch") {
                "RetroArch".to_string()
            } else {
                "Dolphin Emulator".to_string()
            }
        }
        "gba" | "gbc" | "gb" => {
            if is_app_or_exe_installed("mGBA") {
                "mGBA".to_string()
            } else if is_app_or_exe_installed("RetroArch") {
                "RetroArch".to_string()
            } else if is_app_or_exe_installed("OpenEmu") {
                "OpenEmu".to_string()
            } else {
                "mGBA".to_string()
            }
        }
        "nds" => {
            if is_app_or_exe_installed("DeSmuME") {
                "DeSmuME".to_string()
            } else if is_app_or_exe_installed("RetroArch") {
                "RetroArch".to_string()
            } else {
                "DeSmuME".to_string()
            }
        }
        "3ds" => {
            if is_app_or_exe_installed("Citra") {
                "Citra".to_string()
            } else {
                "Citra".to_string()
            }
        }
        "n64" | "z64" | "v64" => {
            if is_app_or_exe_installed("mupen64plus") {
                "mupen64plus".to_string()
            } else if is_app_or_exe_installed("RetroArch") {
                "RetroArch".to_string()
            } else {
                "RetroArch".to_string()
            }
        }
        "nes" | "snes" | "smc" | "sfc" => {
            if is_app_or_exe_installed("Snes9x") {
                "Snes9x".to_string()
            } else if is_app_or_exe_installed("RetroArch") {
                "RetroArch".to_string()
            } else {
                "RetroArch".to_string()
            }
        }
        "nsp" | "xci" => {
            if is_app_or_exe_installed("Ryujinx") {
                "Ryujinx".to_string()
            } else if is_app_or_exe_installed("Yuzu") {
                "Yuzu".to_string()
            } else {
                "Ryujinx".to_string()
            }
        }
        _ => "RetroArch".to_string(),
    }
}

const SUPPORTED_ROM_EXTS: &[&str] = &[
    "iso", "gcm", "rvz", "wbfs", "ciso",
    "gba", "gbc", "gb",
    "nds", "3ds",
    "n64", "z64", "v64",
    "nes", "snes", "smc", "sfc",
    "nsp", "xci",
    "chd", "cue", "bin",
];

pub fn scan_roms() -> Vec<Game> {
    let mut results = Vec::new();
    let mut candidate_dirs = Vec::new();

    if let Some(home) = user_home_dir() {
        candidate_dirs.push(home.join("Documents/ROMS"));
        candidate_dirs.push(home.join("Documents/ROMs"));
        candidate_dirs.push(home.join("Documents/Roms"));
        candidate_dirs.push(home.join("ROMS"));
        candidate_dirs.push(home.join("ROMs"));
        candidate_dirs.push(home.join("Roms"));
        candidate_dirs.push(home.join("Downloads"));
        candidate_dirs.push(home.join("Downloads/ROMS"));
        candidate_dirs.push(home.join("Downloads/ROMs"));
        candidate_dirs.push(home.join("Desktop/ROMS"));
        candidate_dirs.push(home.join("Desktop/ROMs"));
    }

    #[cfg(windows)]
    {
        for drive_letter in b'C'..=b'Z' {
            let drive_root = format!("{}:\\", drive_letter as char);
            let path = PathBuf::from(&drive_root);
            if path.exists() {
                candidate_dirs.push(path.join("ROMS"));
                candidate_dirs.push(path.join("ROMs"));
                candidate_dirs.push(path.join("Roms"));
                candidate_dirs.push(path.join("Emulation/ROMs"));
                candidate_dirs.push(path.join("Games/ROMS"));
            }
        }
    }

    let mut canonical_dirs = HashSet::new();
    for dir in candidate_dirs {
        if dir.exists() {
            if let Ok(canon) = fs::canonicalize(&dir) {
                canonical_dirs.insert(canon);
            }
        }
    }

    for dir in canonical_dirs {
        for entry in WalkDir::new(&dir).max_depth(4).into_iter().flatten() {
            if entry.file_type().is_file() {
                let path = entry.path();
                if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if SUPPORTED_ROM_EXTS.contains(&ext_lower.as_str()) {
                        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
                        let parent_path = path.parent().map(|p| p.to_string_lossy().to_string()).unwrap_or_default();
                        let cleaned = clean_rom_name(file_name);
                        let (pub_opt, yr_opt, gen_opt) = lookup_game_metadata(&cleaned);

                        results.push(Game {
                            name: cleaned,
                            path: parent_path,
                            full_path: Some(path.to_string_lossy().to_string()),
                            launcher: resolve_emulator_for_ext(&ext_lower),
                            cover_art: None,
                            genre: gen_opt.map(|s| s.to_string()),
                            publisher: pub_opt.map(|s| s.to_string()),
                            year: yr_opt.map(|s| s.to_string()),
                            description: None,
                            custom_launcher_path: None,
                        });
                    }
                }
            }
        }
    }

    results
}

pub async fn scan_for_games() -> Vec<Game> {
    let apps = scan_applications();
    let steam = scan_steam();
    let roms = scan_roms();

    let mut combined = Vec::new();
    combined.extend(apps);
    combined.extend(steam);
    combined.extend(roms);

    let mut seen = HashSet::new();
    let mut deduplicated = Vec::new();

    for game in combined {
        let key = format!("{}:::{}", game.name, game.launcher);
        if seen.insert(key) {
            deduplicated.push(game);
        }
    }

    // Fetch cover art using SteamGridDB via Cloudflare Worker proxy
    let client = reqwest::Client::new();
    let mut tasks = Vec::new();

    for mut game in deduplicated {
        let client_clone = client.clone();
        tasks.push(async move {
            if game.cover_art.is_none() {
                game.cover_art = steamgriddb::get_cover_art(&client_clone, &game.name).await;
            }
            game
        });
    }

    futures::future::join_all(tasks).await
}
