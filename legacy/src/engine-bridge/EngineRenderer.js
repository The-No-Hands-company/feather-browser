const fs = require('fs');
const path = require('path');
const https = require('https');
const http = require('http');

class EngineRenderer {
  constructor(tab) {
    this.tab = tab;
    console.log('✓ EngineRenderer initialized');
  }

  async render(url) {
    try {
      this.tab.setLoading(true);
      
      // Use engine integration
      if (!this.tab.engine) {
        throw new Error('Engine not initialized');
      }
      
      // Handle special URLs
      if (url.startsWith('pronbrow://')) {
        await this.renderPronBrowPage(url);
      } else if (url.startsWith('file://')) {
        await this.renderLocalFile(url);
      } else {
        await this.renderRemoteURL(url);
      }
      
      this.tab.setLoading(false);
    } catch (error) {
      console.error('Rendering error:', error);
      this.tab.setLoading(false);
      this.renderErrorPage(error);
    }
  }

  async renderPronBrowPage(url) {
    const page = url.replace('pronbrow://', '');
    
    const pageHTML = {
      'newtab': this.getNewTabHTML(),
      'settings': this.getSettingsHTML(),
      'history': this.getHistoryHTML(),
      'bookmarks': this.getBookmarksHTML(),
    };
    
    const html = pageHTML[page] || this.getErrorHTML('Unknown page');
    const title = page.charAt(0).toUpperCase() + page.slice(1);
    
    this.tab.setTitle(title);
    await this.tab.engine.loadPage(url, html, '');
  }

  async renderLocalFile(url) {
    const filePath = url.replace('file://', '');
    
    try {
      const content = fs.readFileSync(filePath, 'utf8');
      const ext = path.extname(filePath).toLowerCase();
      
      let html = content;
      if (ext !== '.html' && ext !== '.htm') {
        html = `<html><body><pre>${content}</pre></body></html>`;
      }
      
      this.tab.setTitle(path.basename(filePath));
      await this.tab.engine.loadPage(url, html, '');
    } catch (error) {
      this.renderErrorPage(error);
    }
  }

  async renderRemoteURL(url) {
    try {
      const html = await this.fetchURL(url);
      
      // Extract title
      const titleMatch = html.match(/<title>(.*?)<\/title>/i);
      if (titleMatch) {
        this.tab.setTitle(titleMatch[1]);
      }
      
      // Extract CSS
      const cssMatches = html.match(/<style[^>]*>(.*?)<\/style>/gis);
      const css = cssMatches ? cssMatches.map(m => m.replace(/<\/?style[^>]*>/gi, '')).join('\n') : '';
      
      await this.tab.engine.loadPage(url, html, css);
    } catch (error) {
      console.error('Error rendering remote URL:', error);
      this.renderErrorPage(error);
    }
  }

  fetchURL(url) {
    return new Promise((resolve, reject) => {
      const protocol = url.startsWith('https') ? https : http;
      
      protocol.get(url, (res) => {
        // Handle redirects
        if (res.statusCode >= 300 && res.statusCode < 400 && res.headers.location) {
          return this.fetchURL(res.headers.location).then(resolve).catch(reject);
        }
        
        let data = '';
        res.on('data', (chunk) => { data += chunk; });
        res.on('end', () => { resolve(data); });
      }).on('error', (err) => {
        reject(err);
      });
    });
  }

  renderErrorPage(error) {
    const errorHTML = this.getErrorHTML(error.message);
    this.tab.setTitle('Error');
    this.tab.engine.loadPage('about:error', errorHTML, '').catch(err => {
      console.error('Failed to render error page:', err);
    });
  }

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
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
              background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
              color: white;
              min-height: 100vh;
              display: flex;
              flex-direction: column;
              align-items: center;
              justify-content: center;
            }
            h1 {
              font-size: 3em;
              margin: 0 0 20px 0;
              text-shadow: 2px 2px 4px rgba(0,0,0,0.3);
            }
            p {
              font-size: 1.2em;
              opacity: 0.9;
            }
            .shortcuts {
              display: grid;
              grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
              gap: 20px;
              margin-top: 40px;
              width: 100%;
              max-width: 600px;
            }
            .shortcut {
              background: rgba(255,255,255,0.1);
              padding: 20px;
              border-radius: 10px;
              text-align: center;
              cursor: pointer;
              transition: all 0.3s;
            }
            .shortcut:hover {
              background: rgba(255,255,255,0.2);
              transform: translateY(-2px);
            }
          </style>
        </head>
        <body>
          <h1>🌐 PronBrow</h1>
          <p>Your Privacy-First Web Browser</p>
          <div class="shortcuts">
            <div class="shortcut" onclick="location.href='pronbrow://settings'">
              ⚙️<br>Settings
            </div>
            <div class="shortcut" onclick="location.href='pronbrow://history'">
              📜<br>History
            </div>
            <div class="shortcut" onclick="location.href='pronbrow://bookmarks'">
              ⭐<br>Bookmarks
            </div>
          </div>
        </body>
      </html>
    `;
  }

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
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
              background: #f5f5f5;
            }
            h1 { color: #333; }
            .setting {
              background: white;
              padding: 20px;
              margin: 10px 0;
              border-radius: 8px;
              box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            }
          </style>
        </head>
        <body>
          <h1>Settings</h1>
          <div class="setting">
            <h3>Privacy</h3>
            <p>Configure privacy and security settings</p>
          </div>
          <div class="setting">
            <h3>Appearance</h3>
            <p>Customize the browser appearance</p>
          </div>
          <div class="setting">
            <h3>Advanced</h3>
            <p>Advanced browser configuration</p>
          </div>
        </body>
      </html>
    `;
  }

  getHistoryHTML() {
    return `
      <!DOCTYPE html>
      <html>
        <head>
          <title>History</title>
          <style>
            body {
              margin: 0;
              padding: 40px;
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
              background: #f5f5f5;
            }
            h1 { color: #333; }
          </style>
        </head>
        <body>
          <h1>Browsing History</h1>
          <p>Your browsing history will appear here.</p>
        </body>
      </html>
    `;
  }

  getBookmarksHTML() {
    return `
      <!DOCTYPE html>
      <html>
        <head>
          <title>Bookmarks</title>
          <style>
            body {
              margin: 0;
              padding: 40px;
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
              background: #f5f5f5;
            }
            h1 { color: #333; }
          </style>
        </head>
        <body>
          <h1>Bookmarks</h1>
          <p>Your bookmarks will appear here.</p>
        </body>
      </html>
    `;
  }

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
              font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', sans-serif;
              background: #f5f5f5;
            }
            h1 { color: #d32f2f; }
            .error-box {
              background: white;
              padding: 30px;
              border-radius: 8px;
              border-left: 4px solid #d32f2f;
              box-shadow: 0 2px 4px rgba(0,0,0,0.1);
            }
          </style>
        </head>
        <body>
          <div class="error-box">
            <h1>⚠️ Error Loading Page</h1>
            <p>${message}</p>
          </div>
        </body>
      </html>
    `;
  }

  executeJavaScript(code, filename = 'console') {
    return this.tab.engine.executeScript(code, filename);
  }

  evaluateExpression(expression) {
    return this.tab.engine.evaluateExpression(expression);
  }

  destroy() {
    // Cleanup if needed
  }
}

module.exports = EngineRenderer;
