use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use serde::Deserialize;

const PROXY_ENDPOINT: &str = "https://restak-proxy.victrlake.workers.dev";

static CACHE: Mutex<Option<HashMap<String, Vec<String>>>> = Mutex::new(None);

fn cache_file_path() -> Option<PathBuf> {
    dirs_home().map(|h| h.join(".restak_cover_cache.json"))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .or_else(|| std::env::var_os("APPDATA"))
        .map(PathBuf::from)
}

fn load_cache() -> HashMap<String, Vec<String>> {
    if let Some(path) = cache_file_path() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(map) = serde_json::from_str(&data) {
                return map;
            }
        }
    }
    HashMap::new()
}

fn save_cache(cache: &HashMap<String, Vec<String>>) {
    if let Some(path) = cache_file_path() {
        if let Ok(data) = serde_json::to_string_pretty(cache) {
            let _ = fs::write(path, data);
        }
    }
}

fn clean_query(name: &str) -> String {
    let mut cleaned = String::new();
    let mut depth_paren = 0;
    let mut depth_bracket = 0;

    for c in name.chars() {
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

    cleaned
        .replace("Rev ", " ")
        .replace(".nkit", "")
        .replace(".iso", "")
        .replace(".gba", "")
        .replace(".exe", "")
        .trim()
        .to_string()
}

#[derive(Deserialize)]
struct ProxyResponse {
    #[serde(default)]
    success: bool,
    #[serde(default)]
    covers: Vec<String>,
}

pub async fn get_covers(client: &reqwest::Client, game_name: &str) -> Vec<String> {
    let key = game_name.trim().to_lowercase();
    if key.is_empty() {
        return Vec::new();
    }

    // Check cache first
    {
        let mut lock = CACHE.lock().unwrap();
        let cache = lock.get_or_insert_with(load_cache);
        if let Some(cached_covers) = cache.get(&key) {
            if !cached_covers.is_empty() {
                return cached_covers.clone();
            }
        }
    }

    let query = clean_query(game_name);
    let url = format!("{}/?title={}", PROXY_ENDPOINT, urlencoding_encode(&query));

    let res = client
        .get(&url)
        .header("User-Agent", "Restak-Client/1.0")
        .send()
        .await;

    let covers = match res {
        Ok(resp) => {
            if let Ok(data) = resp.json::<ProxyResponse>().await {
                if data.success {
                    data.covers
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            }
        }
        Err(_) => Vec::new(),
    };

    // Store in cache
    {
        let mut lock = CACHE.lock().unwrap();
        let cache = lock.get_or_insert_with(load_cache);
        cache.insert(key, covers.clone());
        save_cache(cache);
    }

    covers
}

pub async fn get_cover_art(client: &reqwest::Client, game_name: &str) -> Option<String> {
    let covers = get_covers(client, game_name).await;
    covers.into_iter().next()
}

fn urlencoding_encode(s: &str) -> String {
    let mut encoded = String::new();
    for b in s.bytes() {
        match b {
            b'a'..=b'z' | b'A'..=b'Z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                encoded.push(b as char);
            }
            b' ' => encoded.push_str("%20"),
            _ => encoded.push_str(&format!("%{:02X}", b)),
        }
    }
    encoded
}
