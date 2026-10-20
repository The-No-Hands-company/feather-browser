const { app, BrowserWindow, ipcMain } = require('electron');
const path = require('path');
const pronbrowEngine = require('@pronbrow/engine');
const PageRenderer = require('./src/renderer/PageRenderer');

let mainWindow;
let engine;
let pageRenderer;

function createWindow() {
  // Create the browser window
  mainWindow = new BrowserWindow({
    width: 1280,
    height: 900,
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false,
      enableRemoteModule: false
    },
    title: 'PronBrow - Custom Browser Engine',
    icon: path.join(__dirname, 'assets', 'icon.png')
  });

  // Initialize the pronbrow engine
  try {
    console.log('🚀 Initializing PronBrow Engine...');
    engine = pronbrowEngine.createBrowserEngine();
    console.log(`✓ Engine version: ${pronbrowEngine.getVersion()}`);
    
    // Create page renderer
    pageRenderer = new PageRenderer(engine, 1024, 768);
    console.log('✓ Page renderer initialized');
  } catch (error) {
    console.error('Failed to initialize engine:', error);
    app.quit();
    return;
  }

  // Load the browser UI
  mainWindow.loadFile('browser.html');

  // Open DevTools in development
  if (process.env.NODE_ENV === 'development') {
    mainWindow.webContents.openDevTools();
  }

  mainWindow.on('closed', () => {
    mainWindow = null;
    if (pageRenderer) {
      pageRenderer.destroy();
      pageRenderer = null;
    }
    engine = null;
  });

  console.log('✓ Browser window created');
}

// Handle page rendering requests from renderer process
ipcMain.handle('render-page', async (event, { url, width, height }) => {
  console.log(`📄 Render request: ${url} (${width}x${height})`);
  
  try {
    // Resize if needed
    if (width !== pageRenderer.canvasRenderer.width || height !== pageRenderer.canvasRenderer.height) {
      pageRenderer.resize(width, height);
    }
    
    // Navigate and render
    const result = await pageRenderer.navigate(url);
    
    if (result.success) {
      // Get the rendered pixels
      const html = pageRenderer.currentHtml || '';
      const css = pageRenderer.currentCss || '';
      
      const rgba_bytes = engine.renderPage(html, css, width, height);
      
      // Send the rendered frame to the renderer process
      mainWindow.webContents.send('frame-rendered', {
        width,
        height,
        pixels: Array.from(rgba_bytes)
      });
      
      return { success: true, url: result.url };
    } else {
      return { success: false, error: result.error };
    }
  } catch (error) {
    console.error('Render error:', error);
    return { success: false, error: error.message };
  }
});

// Handle JavaScript execution requests
ipcMain.handle('execute-script', async (event, code) => {
  try {
    const result = engine.executeJavascript(code);
    return { success: true, result };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Handle DOM query requests (for hit testing)
ipcMain.handle('query-dom', async (event, selector) => {
  try {
    const result = engine.getDomTree();
    return { success: true, result };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// Handle DOM layout queries (for hit testing)
ipcMain.handle('get-dom-layout', async () => {
  try {
    // Get DOM layout information for hit testing
    const layout = engine.getDomLayout ? engine.getDomLayout() : [];
    return { success: true, layout };
  } catch (error) {
    return { success: false, error: error.message, layout: [] };
  }
});

// Handle navigation events from user clicks
ipcMain.handle('navigate-to', async (event, url) => {
  console.log(`🔗 Navigation requested: ${url}`);
  try {
    // Navigate using the page renderer
    const result = await pageRenderer.navigate(url);
    
    if (result.success) {
      const html = pageRenderer.currentHtml || '';
      const css = pageRenderer.currentCss || '';
      const { width, height } = pageRenderer.canvasRenderer;
      
      const rgba_bytes = engine.renderPage(html, css, width, height);
      
      mainWindow.webContents.send('frame-rendered', {
        width,
        height,
        pixels: Array.from(rgba_bytes)
      });
      
      return { success: true, url: result.url };
    }
    
    return result;
  } catch (error) {
    console.error('Navigation error:', error);
    return { success: false, error: error.message };
  }
});

// Handle screenshot requests
ipcMain.handle('take-screenshot', async () => {
  try {
    const dataUrl = pageRenderer.screenshot();
    return { success: true, dataUrl };
  } catch (error) {
    return { success: false, error: error.message };
  }
});

// App lifecycle
app.whenReady().then(() => {
  createWindow();

  app.on('activate', () => {
    if (BrowserWindow.getAllWindows().length === 0) {
      createWindow();
    }
  });
});

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') {
    app.quit();
  }
});

app.on('before-quit', () => {
  if (pageRenderer) {
    pageRenderer.destroy();
  }
});

// Error handling
process.on('uncaughtException', (error) => {
  console.error('Uncaught exception:', error);
});

process.on('unhandledRejection', (reason, promise) => {
  console.error('Unhandled rejection at:', promise, 'reason:', reason);
});

console.log('🌐 PronBrow starting...');
console.log(`📍 Working directory: ${__dirname}`);
console.log(`🔧 Node version: ${process.version}`);
console.log(`⚡ Electron version: ${process.versions.electron}`);
