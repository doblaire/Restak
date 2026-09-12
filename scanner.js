const { getCoverArt } = require('./steamgriddb');
const fs = require('fs');
const path = require('path');
const os = require('os');

// Known standalone game applications on macOS that might not specify App Store categories
const KNOWN_GAME_APPS = new Set([
  'Minecraft',
  'Roblox',
  'League of Legends',
  'World of Warcraft',
  'Diablo IV',
  'Hearthstone',
  'Overwatch',
  'Fortnite',
  'Valorant',
  'Genshin Impact',
  'Honkai: Star Rail',
  'Civilization VI',
  'Stardew Valley',
  'Terraria',
  'Hollow Knight',
  "Baldur's Gate 3",
]);

// Known emulators & helper tools to exclude from being shown as games
const EMULATOR_NAMES = new Set([
  'Dolphin',
  'RetroArch',
  'OpenEmu',
  'mGBA',
  'PCSX2',
  'Ryujinx',
  'Yuzu',
  'Citra',
  'DeSmuME',
  'PPSSPP',
  'DuckStation',
  'RPCS3',
  'Flycast',
  'Cemu',
  'Steam ROM Manager',
  'GCAdapterDriver',
]);

// Map file extensions to matching emulators
const EMULATOR_MAP = {
  // GameCube & Wii
  '.iso':  ['Dolphin', 'RetroArch'],
  '.gcm':  ['Dolphin', 'RetroArch'],
  '.rvz':  ['Dolphin', 'RetroArch'],
  '.wbfs': ['Dolphin', 'RetroArch'],
  '.ciso': ['Dolphin', 'RetroArch'],
  // Game Boy / GBA
  '.gba':  ['mGBA', 'RetroArch'],
  '.gbc':  ['mGBA', 'RetroArch'],
  '.gb':   ['mGBA', 'RetroArch'],
  // Nintendo DS / 3DS
  '.nds':  ['DeSmuME', 'RetroArch'],
  '.3ds':  ['Citra', 'RetroArch'],
  // Nintendo 64
  '.n64':  ['mupen64plus', 'RetroArch'],
  '.z64':  ['mupen64plus', 'RetroArch'],
  '.v64':  ['mupen64plus', 'RetroArch'],
  // NES / SNES
  '.nes':  ['Nestopia', 'RetroArch'],
  '.snes': ['Snes9x', 'RetroArch'],
  '.smc':  ['Snes9x', 'RetroArch'],
  '.sfc':  ['Snes9x', 'RetroArch'],
  // Switch
  '.nsp':  ['Ryujinx', 'Yuzu'],
  '.xci':  ['Ryujinx', 'Yuzu'],
  // PlayStation
  '.chd':  ['DuckStation', 'PCSX2', 'RetroArch'],
  '.cue':  ['RetroArch'],
  '.bin':  ['RetroArch'],
};

/**
 * Check whether an app bundle exists in /Applications or ~/Applications.
 */
function appExistsInApplications(appName) {
  const appFile = appName + '.app';
  try {
    fs.accessSync(path.join('/Applications', appFile), fs.constants.F_OK);
    return true;
  } catch {
    try {
      fs.accessSync(path.join(os.homedir(), 'Applications', appFile), fs.constants.F_OK);
      return true;
    } catch {
      return false;
    }
  }
}

/**
 * Resolve the user-friendly emulator name for a ROM extension.
 * Strictly returns 'No Launcher Found' if no matching emulator is installed on disk.
 */
function resolveEmulator(ext) {
  const candidates = EMULATOR_MAP[ext.toLowerCase()];
  if (!candidates || candidates.length === 0) return 'No Launcher Found';

  for (const candidate of candidates) {
    if (appExistsInApplications(candidate)) {
      return candidate === 'Dolphin' ? 'Dolphin Emulator' : candidate;
    }
  }

  return 'No Launcher Found';
}

function normalizeTitle(name) {
  return (name || '')
    .replace(/-/g, ' ')
    .toLowerCase()
    .replace(/[^a-z0-9\s]/g, '')
    .replace(/\s+/g, ' ')
    .trim();
}

function lookupGameMetadata(title) {
  const norm = normalizeTitle(title);
  if (norm === 'minecraft') return { publisher: 'Mojang Studios', year: '2011', genre: 'Sandbox' };
  if (norm === 'roblox') return { publisher: 'Roblox Corporation', year: '2006', genre: 'Sandbox' };
  if (norm.includes('super mario sunshine')) return { publisher: 'Nintendo', year: '2002', genre: 'Platformer' };
  if (norm.includes('super smash bros melee') || norm === 'melee') return { publisher: 'Nintendo', year: '2001', genre: 'Fighting' };
  if (norm.includes('super smash bros brawl') || norm === 'brawl') return { publisher: 'Nintendo', year: '2008', genre: 'Fighting' };
  if (norm.includes('super smash bros for nintendo 3ds') || norm.includes('smash 3ds')) return { publisher: 'Nintendo', year: '2014', genre: 'Fighting' };
  if (norm.includes('chibi robo')) return { publisher: 'Nintendo', year: '2005', genre: 'Action-Adventure' };
  if (norm.includes('bloons td 6') || norm === 'btd6') return { publisher: 'Ninja Kiwi', year: '2018', genre: 'Tower Defense' };
  if (norm.includes('mario kart double dash')) return { publisher: 'Nintendo', year: '2003', genre: 'Racing' };
  if (norm.includes('wind waker')) return { publisher: 'Nintendo', year: '2002', genre: 'Action-Adventure' };
  if (norm.includes('twilight princess')) return { publisher: 'Nintendo', year: '2006', genre: 'Action-Adventure' };
  if (norm.includes('metroid prime')) return { publisher: 'Nintendo', year: '2002', genre: 'Action-Adventure' };
  if (norm.includes('luigis mansion')) return { publisher: 'Nintendo', year: '2001', genre: 'Action-Adventure' };
  if (norm.includes('super mario 64')) return { publisher: 'Nintendo', year: '1996', genre: 'Platformer' };
  if (norm.includes('ocarina of time')) return { publisher: 'Nintendo', year: '1998', genre: 'Action-Adventure' };
  if (norm.includes('majoras mask')) return { publisher: 'Nintendo', year: '2000', genre: 'Action-Adventure' };
  if (norm.includes('pokemon emerald')) return { publisher: 'The Pokémon Company', year: '2004', genre: 'RPG' };
  if (norm.includes('super mario world')) return { publisher: 'Nintendo', year: '1990', genre: 'Platformer' };
  if (norm.includes('chrono trigger')) return { publisher: 'Square', year: '1995', genre: 'RPG' };
  if (norm.includes('baldurs gate 3')) return { publisher: 'Larian Studios', year: '2023', genre: 'RPG' };
  if (norm.includes('hollow knight')) return { publisher: 'Team Cherry', year: '2017', genre: 'Metroidvania' };
  if (norm.includes('stardew valley')) return { publisher: 'ConcernedApe', year: '2016', genre: 'Simulation' };
  if (norm.includes('terraria')) return { publisher: 'Re-Logic', year: '2011', genre: 'Sandbox' };
  if (norm.includes('civilization vi')) return { publisher: '2K Games', year: '2016', genre: 'Strategy' };
  if (norm.includes('league of legends')) return { publisher: 'Riot Games', year: '2009', genre: 'MOBA' };
  if (norm.includes('world of warcraft')) return { publisher: 'Blizzard Entertainment', year: '2004', genre: 'MMORPG' };
  if (norm.includes('portal 2')) return { publisher: 'Valve', year: '2011', genre: 'Puzzle' };
  if (norm.includes('portal')) return { publisher: 'Valve', year: '2007', genre: 'Puzzle' };
  return { publisher: null, year: null, genre: null };
}

/**
 * Clean ROM filenames into readable game titles.
 * E.g., "Super Smash Bros. Melee (USA) (En,Ja) (Rev 2).nkit.iso" -> "Super Smash Bros. Melee"
 */
function cleanRomName(filename) {
  let name = filename.replace(/\.(nkit\.)?[a-z0-9]+$/i, '');
  // Remove region tags and revision notes in parentheses or brackets
  name = name.replace(/\s*\([^)]*\)|\s*\[[^\]]*\]/g, '').trim();
  return name || filename;
}

/**
 * Determines if a given .app bundle is genuinely a game.
 */
function isGameApp(appName, fullPath) {
  if (EMULATOR_NAMES.has(appName)) return false;
  if (KNOWN_GAME_APPS.has(appName)) return true;

  const plistPath = path.join(fullPath, 'Contents', 'Info.plist');
  try {
    const buf = fs.readFileSync(plistPath);
    if (buf.includes('app-category.game')) {
      return true;
    }
  } catch {
    // Skip if unreadable
  }

  return false;
}

// ──────────────────────────────────────────────
// A. Native macOS Games in /Applications
// ──────────────────────────────────────────────
async function scanApplications() {
  const results = [];
  try {
    const entries = await fs.promises.readdir('/Applications');
    for (const name of entries) {
      if (!name.endsWith('.app')) continue;
      const appName = name.replace(/\.app$/, '');
      const fullPath = path.join('/Applications', name);

      if (isGameApp(appName, fullPath)) {
        const meta = lookupGameMetadata(appName);
        results.push({
          name: appName,
          path: fullPath,
          fullPath: fullPath,
          launcher: 'macOS',
          coverArt: null,
          genre: meta.genre,
          publisher: meta.publisher,
          year: meta.year,
        });
      }
    }
  } catch {
    // Skip on error
  }
  return results;
}

// ──────────────────────────────────────────────
// B. Steam Games
// ──────────────────────────────────────────────
async function scanSteam() {
  const results = [];
  const steamAppsDir = path.join(
    os.homedir(),
    'Library',
    'Application Support',
    'Steam',
    'steamapps',
  );

  try {
    const entries = await fs.promises.readdir(steamAppsDir);
    for (const entry of entries) {
      if (!entry.startsWith('appmanifest_') || !entry.endsWith('.acf')) continue;

      try {
        const filePath = path.join(steamAppsDir, entry);
        const content = await fs.promises.readFile(filePath, 'utf-8');

        const nameMatch = content.match(/"name"\s+"(.+?)"/);
        const installMatch = content.match(/"installdir"\s+"(.+?)"/);
        const appIdMatch = content.match(/"appid"\s+"(\d+)"/);

        if (nameMatch) {
          const gameName = nameMatch[1];
          let gamePath = steamAppsDir;
          if (installMatch) {
            const commonDir = path.join(steamAppsDir, 'common', installMatch[1]);
            if (fs.existsSync(commonDir)) {
              gamePath = commonDir;
            }
          }

          const meta = lookupGameMetadata(gameName);
          const hasSteam = appExistsInApplications('Steam');

          results.push({
            name: gameName,
            path: gamePath,
            fullPath: appIdMatch ? `steam://run/${appIdMatch[1]}` : gamePath,
            launcher: hasSteam ? 'Steam' : 'No Launcher Found',
            coverArt: null,
            genre: meta.genre,
            publisher: meta.publisher,
            year: meta.year,
          });
        }
      } catch {
        // Skip
      }
    }
  } catch {
    // Skip silently
  }
  return results;
}

// ──────────────────────────────────────────────
// C. ROM Files
// ──────────────────────────────────────────────
async function walkForRoms(dir, depth = 0, maxDepth = 4) {
  const results = [];
  if (depth > maxDepth) return results;

  try {
    const entries = await fs.promises.readdir(dir, { withFileTypes: true });
    for (const entry of entries) {
      const fullPath = path.join(dir, entry.name);

      if (entry.isDirectory()) {
        const nested = await walkForRoms(fullPath, depth + 1, maxDepth);
        results.push(...nested);
      } else if (entry.isFile()) {
        const extMatch = entry.name.match(/\.([a-z0-9]+)$/i);
        if (extMatch) {
          const ext = '.' + extMatch[1].toLowerCase();
          if (EMULATOR_MAP[ext]) {
            const cleaned = cleanRomName(entry.name);
            const meta = lookupGameMetadata(cleaned);
            results.push({
              name: cleaned,
              path: dir,
              fullPath: fullPath,
              launcher: resolveEmulator(ext),
              coverArt: null,
              genre: meta.genre,
              publisher: meta.publisher,
              year: meta.year,
            });
          }
        }
      }
    }
  } catch {
    // Skip
  }
  return results;
}

async function scanRoms() {
  const home = os.homedir();
  const candidateDirs = [
    path.join(home, 'Documents', 'ROMS'),
    path.join(home, 'Documents', 'ROMs'),
    path.join(home, 'ROMS'),
    path.join(home, 'ROMs'),
    path.join(home, 'Downloads'),
    path.join(home, 'Desktop', 'ROMS'),
    path.join(home, 'Desktop', 'ROMs'),
  ];

  const canonicalDirs = new Set();
  for (const dir of candidateDirs) {
    try {
      if (fs.existsSync(dir)) {
        const real = fs.realpathSync(dir);
        canonicalDirs.add(real);
      }
    } catch {
      // Skip
    }
  }

  const results = [];
  for (const dir of canonicalDirs) {
    const found = await walkForRoms(dir);
    results.push(...found);
  }
  return results;
}

// ──────────────────────────────────────────────
// D. Epic Games
// ──────────────────────────────────────────────
async function scanEpicGames() {
  const results = [];
  const manifestDir = path.join(
    os.homedir(),
    'Library',
    'Application Support',
    'Epic',
    'EpicGamesLauncher',
    'Data',
    'Manifests',
  );

  try {
    const entries = await fs.promises.readdir(manifestDir);
    for (const entry of entries) {
      if (!entry.endsWith('.item')) continue;

      try {
        const filePath = path.join(manifestDir, entry);
        const content = await fs.promises.readFile(filePath, 'utf-8');
        const manifest = JSON.parse(content);

        if (manifest.DisplayName) {
          results.push({
            name: manifest.DisplayName,
            path: manifest.InstallLocation || manifestDir,
            launcher: 'Epic Games',
            coverArt: null,
          });
        }
      } catch {
        // Skip
      }
    }
  } catch {
    // Skip
  }
  return results;
}

// ──────────────────────────────────────────────
// Main Exported Entry Point
// ──────────────────────────────────────────────
async function scanForGames() {
  const [apps, steam, roms, epic] = await Promise.all([
    scanApplications(),
    scanSteam(),
    scanRoms(),
    scanEpicGames(),
  ]);

  const combined = [...apps, ...steam, ...roms, ...epic];

  const seen = new Set();
  const deduplicated = [];
  for (const game of combined) {
    const key = game.name + ':::' + game.launcher;
    if (!seen.has(key)) {
      seen.add(key);
      deduplicated.push(game);
    }
  }

  // Fetch SteamGridDB cover art concurrently for all detected games
  await Promise.all(
    deduplicated.map(async (game) => {
      try {
        game.coverArt = await getCoverArt(game.name);
      } catch (e) {
        game.coverArt = null;
      }
    })
  );

  return deduplicated;
}

module.exports = { scanForGames };
