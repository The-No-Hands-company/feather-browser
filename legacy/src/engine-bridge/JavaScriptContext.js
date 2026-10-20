const vm = require('vm');
const EventEmitter = require('events');

/**
 * JavaScript execution context for web pages
 * Provides a secure sandboxed environment for running page scripts
 */
class JavaScriptContext extends EventEmitter {
  constructor(tab) {
    super();
    this.tab = tab;
    this.context = null;
    this.scripts = [];
    this.timers = new Map();
    this.intervals = new Map();
    this.nextTimerId = 1;
    
    this.initializeContext();
  }

  initializeContext() {
    // Create sandbox with Web APIs
    const sandbox = {
      console: this.createConsoleAPI(),
      window: {},
      document: this.createDocumentAPI(),
      navigator: this.createNavigatorAPI(),
      location: this.createLocationAPI(),
      setTimeout: this.createSetTimeout(),
      setInterval: this.createSetInterval(),
      clearTimeout: this.createClearTimeout(),
      clearInterval: this.createClearInterval(),
      fetch: this.createFetchAPI(),
      XMLHttpRequest: this.createXMLHttpRequest(),
      localStorage: this.createStorageAPI('local'),
      sessionStorage: this.createStorageAPI('session'),
      performance: this.createPerformanceAPI(),
      requestAnimationFrame: this.createRequestAnimationFrame(),
      cancelAnimationFrame: this.createCancelAnimationFrame(),
      
      // Global functions
      parseInt: parseInt,
      parseFloat: parseFloat,
      isNaN: isNaN,
      isFinite: isFinite,
      encodeURI: encodeURI,
      encodeURIComponent: encodeURIComponent,
      decodeURI: decodeURI,
      decodeURIComponent: decodeURIComponent,
      
      // Global objects
      Array: Array,
      Object: Object,
      String: String,
      Number: Number,
      Boolean: Boolean,
      Date: Date,
      RegExp: RegExp,
      Error: Error,
      Math: Math,
      JSON: JSON,
      Promise: Promise,
      Set: Set,
      Map: Map,
      WeakSet: WeakSet,
      WeakMap: WeakMap,
      Symbol: Symbol,
      Proxy: Proxy,
      Reflect: Reflect,
    };

    // Make window reference itself
    sandbox.window = sandbox;
    sandbox.self = sandbox;
    sandbox.globalThis = sandbox;

    this.context = vm.createContext(sandbox);
    console.log('✓ JavaScript context initialized');
  }

  createConsoleAPI() {
    return {
      log: (...args) => {
        console.log('[Page]', ...args);
        this.emit('console', { type: 'log', args });
      },
      error: (...args) => {
        console.error('[Page]', ...args);
        this.emit('console', { type: 'error', args });
      },
      warn: (...args) => {
        console.warn('[Page]', ...args);
        this.emit('console', { type: 'warn', args });
      },
      info: (...args) => {
        console.info('[Page]', ...args);
        this.emit('console', { type: 'info', args });
      },
      debug: (...args) => {
        console.debug('[Page]', ...args);
        this.emit('console', { type: 'debug', args });
      },
      trace: (...args) => {
        console.trace('[Page]', ...args);
        this.emit('console', { type: 'trace', args });
      },
      clear: () => {
        console.clear();
        this.emit('console', { type: 'clear', args: [] });
      },
      table: (data) => {
        console.table(data);
        this.emit('console', { type: 'table', args: [data] });
      },
      time: (label) => {
        console.time(label);
      },
      timeEnd: (label) => {
        console.timeEnd(label);
      },
      assert: (condition, ...args) => {
        if (!condition) {
          console.assert(condition, ...args);
          this.emit('console', { type: 'error', args: ['Assertion failed:', ...args] });
        }
      },
      count: (label = 'default') => {
        console.count(label);
      },
      countReset: (label = 'default') => {
        console.countReset(label);
      },
      group: (...args) => {
        console.group(...args);
      },
      groupEnd: () => {
        console.groupEnd();
      },
    };
  }

  createDocumentAPI() {
    const self = this;
    
    return {
      title: this.tab.title || '',
      URL: this.tab.url || '',
      domain: '',
      readyState: 'loading',
      
      getElementById: (id) => {
        // TODO: Query engine's DOM tree
        return null;
      },
      
      getElementsByClassName: (className) => {
        return [];
      },
      
      getElementsByTagName: (tagName) => {
        return [];
      },
      
      querySelector: (selector) => {
        return null;
      },
      
      querySelectorAll: (selector) => {
        return [];
      },
      
      createElement: (tagName) => {
        return {
          tagName: tagName.toUpperCase(),
          attributes: {},
          children: [],
          style: {},
          classList: {
            add: () => {},
            remove: () => {},
            toggle: () => {},
            contains: () => false,
          },
          addEventListener: () => {},
          removeEventListener: () => {},
          setAttribute: function(name, value) {
            this.attributes[name] = value;
          },
          getAttribute: function(name) {
            return this.attributes[name];
          },
          appendChild: function(child) {
            this.children.push(child);
          },
        };
      },
      
      addEventListener: (event, handler, options) => {
        self.emit('dom-event-listener', { event, handler, options });
      },
      
      removeEventListener: (event, handler) => {
        self.emit('dom-event-listener-remove', { event, handler });
      },
      
      cookie: '',
      referrer: '',
      head: {},
      body: {},
    };
  }

  createNavigatorAPI() {
    return {
      userAgent: 'Mozilla/5.0 (X11; Linux x86_64) PronBrow/1.0.0',
      platform: process.platform,
      language: 'en-US',
      languages: ['en-US', 'en'],
      onLine: true,
      cookieEnabled: true,
      doNotTrack: '1',
      hardwareConcurrency: require('os').cpus().length,
      maxTouchPoints: 0,
      vendor: 'PronBrow',
      vendorSub: '',
      product: 'PronBrow',
      productSub: '1.0',
      appName: 'PronBrow',
      appVersion: '1.0.0',
      appCodeName: 'PronBrow',
    };
  }

  createLocationAPI() {
    const url = this.tab.url || '';
    let parsed;
    try {
      parsed = new URL(url);
    } catch (e) {
      parsed = { href: url, protocol: '', host: '', hostname: '', port: '', pathname: '', search: '', hash: '' };
    }

    return {
      href: parsed.href,
      protocol: parsed.protocol,
      host: parsed.host,
      hostname: parsed.hostname,
      port: parsed.port,
      pathname: parsed.pathname,
      search: parsed.search,
      hash: parsed.hash,
      origin: parsed.origin || '',
      reload: () => {
        this.tab.reload();
      },
      replace: (url) => {
        this.tab.navigate(url);
      },
      assign: (url) => {
        this.tab.navigate(url);
      },
    };
  }

  createSetTimeout() {
    return (callback, delay = 0, ...args) => {
      const id = this.nextTimerId++;
      const timer = setTimeout(() => {
        try {
          if (typeof callback === 'function') {
            callback(...args);
          } else if (typeof callback === 'string') {
            this.execute(callback);
          }
        } catch (error) {
          console.error('Timer callback error:', error);
        }
        this.timers.delete(id);
      }, delay);
      
      this.timers.set(id, timer);
      return id;
    };
  }

  createSetInterval() {
    return (callback, delay = 0, ...args) => {
      const id = this.nextTimerId++;
      const interval = setInterval(() => {
        try {
          if (typeof callback === 'function') {
            callback(...args);
          } else if (typeof callback === 'string') {
            this.execute(callback);
          }
        } catch (error) {
          console.error('Interval callback error:', error);
        }
      }, delay);
      
      this.intervals.set(id, interval);
      return id;
    };
  }

  createClearTimeout() {
    return (id) => {
      const timer = this.timers.get(id);
      if (timer) {
        clearTimeout(timer);
        this.timers.delete(id);
      }
    };
  }

  createClearInterval() {
    return (id) => {
      const interval = this.intervals.get(id);
      if (interval) {
        clearInterval(interval);
        this.intervals.delete(id);
      }
    };
  }

  createFetchAPI() {
    return async (url, options = {}) => {
      // Basic fetch implementation
      const http = url.startsWith('https') ? require('https') : require('http');
      
      return new Promise((resolve, reject) => {
        const req = http.request(url, {
          method: options.method || 'GET',
          headers: options.headers || {},
        }, (res) => {
          let data = '';
          res.on('data', chunk => data += chunk);
          res.on('end', () => {
            resolve({
              ok: res.statusCode >= 200 && res.statusCode < 300,
              status: res.statusCode,
              statusText: res.statusMessage,
              headers: res.headers,
              text: async () => data,
              json: async () => JSON.parse(data),
            });
          });
        });
        
        req.on('error', reject);
        
        if (options.body) {
          req.write(options.body);
        }
        
        req.end();
      });
    };
  }

  createXMLHttpRequest() {
    // Simplified XMLHttpRequest implementation
    return class XMLHttpRequest {
      constructor() {
        this.readyState = 0;
        this.status = 0;
        this.statusText = '';
        this.responseText = '';
        this.responseXML = null;
        this.onreadystatechange = null;
      }

      open(method, url, async = true) {
        this.method = method;
        this.url = url;
        this.async = async;
        this.readyState = 1;
        if (this.onreadystatechange) this.onreadystatechange();
      }

      send(data) {
        // TODO: Implement actual HTTP request
        this.readyState = 4;
        this.status = 200;
        this.statusText = 'OK';
        if (this.onreadystatechange) this.onreadystatechange();
      }

      setRequestHeader(name, value) {
        // TODO: Store headers
      }

      abort() {
        // TODO: Abort request
      }
    };
  }

  createStorageAPI(type) {
    const storage = new Map();
    
    return {
      getItem: (key) => storage.get(key) || null,
      setItem: (key, value) => storage.set(key, String(value)),
      removeItem: (key) => storage.delete(key),
      clear: () => storage.clear(),
      get length() { return storage.size; },
      key: (index) => {
        const keys = Array.from(storage.keys());
        return keys[index] || null;
      },
    };
  }

  createPerformanceAPI() {
    return {
      now: () => performance.now(),
      timing: {
        navigationStart: Date.now(),
        loadEventEnd: 0,
      },
      navigation: {
        type: 0,
        redirectCount: 0,
      },
      mark: (name) => {},
      measure: (name, startMark, endMark) => {},
      getEntriesByType: (type) => [],
      getEntriesByName: (name) => [],
    };
  }

  createRequestAnimationFrame() {
    return (callback) => {
      return setTimeout(() => {
        callback(performance.now());
      }, 16); // ~60fps
    };
  }

  createCancelAnimationFrame() {
    return (id) => {
      clearTimeout(id);
    };
  }

  execute(code, filename = 'script.js') {
    try {
      const script = new vm.Script(code, { filename });
      const result = script.runInContext(this.context, {
        timeout: 5000, // 5 second timeout
        displayErrors: true,
      });
      
      this.scripts.push({ code, filename, timestamp: Date.now() });
      return result;
    } catch (error) {
      console.error(`JavaScript execution error in ${filename}:`, error.message);
      this.emit('error', { error, filename });
      throw error;
    }
  }

  evaluateExpression(expression) {
    try {
      return vm.runInContext(expression, this.context, {
        timeout: 1000,
        displayErrors: true,
      });
    } catch (error) {
      throw error;
    }
  }

  destroy() {
    // Clear all timers
    for (const timer of this.timers.values()) {
      clearTimeout(timer);
    }
    this.timers.clear();

    for (const interval of this.intervals.values()) {
      clearInterval(interval);
    }
    this.intervals.clear();

    this.removeAllListeners();
    this.context = null;
    this.scripts = [];
  }
}

module.exports = JavaScriptContext;
