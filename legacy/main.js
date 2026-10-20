const { app, BrowserWindow } = require('electron');
const path = require('path');
const PronBrowWindow = require('./src/browser/BrowserWindow');

let mainWindow;

function createWindow() {
  mainWindow = new PronBrowWindow();
  mainWindow.show();
}

app.whenReady().then(() => {
  createWindow();

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow();
  });
});

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') app.quit();
});
