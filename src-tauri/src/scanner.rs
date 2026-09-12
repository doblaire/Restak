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
    pub tags: Option<String>,
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

fn is_app_installed(app_name: &str) -> bool {
    let app_file = format!("{}.app", app_name);
    if Path::new("/Applications").join(&app_file).exists() {
        return true;
    }
    if let Some(home) = std::env::var_os("HOME").map(PathBuf::from) {
        if home.join("Applications").join(&app_file).exists() {
            return true;
        }
    }
    false
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

fn is_game_app(app_name: &str, full_path: &Path) -> bool {
    if EMULATOR_NAMES.contains(&app_name) {
        return false;
    }
    if KNOWN_GAME_APPS.contains(&app_name) {
        return true;
    }
    let plist_path = full_path.join("Contents/Info.plist");
    if let Ok(bytes) = fs::read(plist_path) {
        let needle = b"app-category.game";
        if bytes.windows(needle.len()).any(|window| window == needle) {
            return true;
        }
    }
    false
}

pub fn scan_applications() -> Vec<Game> {
    let mut results = Vec::new();
    let apps_dir = Path::new("/Applications");
    if let Ok(entries) = fs::read_dir(apps_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                if file_name.ends_with(".app") {
                    let app_name = &file_name[..file_name.len() - 4];
                    if is_game_app(app_name, &path) {
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
                            tags: None,
                            custom_launcher_path: None,
                        });
                    }
                }
            }
        }
    }
    results
}

pub fn scan_steam() -> Vec<Game> {
    let mut results = Vec::new();
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return results;
    };
    let steam_apps_dir = home.join("Library/Application Support/Steam/steamapps");
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

                        let launcher_name = if is_app_installed("Steam") {
                            "Steam".to_string()
                        } else {
                            "No Launcher Found".to_string()
                        };

                        results.push(Game {
                            name: game_name,
                            path: game_path,
                            full_path: full_launch,
                            launcher: launcher_name,
                            cover_art: None,
                            genre: gen_opt.map(|s| s.to_string()),
                            publisher: pub_opt.map(|s| s.to_string()),
                            year: yr_opt.map(|s| s.to_string()),
                            tags: None,
                            custom_launcher_path: None,
                        });
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
            if is_app_installed("Dolphin") {
                "Dolphin Emulator".to_string()
            } else if is_app_installed("RetroArch") {
                "RetroArch".to_string()
            } else {
                "No Launcher Found".to_string()
            }
        }
        "gba" | "gbc" | "gb" => {
            if is_app_installed("mGBA") {
                "mGBA".to_string()
            } else if is_app_installed("RetroArch") {
                "RetroArch".to_string()
            } else if is_app_installed("OpenEmu") {
                "OpenEmu".to_string()
            } else {
                "No Launcher Found".to_string()
            }
        }
        "nds" => {
            if is_app_installed("DeSmuME") {
                "DeSmuME".to_string()
            } else if is_app_installed("RetroArch") {
                "RetroArch".to_string()
            } else if is_app_installed("OpenEmu") {
                "OpenEmu".to_string()
            } else {
                "No Launcher Found".to_string()
            }
        }
        "3ds" => {
            if is_app_installed("Citra") {
                "Citra".to_string()
            } else {
                "No Launcher Found".to_string()
            }
        }
        "n64" | "z64" | "v64" => {
            if is_app_installed("mupen64plus") {
                "mupen64plus".to_string()
            } else if is_app_installed("OpenEmu") {
                "OpenEmu".to_string()
            } else if is_app_installed("RetroArch") {
                "RetroArch".to_string()
            } else {
                "No Launcher Found".to_string()
            }
        }
        "nes" | "snes" | "smc" | "sfc" => {
            if is_app_installed("Snes9x") {
                "Snes9x".to_string()
            } else if is_app_installed("OpenEmu") {
                "OpenEmu".to_string()
            } else if is_app_installed("RetroArch") {
                "RetroArch".to_string()
            } else {
                "No Launcher Found".to_string()
            }
        }
        "nsp" | "xci" => {
            if is_app_installed("Ryujinx") {
                "Ryujinx".to_string()
            } else if is_app_installed("Yuzu") {
                "Yuzu".to_string()
            } else {
                "No Launcher Found".to_string()
            }
        }
        _ => "No Launcher Found".to_string(),
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
    let Some(home) = std::env::var_os("HOME").map(PathBuf::from) else {
        return results;
    };

    let candidate_dirs = [
        home.join("Documents/ROMS"),
        home.join("Documents/ROMs"),
        home.join("ROMS"),
        home.join("ROMs"),
        home.join("Downloads"),
        home.join("Desktop/ROMS"),
        home.join("Desktop/ROMs"),
    ];

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
                            tags: None,
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

    // Fetch SteamGridDB cover art concurrently
    let client = reqwest::Client::new();
    let mut tasks = Vec::new();

    for mut game in deduplicated {
        let client_clone = client.clone();
        tasks.push(async move {
            game.cover_art = steamgriddb::get_cover_art(&client_clone, &game.name).await;
            game
        });
    }

    futures_util_join_all(tasks).await
}

async fn futures_util_join_all<F>(futures: Vec<F>) -> Vec<F::Output>
where
    F: std::future::Future,
{
    let mut results = Vec::with_capacity(futures.len());
    for f in futures {
        results.push(f.await);
    }
    results
}
