const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');

function createWindow() {
  const win = new BrowserWindow({
    width: 1920,
    height: 1080,
    minWidth: 960,
    minHeight: 540,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      nodeIntegration: false,
      contextIsolation: true,
    },
    backgroundColor: '#000000',
    show: false,
  });

  win.loadFile('index.html');

  win.once('ready-to-show', () => {
    win.show();
  });
}

const fs = require('fs');
const os = require('os');
const { spawn, exec } = require('child_process');
const { scanForGames } = require('./scanner');

const libraryFile = path.join(os.homedir(), '.restak_library.json');

app.whenReady().then(createWindow);

ipcMain.handle('scan-for-games', async () => {
  return await scanForGames();
});

ipcMain.handle('load-library', async () => {
  try {
    if (fs.existsSync(libraryFile)) {
      const data = await fs.promises.readFile(libraryFile, 'utf-8');
      return JSON.parse(data);
    }
  } catch (e) {
    console.error('Failed to load library:', e);
  }
  return [];
});

ipcMain.handle('save-library', async (_event, games) => {
  try {
    await fs.promises.writeFile(libraryFile, JSON.stringify(games, null, 2));
    return true;
  } catch (e) {
    console.error('Failed to save library:', e);
    return false;
  }
});

ipcMain.handle('pick-launcher', async () => {
  return new Promise((resolve) => {
    const script = `
      try
        set appPath to POSIX path of (choose file of type {"app"} with prompt "Select Game Launcher / Emulator" default location (POSIX file "/Applications"))
        return appPath
      on error
        return ""
      end try
    `;
    exec(`osascript -e '${script.replace(/'/g, "'\\''")}'`, (err, stdout) => {
      if (err || !stdout.trim()) {
        resolve(null);
      } else {
        resolve(stdout.trim());
      }
    });
  });
});

ipcMain.handle('launch-game', async (_event, game) => {
  const customPath = game.customLauncherPath || '';
  const fullPath = game.fullPath || game.path;
  const launcherLower = (game.launcher || '').toLowerCase();

  let child;
  if (customPath) {
    child = spawn('open', ['-a', customPath, '--args', fullPath]);
  } else if (launcherLower.includes('dolphin')) {
    const dolphinBin = '/Applications/Dolphin.app/Contents/MacOS/Dolphin';
    if (fs.existsSync(dolphinBin)) {
      child = spawn(dolphinBin, ['-b', '-e', fullPath]);
    } else {
      child = spawn('open', ['-a', 'Dolphin', fullPath]);
    }
  } else if (launcherLower.includes('steam') || (fullPath && fullPath.startsWith('steam://'))) {
    child = spawn('open', [fullPath || 'steam://open']);
  } else {
    child = spawn('open', ['-a', fullPath]);
  }

  if (child) {
    child.on('exit', () => {
      exec('osascript -e \'tell application "Restak" to activate\'');
    });
  }
  return true;
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit();
});

app.on('activate', () => {
  if (BrowserWindow.getAllWindows().length === 0) createWindow();
});
