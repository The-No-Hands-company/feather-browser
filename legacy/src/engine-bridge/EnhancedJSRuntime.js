const vm = require('vm');
const EventEmitter = require('events');

/**
 * Enhanced JavaScript Runtime with full ES6+ support
 * Provides comprehensive Web APIs for modern web applications
 */
class EnhancedJSRuntime extends EventEmitter {
  constructor(tab, domBridge, engine) {
    super();
    this.tab = tab;
    this.domBridge = domBridge;
    this.engine = engine;
    this.context = null;
    this.modules = new Map();
    this.workers = new Map();
    this.serviceWorkers = new Map();
    
    this.initializeRuntime();
  }

  initializeRuntime() {
    // Create Event classes first (outside sandbox to avoid circular refs)
    const EventClass = this.createEvent();
    const CustomEventClass = this.createCustomEvent(EventClass);
    const MouseEventClass = this.createMouseEvent(EventClass);
    const KeyboardEventClass = this.createKeyboardEvent(EventClass);
    const TouchEventClass = this.createTouchEvent(EventClass);
    const InputEventClass = this.createInputEvent(EventClass);
    const FocusEventClass = this.createFocusEvent(EventClass);
    const WheelEventClass = this.createWheelEvent(EventClass);
    
    const sandbox = {
      // Core Web APIs
      console: this.createConsoleAPI(),
      window: {},
      document: this.createDocumentAPI(),
      navigator: this.createNavigatorAPI(),
      location: this.createLocationAPI(),
      history: this.createHistoryAPI(),
      screen: this.createScreenAPI(),
      
      // Timers
      setTimeout: this.createSetTimeout(),
      setInterval: this.createSetInterval(),
      clearTimeout: this.createClearTimeout(),
      clearInterval: this.createClearInterval(),
      setImmediate: this.createSetImmediate(),
      clearImmediate: this.createClearImmediate(),
      
      // Animation
      requestAnimationFrame: this.createRequestAnimationFrame(),
      cancelAnimationFrame: this.createCancelAnimationFrame(),
      requestIdleCallback: this.createRequestIdleCallback(),
      cancelIdleCallback: this.createCancelIdleCallback(),
      
      // Network
      fetch: this.createFetchAPI(),
      XMLHttpRequest: this.createXMLHttpRequest(),
      WebSocket: this.createWebSocket(),
      EventSource: this.createEventSource(),
      
      // Storage
      localStorage: this.createStorage('local'),
      sessionStorage: this.createStorage('session'),
      indexedDB: this.createIndexedDB(),
      
      // Workers
      Worker: this.createWorker(),
      SharedWorker: this.createSharedWorker(),
      ServiceWorker: this.createServiceWorker(),
      
      // Media
      Audio: this.createAudio(),
      Video: this.createVideo(),
      MediaRecorder: this.createMediaRecorder(),
      
      // Canvas & Graphics
      CanvasRenderingContext2D: this.createCanvasContext2D(),
      WebGLRenderingContext: this.createWebGLContext(),
      ImageData: this.createImageData(),
      Path2D: this.createPath2D(),
      
      // Crypto
      crypto: this.createCryptoAPI(),
      SubtleCrypto: this.createSubtleCrypto(),
      
      // Performance
      performance: this.createPerformanceAPI(),
      PerformanceObserver: this.createPerformanceObserver(),
      
      // Notifications
      Notification: this.createNotification(),
      
      // Geolocation
      geolocation: this.createGeolocation(),
      
      // Battery
      battery: this.createBattery(),
      
      // Clipboard
      clipboard: this.createClipboard(),
      
      // Web Components
      customElements: this.createCustomElements(),
      ShadowRoot: this.createShadowRoot(),
      
      // Observers
      MutationObserver: this.createMutationObserver(),
      IntersectionObserver: this.createIntersectionObserver(),
      ResizeObserver: this.createResizeObserver(),
      
      // Promises & Async
      Promise: Promise,
      async: true,
      await: true,
      
      // ES6+ Globals
      Array: Array,
      Object: Object,
      String: String,
      Number: Number,
      Boolean: Boolean,
      Symbol: Symbol,
      BigInt: typeof BigInt !== 'undefined' ? BigInt : undefined,
      Date: Date,
      RegExp: RegExp,
      Error: Error,
      TypeError: TypeError,
      ReferenceError: ReferenceError,
      SyntaxError: SyntaxError,
      RangeError: RangeError,
      Math: Math,
      JSON: JSON,
      Map: Map,
      Set: Set,
      WeakMap: WeakMap,
      WeakSet: WeakSet,
      Proxy: Proxy,
      Reflect: Reflect,
      ArrayBuffer: ArrayBuffer,
      Uint8Array: Uint8Array,
      Uint16Array: Uint16Array,
      Uint32Array: Uint32Array,
      Int8Array: Int8Array,
      Int16Array: Int16Array,
      Int32Array: Int32Array,
      Float32Array: Float32Array,
      Float64Array: Float64Array,
      DataView: DataView,
      
      // URL APIs
      URL: require('url').URL,
      URLSearchParams: require('url').URLSearchParams,
      
      // Encoding
      TextEncoder: require('util').TextEncoder,
      TextDecoder: require('util').TextDecoder,
      atob: (str) => Buffer.from(str, 'base64').toString('binary'),
      btoa: (str) => Buffer.from(str, 'binary').toString('base64'),
      
      // Global functions
      parseInt: parseInt,
      parseFloat: parseFloat,
      isNaN: isNaN,
      isFinite: isFinite,
      encodeURI: encodeURI,
      encodeURIComponent: encodeURIComponent,
      decodeURI: decodeURI,
      decodeURIComponent: decodeURIComponent,
      escape: escape,
      unescape: unescape,
      
      // Global values
      NaN: NaN,
      Infinity: Infinity,
      undefined: undefined,
      null: null,
      
      // DOM Events
      Event: EventClass,
      CustomEvent: CustomEventClass,
      MouseEvent: MouseEventClass,
      KeyboardEvent: KeyboardEventClass,
      TouchEvent: TouchEventClass,
      InputEvent: InputEventClass,
      FocusEvent: FocusEventClass,
      WheelEvent: WheelEventClass,
    };
    
    // Self-references
    sandbox.window = sandbox;
    sandbox.self = sandbox;
    sandbox.globalThis = sandbox;
    sandbox.frames = sandbox;
    sandbox.parent = sandbox;
    sandbox.top = sandbox;

    this.context = vm.createContext(sandbox);
    console.log('✓ Enhanced JavaScript runtime initialized');
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
      dir: (obj) => {
        console.dir(obj);
        this.emit('console', { type: 'dir', args: [obj] });
      },
      dirxml: (obj) => {
        console.dirxml(obj);
        this.emit('console', { type: 'dirxml', args: [obj] });
      },
      clear: () => {
        console.clear();
        this.emit('console', { type: 'clear', args: [] });
      },
      table: (data) => {
        console.table(data);
        this.emit('console', { type: 'table', args: [data] });
      },
      time: (label) => console.time(label),
      timeEnd: (label) => console.timeEnd(label),
      timeLog: (label, ...args) => console.timeLog(label, ...args),
      assert: (condition, ...args) => {
        if (!condition) {
          console.assert(condition, ...args);
          this.emit('console', { type: 'error', args: ['Assertion failed:', ...args] });
        }
      },
      count: (label = 'default') => console.count(label),
      countReset: (label = 'default') => console.countReset(label),
      group: (...args) => console.group(...args),
      groupCollapsed: (...args) => console.groupCollapsed(...args),
      groupEnd: () => console.groupEnd(),
      profile: (label) => {},
      profileEnd: (label) => {},
      timeStamp: (label) => {},
    };
  }

  createDocumentAPI() {
    return {
      // Properties
      title: this.tab.title || '',
      URL: this.tab.url || '',
      documentURI: this.tab.url || '',
      domain: '',
      referrer: '',
      cookie: '',
      lastModified: new Date().toISOString(),
      readyState: 'loading',
      characterSet: 'UTF-8',
      charset: 'UTF-8',
      inputEncoding: 'UTF-8',
      contentType: 'text/html',
      doctype: null,
      documentElement: null,
      head: null,
      body: null,
      
      // Methods - will be populated by DOMBridge
      getElementById: (id) => this.domBridge ? this.domBridge.getElementById(id) : null,
      getElementsByClassName: (className) => this.domBridge ? this.domBridge.getElementsByClassName(className) : [],
      getElementsByTagName: (tagName) => this.domBridge ? this.domBridge.getElementsByTagName(tagName) : [],
      getElementsByName: (name) => this.domBridge ? this.domBridge.getElementsByName(name) : [],
      querySelector: (selector) => this.domBridge ? this.domBridge.querySelector(selector) : null,
      querySelectorAll: (selector) => this.domBridge ? this.domBridge.querySelectorAll(selector) : [],
      
      createElement: (tagName) => this.domBridge ? this.domBridge.createElement(tagName) : null,
      createElementNS: (ns, tagName) => this.domBridge ? this.domBridge.createElementNS(ns, tagName) : null,
      createTextNode: (text) => this.domBridge ? this.domBridge.createTextNode(text) : null,
      createComment: (text) => this.domBridge ? this.domBridge.createComment(text) : null,
      createDocumentFragment: () => this.domBridge ? this.domBridge.createDocumentFragment() : null,
      
      createEvent: (type) => this.createEvent(),
      createRange: () => this.createRange(),
      createTreeWalker: (root, filter) => this.createTreeWalker(root, filter),
      createNodeIterator: (root, filter) => this.createNodeIterator(root, filter),
      
      importNode: (node, deep) => node,
      adoptNode: (node) => node,
      
      addEventListener: (event, handler, options) => {
        this.emit('document-event-listener', { event, handler, options });
      },
      removeEventListener: (event, handler) => {
        this.emit('document-event-listener-remove', { event, handler });
      },
      dispatchEvent: (event) => {
        this.emit('document-event', event);
      },
      
      // Selection
      getSelection: () => this.createSelection(),
      
      // Forms
      forms: [],
      images: [],
      links: [],
      scripts: [],
      styleSheets: [],
      
      // Deprecated but still used
      write: (...args) => {
        console.warn('document.write is deprecated');
      },
      writeln: (...args) => {
        console.warn('document.writeln is deprecated');
      },
      open: () => {},
      close: () => {},
    };
  }

  createNavigatorAPI() {
    const os = require('os');
    return {
      userAgent: 'Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) PronBrow/1.0.0 Chrome/120.0.0.0',
      appName: 'PronBrow',
      appVersion: '1.0.0',
      appCodeName: 'Mozilla',
      platform: process.platform,
      product: 'Gecko',
      productSub: '20100101',
      vendor: 'PronBrow',
      vendorSub: '',
      language: 'en-US',
      languages: ['en-US', 'en'],
      onLine: true,
      cookieEnabled: true,
      doNotTrack: '1',
      maxTouchPoints: 0,
      hardwareConcurrency: os.cpus().length,
      deviceMemory: Math.round(os.totalmem() / (1024 ** 3)),
      connection: {
        effectiveType: '4g',
        downlink: 10,
        rtt: 50,
        saveData: false,
      },
      permissions: this.createPermissions(),
      mediaDevices: this.createMediaDevices(),
      serviceWorker: this.createServiceWorkerContainer(),
      storage: this.createStorageManager(),
      getBattery: () => Promise.resolve(this.createBattery()),
      geolocation: this.createGeolocation(),
      clipboard: this.createClipboard(),
      credentials: this.createCredentials(),
      sendBeacon: (url, data) => true,
      vibrate: (pattern) => false,
      share: (data) => Promise.resolve(),
    };
  }

  createHistoryAPI() {
    return {
      length: 1,
      state: null,
      scrollRestoration: 'auto',
      back: () => this.tab.goBack(),
      forward: () => this.tab.goForward(),
      go: (delta) => {
        if (delta < 0) this.tab.goBack();
        else if (delta > 0) this.tab.goForward();
      },
      pushState: (state, title, url) => {
        console.log('pushState:', state, title, url);
        this.emit('history-push', { state, title, url });
      },
      replaceState: (state, title, url) => {
        console.log('replaceState:', state, title, url);
        this.emit('history-replace', { state, title, url });
      },
    };
  }

  createLocationAPI() {
    const url = this.tab.url || '';
    let parsed;
    try {
      parsed = new URL(url);
    } catch (e) {
      parsed = {
        href: url,
        protocol: '',
        host: '',
        hostname: '',
        port: '',
        pathname: '/',
        search: '',
        hash: '',
        origin: '',
      };
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
      origin: parsed.origin,
      ancestorOrigins: [],
      assign: (url) => this.tab.navigate(url),
      replace: (url) => this.tab.navigate(url),
      reload: () => this.tab.reload(),
      toString: () => parsed.href,
    };
  }

  createScreenAPI() {
    return {
      width: 1920,
      height: 1080,
      availWidth: 1920,
      availHeight: 1040,
      colorDepth: 24,
      pixelDepth: 24,
      orientation: {
        type: 'landscape-primary',
        angle: 0,
      },
    };
  }

  createSetTimeout() {
    const timers = new Map();
    let nextId = 1;

    return (callback, delay = 0, ...args) => {
      const id = nextId++;
      const timer = setTimeout(() => {
        try {
          if (typeof callback === 'function') {
            callback(...args);
          } else if (typeof callback === 'string') {
            this.execute(callback);
          }
        } catch (error) {
          console.error('Timer callback error:', error);
          this.emit('error', error);
        }
        timers.delete(id);
      }, delay);
      
      timers.set(id, timer);
      return id;
    };
  }

  createSetInterval() {
    const intervals = new Map();
    let nextId = 1;

    return (callback, delay = 0, ...args) => {
      const id = nextId++;
      const interval = setInterval(() => {
        try {
          if (typeof callback === 'function') {
            callback(...args);
          } else if (typeof callback === 'string') {
            this.execute(callback);
          }
        } catch (error) {
          console.error('Interval callback error:', error);
          this.emit('error', error);
        }
      }, delay);
      
      intervals.set(id, interval);
      return id;
    };
  }

  createClearTimeout() {
    return (id) => clearTimeout(id);
  }

  createClearInterval() {
    return (id) => clearInterval(id);
  }

  createSetImmediate() {
    return (callback, ...args) => {
      return setImmediate(() => {
        try {
          callback(...args);
        } catch (error) {
          console.error('Immediate callback error:', error);
        }
      });
    };
  }

  createClearImmediate() {
    return (id) => clearImmediate(id);
  }

  createRequestAnimationFrame() {
    return (callback) => {
      return setTimeout(() => {
        callback(performance.now());
      }, 16); // ~60fps
    };
  }

  createCancelAnimationFrame() {
    return (id) => clearTimeout(id);
  }

  createRequestIdleCallback() {
    return (callback, options = {}) => {
      const start = Date.now();
      return setTimeout(() => {
        callback({
          didTimeout: false,
          timeRemaining: () => Math.max(0, 50 - (Date.now() - start)),
        });
      }, 1);
    };
  }

  createCancelIdleCallback() {
    return (id) => clearTimeout(id);
  }

  createFetchAPI() {
    return async (url, options = {}) => {
      // Use Electron's net module for proper browser-like fetch
      const { net } = require('electron');
      
      return new Promise((resolve, reject) => {
        const request = net.request({
          method: options.method || 'GET',
          url: url,
        });

        if (options.headers) {
          for (const [key, value] of Object.entries(options.headers)) {
            request.setHeader(key, value);
          }
        }

        request.on('response', (response) => {
          const chunks = [];
          response.on('data', (chunk) => chunks.push(chunk));
          response.on('end', () => {
            const buffer = Buffer.concat(chunks);
            const text = buffer.toString('utf8');
            
            resolve({
              ok: response.statusCode >= 200 && response.statusCode < 300,
              status: response.statusCode,
              statusText: response.statusMessage,
              headers: new Map(Object.entries(response.headers)),
              url: url,
              type: 'basic',
              redirected: false,
              text: async () => text,
              json: async () => JSON.parse(text),
              blob: async () => buffer,
              arrayBuffer: async () => buffer.buffer,
              formData: async () => null,
              clone: () => this,
            });
          });
        });

        request.on('error', reject);

        if (options.body) {
          request.write(options.body);
        }

        request.end();
      });
    };
  }

  createXMLHttpRequest() {
    const self = this;
    
    return class XMLHttpRequest {
      constructor() {
        this.readyState = 0;
        this.status = 0;
        this.statusText = '';
        this.responseText = '';
        this.responseXML = null;
        this.response = null;
        this.responseType = '';
        this.responseURL = '';
        this.timeout = 0;
        this.withCredentials = false;
        this.upload = {};
        
        this.onreadystatechange = null;
        this.onload = null;
        this.onerror = null;
        this.onprogress = null;
        this.onabort = null;
        this.ontimeout = null;
        this.onloadstart = null;
        this.onloadend = null;
        
        this._headers = {};
        this._method = null;
        this._url = null;
      }

      open(method, url, async = true, user, password) {
        this._method = method;
        this._url = url;
        this._async = async;
        this.readyState = 1;
        if (this.onreadystatechange) this.onreadystatechange();
      }

      send(data) {
        if (this.onloadstart) this.onloadstart();
        
        // Use fetch internally
        const options = {
          method: this._method,
          headers: this._headers,
          body: data,
        };

        self.context.fetch(this._url, options)
          .then(async (response) => {
            this.status = response.status;
            this.statusText = response.statusText;
            this.responseURL = response.url;
            this.responseText = await response.text();
            this.response = this.responseText;
            this.readyState = 4;
            
            if (this.onreadystatechange) this.onreadystatechange();
            if (this.onload) this.onload();
            if (this.onloadend) this.onloadend();
          })
          .catch((error) => {
            if (this.onerror) this.onerror(error);
            if (this.onloadend) this.onloadend();
          });
      }

      setRequestHeader(name, value) {
        this._headers[name] = value;
      }

      getResponseHeader(name) {
        return null; // TODO: Implement
      }

      getAllResponseHeaders() {
        return ''; // TODO: Implement
      }

      abort() {
        this.readyState = 0;
        if (this.onabort) this.onabort();
      }

      overrideMimeType(mime) {
        // TODO: Implement
      }
    };
  }

  // Stub implementations for additional APIs
  createWebSocket() {
    return class WebSocket {
      constructor(url, protocols) {
        console.log('WebSocket created:', url);
      }
    };
  }

  createEventSource() {
    return class EventSource {
      constructor(url) {
        console.log('EventSource created:', url);
      }
    };
  }

  createStorage(type) {
    const storage = new Map();
    return {
      getItem: (key) => storage.get(key) || null,
      setItem: (key, value) => storage.set(key, String(value)),
      removeItem: (key) => storage.delete(key),
      clear: () => storage.clear(),
      key: (index) => Array.from(storage.keys())[index] || null,
      get length() { return storage.size; },
    };
  }

  createIndexedDB() {
    return {
      open: (name, version) => {
        console.log('IndexedDB open:', name, version);
        return Promise.resolve({});
      },
      deleteDatabase: (name) => Promise.resolve(),
      cmp: (a, b) => 0,
    };
  }

  createWorker() {
    return class Worker {
      constructor(url) {
        console.log('Worker created:', url);
      }
    };
  }

  createSharedWorker() {
    return class SharedWorker {
      constructor(url) {
        console.log('SharedWorker created:', url);
      }
    };
  }

  createServiceWorker() {
    return class ServiceWorker {
      constructor() {
        console.log('ServiceWorker created');
      }
    };
  }

  createAudio() {
    return class Audio {
      constructor(src) {
        this.src = src;
        console.log('Audio created:', src);
      }
    };
  }

  createVideo() {
    return class Video {
      constructor() {
        console.log('Video created');
      }
    };
  }

  createMediaRecorder() {
    return class MediaRecorder {
      constructor(stream) {
        console.log('MediaRecorder created');
      }
    };
  }

  createCanvasContext2D() {
    return class CanvasRenderingContext2D {};
  }

  createWebGLContext() {
    return class WebGLRenderingContext {};
  }

  createImageData() {
    return class ImageData {};
  }

  createPath2D() {
    return class Path2D {};
  }

  createCryptoAPI() {
    const crypto = require('crypto');
    return {
      getRandomValues: (array) => {
        crypto.randomFillSync(array);
        return array;
      },
      randomUUID: () => crypto.randomUUID(),
      subtle: this.createSubtleCrypto(),
    };
  }

  createSubtleCrypto() {
    return {};
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
      clearMarks: () => {},
      clearMeasures: () => {},
    };
  }

  createPerformanceObserver() {
    return class PerformanceObserver {};
  }

  createNotification() {
    return class Notification {};
  }

  createGeolocation() {
    return {
      getCurrentPosition: (success, error) => {
        if (error) error({ code: 1, message: 'Not implemented' });
      },
      watchPosition: (success, error) => 0,
      clearWatch: (id) => {},
    };
  }

  createBattery() {
    return {
      charging: true,
      chargingTime: 0,
      dischargingTime: Infinity,
      level: 1.0,
    };
  }

  createClipboard() {
    return {
      readText: () => Promise.resolve(''),
      writeText: (text) => Promise.resolve(),
      read: () => Promise.resolve([]),
      write: (data) => Promise.resolve(),
    };
  }

  createCustomElements() {
    return {
      define: (name, constructor, options) => {
        console.log('Custom element defined:', name);
      },
      get: (name) => null,
      upgrade: (root) => {},
      whenDefined: (name) => Promise.resolve(),
    };
  }

  createShadowRoot() {
    return class ShadowRoot {};
  }

  createMutationObserver() {
    return class MutationObserver {
      constructor(callback) {
        this.callback = callback;
      }
      observe(target, options) {}
      disconnect() {}
      takeRecords() { return []; }
    };
  }

  createIntersectionObserver() {
    return class IntersectionObserver {
      constructor(callback, options) {
        this.callback = callback;
      }
      observe(target) {}
      unobserve(target) {}
      disconnect() {}
      takeRecords() { return []; }
    };
  }

  createResizeObserver() {
    return class ResizeObserver {
      constructor(callback) {
        this.callback = callback;
      }
      observe(target, options) {}
      unobserve(target) {}
      disconnect() {}
    };
  }

  createEvent() {
    return class Event {
      constructor(type, options = {}) {
        this.type = type;
        this.bubbles = options.bubbles || false;
        this.cancelable = options.cancelable || false;
        this.composed = options.composed || false;
      }
    };
  }

  createCustomEvent(EventBase) {
    return class CustomEvent extends EventBase {
      constructor(type, options = {}) {
        super(type, options);
        this.detail = options.detail;
      }
    };
  }

  createMouseEvent(EventBase) {
    return class MouseEvent extends EventBase {
      constructor(type, options = {}) {
        super(type, options);
        this.clientX = options.clientX || 0;
        this.clientY = options.clientY || 0;
        this.screenX = options.screenX || 0;
        this.screenY = options.screenY || 0;
        this.button = options.button || 0;
        this.buttons = options.buttons || 0;
      }
    };
  }

  createKeyboardEvent(EventBase) {
    return class KeyboardEvent extends EventBase {
      constructor(type, options = {}) {
        super(type, options);
        this.key = options.key || '';
        this.code = options.code || '';
        this.ctrlKey = options.ctrlKey || false;
        this.shiftKey = options.shiftKey || false;
        this.altKey = options.altKey || false;
        this.metaKey = options.metaKey || false;
      }
    };
  }

  createTouchEvent(EventBase) {
    return class TouchEvent extends EventBase {
      constructor(type, options = {}) {
        super(type, options);
        this.touches = options.touches || [];
        this.targetTouches = options.targetTouches || [];
        this.changedTouches = options.changedTouches || [];
      }
    };
  }

  createWheelEvent(EventBase) {
    return class WheelEvent extends EventBase {
      constructor(type, options = {}) {
        super(type, options);
        this.deltaX = options.deltaX || 0;
        this.deltaY = options.deltaY || 0;
        this.deltaZ = options.deltaZ || 0;
        this.deltaMode = options.deltaMode || 0;
        this.clientX = options.clientX || 0;
        this.clientY = options.clientY || 0;
        this.screenX = options.screenX || 0;
        this.screenY = options.screenY || 0;
      }
    };
  }

  createInputEvent(EventBase) {
    return class InputEvent extends EventBase {
      constructor(type, options = {}) {
        super(type, options);
        this.data = options.data || null;
        this.inputType = options.inputType || '';
      }
    };
  }

  createFocusEvent(EventBase) {
    return class FocusEvent extends EventBase {
      constructor(type, options = {}) {
        super(type, options);
        this.relatedTarget = options.relatedTarget || null;
      }
    };
  }

  createPermissions() {
    return {
      query: (descriptor) => Promise.resolve({ state: 'granted' }),
    };
  }

  createMediaDevices() {
    return {
      getUserMedia: (constraints) => Promise.reject(new Error('Not implemented')),
      getDisplayMedia: (constraints) => Promise.reject(new Error('Not implemented')),
      enumerateDevices: () => Promise.resolve([]),
    };
  }

  createServiceWorkerContainer() {
    return {
      register: (url) => Promise.resolve({}),
      getRegistration: (url) => Promise.resolve(null),
      getRegistrations: () => Promise.resolve([]),
    };
  }

  createStorageManager() {
    return {
      estimate: () => Promise.resolve({ usage: 0, quota: 1000000000 }),
      persist: () => Promise.resolve(false),
      persisted: () => Promise.resolve(false),
    };
  }

  createCredentials() {
    return {
      get: (options) => Promise.resolve(null),
      create: (options) => Promise.resolve(null),
      store: (credential) => Promise.resolve(),
      preventSilentAccess: () => Promise.resolve(),
    };
  }

  createRange() {
    return {};
  }

  createSelection() {
    return {
      rangeCount: 0,
      getRangeAt: (index) => this.createRange(),
      addRange: (range) => {},
      removeRange: (range) => {},
      removeAllRanges: () => {},
      toString: () => '',
    };
  }

  createTreeWalker(root, filter) {
    return {};
  }

  createNodeIterator(root, filter) {
    return {};
  }

  /**
   * Execute JavaScript code in the runtime
   */
  execute(code, filename = 'script.js') {
    try {
      const script = new vm.Script(code, {
        filename,
        lineOffset: 0,
        columnOffset: 0,
        displayErrors: true,
      });

      const result = script.runInContext(this.context, {
        timeout: 10000, // 10 second timeout
        displayErrors: true,
      });

      this.emit('script-executed', { code, filename, result });
      return result;
    } catch (error) {
      console.error(`JavaScript execution error in ${filename}:`, error.message);
      this.emit('script-error', { error, filename, code });
      throw error;
    }
  }

  /**
   * Evaluate expression and return result
   */
  evaluate(expression) {
    try {
      return vm.runInContext(expression, this.context, {
        timeout: 2000,
        displayErrors: true,
      });
    } catch (error) {
      throw error;
    }
  }

  /**
   * Import ES6 module
   */
  async importModule(url) {
    console.log('Importing module:', url);
    // TODO: Implement module loading
    return {};
  }

  /**
   * Clean up runtime
   */
  destroy() {
    this.modules.clear();
    this.workers.clear();
    this.serviceWorkers.clear();
    this.removeAllListeners();
    this.context = null;
  }
}

module.exports = EnhancedJSRuntime;
