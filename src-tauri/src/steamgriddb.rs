use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use serde::Deserialize;

const API_KEY: &str = "e268ef63d70edfb873d28e2714df1273";

static CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);

fn cache_file_path() -> Option<PathBuf> {
    dirs_home().map(|h| h.join(".restak_cover_cache.json"))
}

fn dirs_home() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn load_cache() -> HashMap<String, Option<String>> {
    if let Some(path) = cache_file_path() {
        if let Ok(data) = fs::read_to_string(&path) {
            if let Ok(map) = serde_json::from_str(&data) {
                return map;
            }
        }
    }
    HashMap::new()
}

fn save_cache(cache: &HashMap<String, Option<String>>) {
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
        .trim()
        .to_string()
}

#[derive(Deserialize)]
struct SearchResponse {
    success: bool,
    data: Option<Vec<SearchItem>>,
}

#[derive(Deserialize)]
struct SearchItem {
    id: u64,
}

#[derive(Deserialize)]
struct GridResponse {
    success: bool,
    data: Option<Vec<GridItem>>,
}

#[derive(Deserialize)]
struct GridItem {
    url: Option<String>,
    thumb: Option<String>,
}

pub async fn get_cover_art(client: &reqwest::Client, game_name: &str) -> Option<String> {
    let key = game_name.trim().to_lowercase();
    
    // Check cache
    {
        let mut lock = CACHE.lock().unwrap();
        let cache = lock.get_or_insert_with(load_cache);
        if let Some(cached_val) = cache.get(&key) {
            return cached_val.clone();
        }
    }

    let query = clean_query(game_name);
    let search_url = format!(
        "https://www.steamgriddb.com/api/v2/search/autocomplete/{}",
        urlencoding_encode(&query)
    );

    let res = client
        .get(&search_url)
        .header("Authorization", format!("Bearer {}", API_KEY))
        .send()
        .await;

    let game_id = match res {
        Ok(resp) => {
            if let Ok(search_data) = resp.json::<SearchResponse>().await {
                if search_data.success {
                    search_data.data.and_then(|items| items.into_iter().next().map(|i| i.id))
                } else {
                    None
                }
            } else {
                None
            }
        }
        Err(_) => None,
    };

    let cover_url = if let Some(id) = game_id {
        let grid_url = format!(
            "https://www.steamgriddb.com/api/v2/grids/game/{}?dimensions=600x900",
            id
        );

        let grid_res = client
            .get(&grid_url)
            .header("Authorization", format!("Bearer {}", API_KEY))
            .send()
            .await;

        match grid_res {
            Ok(resp) => {
                if let Ok(grid_data) = resp.json::<GridResponse>().await {
                    if grid_data.success {
                        grid_data.data.and_then(|items| {
                            items.into_iter().next().and_then(|g| g.thumb.or(g.url))
                        })
                    } else {
                        None
                    }
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    } else {
        None
    };

    // Store in cache
    {
        let mut lock = CACHE.lock().unwrap();
        let cache = lock.get_or_insert_with(load_cache);
        cache.insert(key, cover_url.clone());
        save_cache(cache);
    }

    cover_url
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
