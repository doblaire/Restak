const fs = require('fs');
const path = require('path');
const os = require('os');

const API_KEY = 'e268ef63d70edfb873d28e2714df1273';
const CACHE_FILE = path.join(os.homedir(), '.restak_cover_cache.json');

let cache = {};
try {
  if (fs.existsSync(CACHE_FILE)) {
    cache = JSON.parse(fs.readFileSync(CACHE_FILE, 'utf8'));
  }
} catch (e) {
  cache = {};
}

function saveCache() {
  try {
    fs.writeFileSync(CACHE_FILE, JSON.stringify(cache, null, 2), 'utf8');
  } catch (e) {}
}

/**
 * Clean game title for optimal search matching on SteamGridDB
 */
function cleanQuery(name) {
  return name
    .replace(/\s*\([^)]*\)|\s*\[[^\]]*\]/g, '') // remove brackets/parens
    .replace(/Rev \d+/i, '')
    .replace(/\.(nkit|iso|gba|nds|gcm|rvz|wbfs)/i, '')
    .trim();
}

/**
 * Fetch 600x900 cover art URL for a given game title from SteamGridDB.
 */
async function getCoverArt(gameName) {
  if (!gameName) return null;
  const key = gameName.toLowerCase().trim();
  if (cache[key] !== undefined) {
    return cache[key];
  }

  const query = cleanQuery(gameName);

  try {
    const searchRes = await fetch(
      'https://www.steamgriddb.com/api/v2/search/autocomplete/' + encodeURIComponent(query),
      {
        headers: { Authorization: 'Bearer ' + API_KEY },
      }
    );
    const searchData = await searchRes.json();

    if (!searchData.success || !searchData.data || searchData.data.length === 0) {
      cache[key] = null;
      saveCache();
      return null;
    }

    const gameId = searchData.data[0].id;

    // Fetch vertical grid covers (600x900)
    const gridRes = await fetch(
      'https://www.steamgriddb.com/api/v2/grids/game/' + gameId + '?dimensions=600x900',
      {
        headers: { Authorization: 'Bearer ' + API_KEY },
      }
    );
    const gridData = await gridRes.json();

    if (gridData.success && gridData.data && gridData.data.length > 0) {
      // Use thumb for fast, efficient rendering
      const coverUrl = gridData.data[0].thumb || gridData.data[0].url;
      cache[key] = coverUrl;
      saveCache();
      return coverUrl;
    }
  } catch (err) {
    console.error('SteamGridDB fetch error for', gameName, err);
  }

  cache[key] = null;
  saveCache();
  return null;
}

module.exports = { getCoverArt };
