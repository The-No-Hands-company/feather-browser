const CanvasRenderer = require('./CanvasRenderer');
const https = require('https');
const http = require('http');
const fs = require('fs');

/**
 * Main page renderer using pronbrow-engine
 * Handles fetching, parsing, and rendering web pages
 */
class PageRenderer {
  constructor(engine, width = 1024, height = 768) {
    if (!engine) {
      throw new Error('PronBrow engine is required');
    }
    
    this.engine = engine;
    this.canvasRenderer = new CanvasRenderer(width, height);
    this.currentUrl = null;
    this.currentHtml = null;
    this.currentCss = null;
    this.isLoading = false;
  }

  /**
   * Set the target canvas element for rendering
   */
  setCanvas(canvasElement) {
    this.canvasRenderer.setTargetCanvas(canvasElement);
  }

  /**
   * Navigate to and render a URL
   */
  async navigate(url) {
    console.log(`📄 Navigating to: ${url}`);
    
    this.isLoading = true;
    this.currentUrl = url;
    this.canvasRenderer.renderLoading();

    try {
      // Handle different URL types
      if (url.startsWith('pronbrow://')) {
        await this.renderBuiltinPage(url);
      } else if (url.startsWith('file://')) {
        await this.renderLocalFile(url);
      } else if (url.startsWith('http://') || url.startsWith('https://')) {
        await this.renderRemotePage(url);
      } else {
        throw new Error(`Unsupported URL scheme: ${url}`);
      }

      this.isLoading = false;
      return { success: true, url: this.currentUrl };
    } catch (error) {
      console.error('Navigation error:', error);
      this.isLoading = false;
      this.renderErrorPage(error.message);
      return { success: false, error: error.message };
    }
  }

  /**
   * Render a built-in browser page (pronbrow://...)
   */
  async renderBuiltinPage(url) {
    const pageName = url.replace('pronbrow://', '');
    
    const pages = {
      'newtab': this.getNewTabHTML(),
      'settings': this.getSettingsHTML(),
      'blank': '<html><head><title>Blank Page</title></head><body></body></html>',
    };

    const html = pages[pageName] || this.getErrorHTML(`Unknown page: ${pageName}`);
    await this.renderHTML(html, '');
  }

  /**
   * Render a local file
   */
  async renderLocalFile(url) {
    const filePath = url.replace('file://', '');
    const html = fs.readFileSync(filePath, 'utf8');
    await this.renderHTML(html, '');
  }

  /**
   * Fetch and render a remote web page
   */
  async renderRemotePage(url) {
    const html = await this.fetchURL(url);
    
    // Extract CSS from HTML
    const cssMatches = html.match(/<style[^>]*>(.*?)<\/style>/gis);
    const css = cssMatches ? cssMatches.map(m => m.replace(/<\/?style[^>]*>/gi, '')).join('\n') : '';
    
    await this.renderHTML(html, css);
  }

  /**
   * Render HTML + CSS using pronbrow-engine
   */
  async renderHTML(html, css) {
    console.log('🎨 Rendering page...');
    this.currentHtml = html;
    this.currentCss = css;

    try {
      // Use pronbrow-engine to render
      const rgba_bytes = this.engine.renderPage(
        html,
        css,
        this.canvasRenderer.width,
        this.canvasRenderer.height
      );

      console.log(`✓ Engine returned ${rgba_bytes.length} bytes`);

      // Render pixels to canvas
      this.canvasRenderer.renderPixels(
        rgba_bytes,
        this.canvasRenderer.width,
        this.canvasRenderer.height
      );

      console.log('✓ Page rendered successfully');
    } catch (error) {
      console.error('Rendering error:', error);
      throw error;
    }
  }

  /**
   * Execute JavaScript on the current page
   */
  executeScript(code) {
    try {
      return this.engine.executeJavascript(code);
    } catch (error) {
      console.error('Script execution error:', error);
      throw error;
    }
  }

  /**
   * Fetch URL content
   */
  fetchURL(url) {
    return new Promise((resolve, reject) => {
      const protocol = url.startsWith('https') ? https : http;
      
      const request = protocol.get(url, (res) => {
        // Handle redirects
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return this.fetchURL(res.headers.location).then(resolve).catch(reject);
        }
        
        if (res.statusCode !== 200) {
          reject(new Error(`HTTP ${res.statusCode}: ${res.statusMessage}`));
          return;
        }
        
        let data = '';
        res.on('data', (chunk) => { data += chunk; });
        res.on('end', () => { resolve(data); });
      });

      request.on('error', (err) => {
        reject(new Error(`Network error: ${err.message}`));
      });

      request.setTimeout(10000, () => {
        request.destroy();
        reject(new Error('Request timeout'));
      });
    });
  }

  /**
   * Render error page
   */
  renderErrorPage(message) {
    const html = this.getErrorHTML(message);
    this.renderHTML(html, '').catch(err => {
      console.error('Failed to render error page:', err);
      this.canvasRenderer.renderError(message);
    });
  }

  /**
   * Get New Tab page HTML
   */
  getNewTabHTML() {
    return `
<!DOCTYPE html>
<html>
<head>
  <title>New Tab</title>
  <style>
    body {
      margin: 0;
      padding: 40px;
      font-family: Arial, sans-serif;
      background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
      color: white;
      text-align: center;
    }
    h1 {
      font-size: 48px;
      margin: 100px 0 20px 0;
    }
    p {
      font-size: 20px;
      opacity: 0.9;
    }
  </style>
</head>
<body>
  <h1>🌐 PronBrow</h1>
  <p>Built from scratch with 100% ES2025 JavaScript</p>
  <p>Enter a URL to start browsing</p>
</body>
</html>
    `;
  }

  /**
   * Get Settings page HTML
   */
  getSettingsHTML() {
    return `
<!DOCTYPE html>
<html>
<head>
  <title>Settings</title>
  <style>
    body {
      margin: 0;
      padding: 40px;
      font-family: Arial, sans-serif;
      background: #f5f5f5;
    }
    h1 { color: #333; }
    .setting {
      background: white;
      padding: 20px;
      margin: 10px 0;
      border-radius: 8px;
    }
  </style>
</head>
<body>
  <h1>⚙️ Settings</h1>
  <div class="setting">
    <h3>Engine Version</h3>
    <p>PronBrow Engine v0.3.0</p>
  </div>
  <div class="setting">
    <h3>JavaScript</h3>
    <p>ES2025 Compliance: 100%</p>
  </div>
</body>
</html>
    `;
  }

  /**
   * Get Error page HTML
   */
  getErrorHTML(message) {
    return `
<!DOCTYPE html>
<html>
<head>
  <title>Error</title>
  <style>
    body {
      margin: 0;
      padding: 40px;
      font-family: Arial, sans-serif;
      background: #f5f5f5;
    }
    .error-box {
      background: white;
      padding: 30px;
      border-radius: 8px;
      border-left: 4px solid #d32f2f;
      max-width: 600px;
      margin: 50px auto;
    }
    h1 {
      color: #d32f2f;
      margin-top: 0;
    }
    p {
      color: #666;
      line-height: 1.6;
    }
  </style>
</head>
<body>
  <div class="error-box">
    <h1>⚠️ Error Loading Page</h1>
    <p>${this.escapeHtml(message)}</p>
  </div>
</body>
</html>
    `;
  }

  /**
   * Escape HTML special characters
   */
  escapeHtml(text) {
    return text
      .replace(/&/g, '&amp;')
      .replace(/</g, '&lt;')
      .replace(/>/g, '&gt;')
      .replace(/"/g, '&quot;')
      .replace(/'/g, '&#039;');
  }

  /**
   * Resize viewport
   */
  resize(width, height) {
    this.canvasRenderer.resize(width, height);
    
    // Re-render current page if any
    if (this.currentHtml) {
      this.renderHTML(this.currentHtml, this.currentCss || '').catch(err => {
        console.error('Error re-rendering after resize:', err);
      });
    }
  }

  /**
   * Reload current page
   */
  async reload() {
    if (this.currentUrl) {
      return this.navigate(this.currentUrl);
    }
  }

  /**
   * Take screenshot
   */
  screenshot() {
    return this.canvasRenderer.toDataURL();
  }

  /**
   * Clean up
   */
  destroy() {
    this.canvasRenderer.destroy();
    this.engine = null;
    this.currentUrl = null;
    this.currentHtml = null;
    this.currentCss = null;
  }
}

module.exports = PageRenderer;
