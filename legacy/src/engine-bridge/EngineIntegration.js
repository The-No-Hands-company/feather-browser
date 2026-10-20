const { spawn } = require('child_process');
const EventEmitter = require('events');
const path = require('path');
const DOMBridge = require('./DOMBridge');
const EnhancedJSRuntime = require('./EnhancedJSRuntime');

/**
 * PronBrow Engine Integration
 * Manages communication between Electron and the Rust-based rendering engine
 */
class EngineIntegration extends EventEmitter {
  constructor(tab) {
    super();
    this.tab = tab;
    this.engineProcess = null;
    this.domBridge = null;
    this.jsRuntime = null;
    this.isInitialized = false;
    this.renderCallbacks = new Map();
    this.nextCallbackId = 1;
    
    this.capabilities = {
      html5: true,
      css3: true,
      javascript: true,
      webgl: false,
      webrtc: false,
      serviceWorkers: false,
      webassembly: false,
    };
  }

  /**
   * Initialize the engine
   */
  async initialize() {
    if (this.isInitialized) return;

    console.log('Initializing PronBrow Engine...');

    try {
      // Try to load the native Rust engine
      try {
        const path = require('path');
        const enginePath = path.join(__dirname, '../../pronbrow-engine');
        const engineModule = require(enginePath);
        this.nativeEngine = engineModule.createBrowserEngine();
        console.log('✓ Native Rust engine loaded');
      } catch (err) {
        console.warn('Native engine not available, using JS fallback:', err.message);
        this.nativeEngine = null;
      }
      
      // Initialize DOM Bridge
      this.domBridge = new DOMBridge(this, null);
      
      // Initialize JavaScript Runtime
      this.jsRuntime = new EnhancedJSRuntime(this.tab, this.domBridge, this);
      
      // Connect runtime to DOM bridge
      this.domBridge.jsContext = this.jsRuntime;
      this.domBridge.nativeEngine = this.nativeEngine;
      
      // Set up event listeners
      this.setupEventListeners();
      
      // Try to spawn the Rust engine process
      await this.spawnEngineProcess();
      
      this.isInitialized = true;
      console.log('✓ PronBrow Engine initialized');
      this.emit('initialized');
    } catch (error) {
      console.error('Engine initialization error:', error);
      this.emit('error', error);
      throw error;
    }
  }

  /**
   * Spawn the Rust engine process
   */
  async spawnEngineProcess() {
    const enginePath = path.join(__dirname, '../../..', 'pronbrow-engine', 'target', 'debug', 'pronbrow');
    
    return new Promise((resolve, reject) => {
      try {
        // Check if engine binary exists
        const fs = require('fs');
        if (!fs.existsSync(enginePath)) {
          console.warn('Engine binary not found, running in fallback mode');
          this.capabilities.nativeRendering = false;
          resolve();
          return;
        }

        this.engineProcess = spawn(enginePath, [], {
          stdio: ['pipe', 'pipe', 'pipe'],
        });

        this.engineProcess.stdout.on('data', (data) => {
          this.handleEngineOutput(data.toString());
        });

        this.engineProcess.stderr.on('data', (data) => {
          console.error('[Engine Error]', data.toString());
        });

        this.engineProcess.on('error', (error) => {
          console.error('Engine process error:', error);
          this.capabilities.nativeRendering = false;
        });

        this.engineProcess.on('close', (code) => {
          console.log(`Engine process exited with code ${code}`);
          this.engineProcess = null;
        });

        this.capabilities.nativeRendering = true;
        console.log('✓ Engine process spawned');
        resolve();
      } catch (error) {
        console.warn('Could not spawn engine process:', error.message);
        this.capabilities.nativeRendering = false;
        resolve(); // Continue in fallback mode
      }
    });
  }

  /**
   * Handle engine output
   */
  handleEngineOutput(data) {
    try {
      const lines = data.split('\n').filter(l => l.trim());
      for (const line of lines) {
        if (line.startsWith('{')) {
          const message = JSON.parse(line);
          this.handleEngineMessage(message);
        } else {
          console.log('[Engine]', line);
        }
      }
    } catch (error) {
      console.error('Error parsing engine output:', error);
    }
  }

  /**
   * Handle engine messages
   */
  handleEngineMessage(message) {
    switch (message.type) {
      case 'render-complete':
        this.emit('render-complete', message.data);
        const callback = this.renderCallbacks.get(message.id);
        if (callback) {
          callback(null, message.data);
          this.renderCallbacks.delete(message.id);
        }
        break;
        
      case 'dom-updated':
        this.emit('dom-updated', message.data);
        break;
        
      case 'layout-complete':
        this.emit('layout-complete', message.data);
        break;
        
      case 'paint-complete':
        this.emit('paint-complete', message.data);
        break;
        
      case 'error':
        this.emit('engine-error', message.error);
        const errorCallback = this.renderCallbacks.get(message.id);
        if (errorCallback) {
          errorCallback(new Error(message.error), null);
          this.renderCallbacks.delete(message.id);
        }
        break;
        
      default:
        console.log('Unknown engine message:', message);
    }
  }

  /**
   * Send command to engine
   */
  sendCommand(command) {
    if (!this.engineProcess) {
      console.warn('Engine process not available');
      return;
    }

    try {
      this.engineProcess.stdin.write(JSON.stringify(command) + '\n');
    } catch (error) {
      console.error('Error sending command to engine:', error);
    }
  }

  /**
   * Set up event listeners
   */
  setupEventListeners() {
    // JS Runtime events
    this.jsRuntime.on('console', (data) => {
      this.emit('console', data);
    });

    this.jsRuntime.on('script-error', (data) => {
      this.emit('script-error', data);
    });

    this.jsRuntime.on('script-executed', (data) => {
      this.emit('script-executed', data);
    });

    // DOM Bridge events
    this.domBridge.on('dom-changed', (data) => {
      this.emit('dom-changed', data);
      // Send to engine for re-render
      this.sendCommand({
        type: 'update-dom',
        data: this.serializeDOM(),
      });
    });
  }

  /**
   * Load and render a web page
   */
  async loadPage(url, html, css) {
    console.log('Loading page:', url);

    try {
      // Parse HTML and build DOM
      this.domBridge.parseHTML(html);
      
      // Extract and execute scripts
      await this.domBridge.executeScripts(html);
      
      // Send to engine for rendering
      if (this.engineProcess) {
        const renderData = {
          html,
          css,
          url,
          viewport: {
            width: 1024,
            height: 768,
          },
        };

        const callbackId = this.nextCallbackId++;
        
        return new Promise((resolve, reject) => {
          this.renderCallbacks.set(callbackId, (error, data) => {
            if (error) reject(error);
            else resolve(data);
          });

          this.sendCommand({
            type: 'render',
            id: callbackId,
            data: renderData,
          });

          // Timeout after 30 seconds
          setTimeout(() => {
            if (this.renderCallbacks.has(callbackId)) {
              this.renderCallbacks.delete(callbackId);
              reject(new Error('Render timeout'));
            }
          }, 30000);
        });
      } else {
        // Fallback rendering (canvas-based)
        return this.fallbackRender(html, css);
      }
    } catch (error) {
      console.error('Error loading page:', error);
      throw error;
    }
  }

  /**
   * Fallback rendering when engine is not available
   */
  async fallbackRender(html, css) {
    console.log('Using fallback rendering');
    
    // We can't use document in Node.js main process
    // Instead, return a simple render result
    return {
      type: 'fallback',
      html,
      css,
      message: 'PronBrow running in fallback mode'
    };
  }

  /**
   * Execute JavaScript code
   */
  executeScript(code, filename = 'inline') {
    if (!this.jsRuntime) {
      throw new Error('JavaScript runtime not initialized');
    }
    
    return this.jsRuntime.execute(code, filename);
  }

  /**
   * Evaluate JavaScript expression
   */
  evaluateExpression(expression) {
    if (!this.jsRuntime) {
      throw new Error('JavaScript runtime not initialized');
    }
    
    return this.jsRuntime.evaluate(expression);
  }

  /**
   * Get DOM element by selector
   */
  querySelector(selector) {
    if (!this.domBridge) return null;
    return this.domBridge.querySelector(this.domBridge.domTree, selector);
  }

  /**
   * Get all DOM elements by selector
   */
  querySelectorAll(selector) {
    if (!this.domBridge) return [];
    return this.domBridge.querySelectorAll(this.domBridge.domTree, selector);
  }

  /**
   * Serialize DOM for engine
   */
  serializeDOM() {
    if (!this.domBridge || !this.domBridge.domTree) {
      return null;
    }

    return {
      type: 'document',
      children: this.serializeNode(this.domBridge.domTree),
    };
  }

  /**
   * Serialize a single node
   */
  serializeNode(node) {
    if (!node) return null;

    const serialized = {
      type: node.type || 'element',
      tagName: node.tagName,
      attributes: node.attributes || {},
      textContent: node.textContent || '',
    };

    if (node.children && node.children.length > 0) {
      serialized.children = node.children.map(child => this.serializeNode(child));
    }

    return serialized;
  }

  /**
   * Get engine capabilities
   */
  getCapabilities() {
    return { ...this.capabilities };
  }

  /**
   * Take screenshot
   */
  async screenshot() {
    if (this.engineProcess) {
      return new Promise((resolve, reject) => {
        const callbackId = this.nextCallbackId++;
        
        this.renderCallbacks.set(callbackId, (error, data) => {
          if (error) reject(error);
          else resolve(data);
        });

        this.sendCommand({
          type: 'screenshot',
          id: callbackId,
        });

        setTimeout(() => {
          if (this.renderCallbacks.has(callbackId)) {
            this.renderCallbacks.delete(callbackId);
            reject(new Error('Screenshot timeout'));
          }
        }, 10000);
      });
    } else {
      // Return fallback render
      return this.fallbackRender('', '');
    }
  }

  /**
   * Get performance metrics
   */
  getPerformanceMetrics() {
    return {
      domNodes: this.domBridge?.elementMap.size || 0,
      scriptsExecuted: 0,
      renderTime: 0,
      layoutTime: 0,
      paintTime: 0,
    };
  }

  /**
   * Reload page
   */
  async reload() {
    console.log('Reloading page...');
    // TODO: Re-fetch and render current page
  }

  /**
   * Parse HTML using the engine
   */
  parseHtml(html) {
    // If native engine is available, send parse command
    if (this.engineProcess) {
      return new Promise((resolve, reject) => {
        const callbackId = this.nextCallbackId++;
        this.renderCallbacks.set(callbackId, (error, data) => {
          if (error) reject(error);
          else resolve(data);
        });

        this.sendCommand({
          type: 'parse-html',
          id: callbackId,
          data: { html },
        });

        setTimeout(() => {
          if (this.renderCallbacks.has(callbackId)) {
            this.renderCallbacks.delete(callbackId);
            reject(new Error('Parse timeout'));
          }
        }, 5000);
      });
    }
    
    // Fallback: Use DOMBridge's parseHTML method
    return this.domBridge.parseHTML(html);
  }

  /**
   * Parse CSS using the engine
   */
  parseCSS(css) {
    // If native engine is available, send parse command
    if (this.engineProcess) {
      return new Promise((resolve, reject) => {
        const callbackId = this.nextCallbackId++;
        this.renderCallbacks.set(callbackId, (error, data) => {
          if (error) reject(error);
          else resolve(data);
        });

        this.sendCommand({
          type: 'parse-css',
          id: callbackId,
          data: { css },
        });

        setTimeout(() => {
          if (this.renderCallbacks.has(callbackId)) {
            this.renderCallbacks.delete(callbackId);
            reject(new Error('Parse timeout'));
          }
        }, 5000);
      });
    }
    
    // Fallback: Return basic parsed structure
    return { rules: [] };
  }

  /**
   * Clean up
   */
  destroy() {
    console.log('Destroying engine integration...');

    if (this.jsRuntime) {
      this.jsRuntime.destroy();
      this.jsRuntime = null;
    }

    if (this.domBridge) {
      this.domBridge.destroy();
      this.domBridge = null;
    }

    if (this.engineProcess) {
      this.engineProcess.kill();
      this.engineProcess = null;
    }

    this.renderCallbacks.clear();
    this.removeAllListeners();
    this.isInitialized = false;
  }
}

module.exports = EngineIntegration;
