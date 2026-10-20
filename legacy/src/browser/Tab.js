const { BrowserView } = require('electron');
const EventEmitter = require('events');
const EngineRenderer = require('../engine-bridge/EngineRenderer');
const EngineIntegration = require('../engine-bridge/EngineIntegration');
const DevTools = require('../devtools/DevTools');

class Tab extends EventEmitter {
  constructor(browserWindow, url) {
    super();
    this.browserWindow = browserWindow;
    this.url = url;
    this.title = 'New Tab';
    this.favicon = null;
    this.isLoading = false;
    this.canGoBack = false;
    this.canGoForward = false;
    this.history = [];
    this.historyIndex = -1;
    
    // Always use PronBrow engine (we're building our own browser!)
    this.usePronBrowEngine = true;
    this.engine = new EngineIntegration(this);
    this.renderer = new EngineRenderer(this);
    this.devtools = new DevTools(this);
    
    console.log('✓ Tab created with PronBrow engine');
    
    // Initialize engine
    this.engine.initialize().catch(err => {
      console.error('Engine initialization failed:', err);
    });
    
    this.navigate(url);
  }

  navigate(url) {
    if (!url) return;
    
    // Normalize URL
    if (!url.match(/^[a-zA-Z]+:\/\//)) {
      if (url.startsWith('pronbrow://')) {
        // Keep pronbrow:// URLs as-is
      } else if (url.includes('.') && !url.includes(' ')) {
        url = 'https://' + url;
      } else {
        url = 'https://www.google.com/search?q=' + encodeURIComponent(url);
      }
    }
    
    this.url = url;
    this.isLoading = true;
    
    // Add to history
    if (this.historyIndex < this.history.length - 1) {
      this.history = this.history.slice(0, this.historyIndex + 1);
    }
    this.history.push(url);
    this.historyIndex = this.history.length - 1;
    
    this.updateNavigationState();
    
    if (this.usePronBrowEngine) {
      this.renderer.render(url);
    } else if (this.view) {
      this.view.webContents.loadURL(url);
    }
    
    this.notifyUpdate();
  }

  goBack() {
    if (this.canGoBack) {
      this.historyIndex--;
      this.url = this.history[this.historyIndex];
      this.updateNavigationState();
      
      if (this.usePronBrowEngine) {
        this.renderer.render(this.url);
      } else if (this.view) {
        this.view.webContents.goBack();
      }
      
      this.notifyUpdate();
    }
  }

  goForward() {
    if (this.canGoForward) {
      this.historyIndex++;
      this.url = this.history[this.historyIndex];
      this.updateNavigationState();
      
      if (this.usePronBrowEngine) {
        this.renderer.render(this.url);
      } else if (this.view) {
        this.view.webContents.goForward();
      }
      
      this.notifyUpdate();
    }
  }

  reload() {
    if (this.usePronBrowEngine) {
      this.renderer.render(this.url);
    } else if (this.view) {
      this.view.webContents.reload();
    }
    
    this.notifyUpdate();
  }

  updateNavigationState() {
    this.canGoBack = this.historyIndex > 0;
    this.canGoForward = this.historyIndex < this.history.length - 1;
  }

  setTitle(title) {
    this.title = title;
    this.notifyUpdate();
  }

  setFavicon(favicon) {
    this.favicon = favicon;
    this.notifyUpdate();
  }

  setLoading(loading) {
    this.isLoading = loading;
    this.notifyUpdate();
  }

  notifyUpdate() {
    this.browserWindow.window.webContents.send('tab-updated', this.getData());
  }

  getData() {
    return {
      url: this.url,
      title: this.title,
      favicon: this.favicon,
      isLoading: this.isLoading,
      canGoBack: this.canGoBack,
      canGoForward: this.canGoForward,
      usePronBrowEngine: this.usePronBrowEngine,
      devToolsOpen: this.devtools ? this.devtools.isOpen : false
    };
  }

  executeJavaScript(code) {
    if (this.renderer) {
      return this.renderer.executeJavaScript(code);
    }
    return null;
  }

  openDevTools() {
    if (this.devtools) {
      this.devtools.open();
      this.notifyUpdate();
    }
  }

  closeDevTools() {
    if (this.devtools) {
      this.devtools.close();
      this.notifyUpdate();
    }
  }

  toggleDevTools() {
    if (this.devtools) {
      this.devtools.toggle();
      this.notifyUpdate();
    }
  }

  destroy() {
    if (this.view) {
      this.view.destroy();
    }
    if (this.renderer) {
      this.renderer.destroy();
    }
    if (this.devtools) {
      this.devtools.destroy();
    }
  }
}

module.exports = Tab;
