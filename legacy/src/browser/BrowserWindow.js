const { BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const Tab = require('./Tab');
const History = require('./History');
const Bookmarks = require('./Bookmarks');

class PronBrowWindow {
  constructor() {
    this.tabs = [];
    this.activeTabIndex = 0;
    this.history = new History();
    this.bookmarks = new Bookmarks();
    this.createWindow();
    this.setupIPC();
  }

  createWindow() {
    this.window = new BrowserWindow({
      width: 1400,
      height: 900,
      minWidth: 800,
      minHeight: 600,
      frame: false,
      titleBarStyle: 'hidden',
      webPreferences: {
        nodeIntegration: true,
        contextIsolation: false,
        webviewTag: true,
        enableRemoteModule: true
      }
    });

    this.window.loadFile(path.join(__dirname, '../ui/browser.html'));
    
    this.window.on('closed', () => {
      this.tabs.forEach(tab => tab.destroy());
      this.window = null;
    });
  }

  setupIPC() {
    ipcMain.on('new-tab', (event, url) => {
      this.createTab(url || 'pronbrow://newtab');
    });

    ipcMain.on('close-tab', (event, index) => {
      this.closeTab(index);
    });

    ipcMain.on('switch-tab', (event, index) => {
      this.switchTab(index);
    });

    ipcMain.on('navigate', (event, url) => {
      this.navigate(url);
    });

    ipcMain.on('go-back', () => {
      this.goBack();
    });

    ipcMain.on('go-forward', () => {
      this.goForward();
    });

    ipcMain.on('reload', () => {
      this.reload();
    });

    ipcMain.on('add-bookmark', (event, data) => {
      this.bookmarks.add(data);
    });

    ipcMain.on('get-bookmarks', (event) => {
      event.returnValue = this.bookmarks.getAll();
    });

    ipcMain.on('get-history', (event) => {
      event.returnValue = this.history.getRecent(50);
    });

    ipcMain.on('open-devtools', () => {
      this.window.webContents.openDevTools({ mode: 'detach' });
    });

    ipcMain.on('window-minimize', () => {
      this.window.minimize();
    });

    ipcMain.on('window-maximize', () => {
      if (this.window.isMaximized()) {
        this.window.unmaximize();
      } else {
        this.window.maximize();
      }
    });

    ipcMain.on('window-close', () => {
      this.window.close();
    });
  }

  createTab(url = 'pronbrow://newtab') {
    const tab = new Tab(this, url);
    this.tabs.push(tab);
    this.activeTabIndex = this.tabs.length - 1;
    
    this.window.webContents.send('tab-created', {
      index: this.activeTabIndex,
      url: url,
      title: 'New Tab'
    });
    
    return tab;
  }

  closeTab(index) {
    if (this.tabs.length === 1) {
      this.window.close();
      return;
    }

    const tab = this.tabs[index];
    if (tab) {
      tab.destroy();
      this.tabs.splice(index, 1);
      
      if (this.activeTabIndex >= index && this.activeTabIndex > 0) {
        this.activeTabIndex--;
      }
      
      this.window.webContents.send('tab-closed', {
        index,
        newActiveIndex: this.activeTabIndex
      });
    }
  }

  switchTab(index) {
    if (index >= 0 && index < this.tabs.length) {
      this.activeTabIndex = index;
      this.window.webContents.send('tab-switched', {
        index,
        tab: this.tabs[index].getData()
      });
    }
  }

  getActiveTab() {
    return this.tabs[this.activeTabIndex];
  }

  navigate(url) {
    const tab = this.getActiveTab();
    if (tab) {
      tab.navigate(url);
      this.history.add({
        url,
        title: tab.title,
        timestamp: Date.now()
      });
    }
  }

  goBack() {
    const tab = this.getActiveTab();
    if (tab) tab.goBack();
  }

  goForward() {
    const tab = this.getActiveTab();
    if (tab) tab.goForward();
  }

  reload() {
    const tab = this.getActiveTab();
    if (tab) tab.reload();
  }

  show() {
    this.window.show();
  }
}

module.exports = PronBrowWindow;
