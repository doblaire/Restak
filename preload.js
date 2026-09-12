const { contextBridge, ipcRenderer } = require('electron');

contextBridge.exposeInMainWorld('restak', {
  scanForGames: () => ipcRenderer.invoke('scan-for-games'),
  loadLibrary: () => ipcRenderer.invoke('load-library'),
  saveLibrary: (games) => ipcRenderer.invoke('save-library', games),
  pickLauncher: () => ipcRenderer.invoke('pick-launcher'),
  launchGame: (game) => ipcRenderer.invoke('launch-game', game),
});
