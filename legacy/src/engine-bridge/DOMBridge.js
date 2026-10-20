const EventEmitter = require('events');

/**
 * DOM Bridge - Connects JavaScript DOM API with PronBrow Engine's DOM tree
 * Provides two-way communication between JS and native engine
 */
class DOMBridge extends EventEmitter {
  constructor(engine, jsContext) {
    super();
    this.engine = engine;
    this.jsContext = jsContext;
    this.nativeEngine = null; // Will be set by EngineIntegration
    this.domTree = null;
    this.elementMap = new Map(); // Maps element IDs to native elements
    this.nextElementId = 1;
    this.eventListeners = new Map();
  }

  /**
   * Parse HTML and build DOM tree in engine
   */
  parseHTML(html) {
    try {
      // Try to use native engine first
      if (this.nativeEngine && this.nativeEngine.parseHtml) {
        try {
          const nativeDOM = this.nativeEngine.parseHtml(html);
          this.domTree = nativeDOM;
          this.updateDocumentAPI();
          console.log('✓ DOM parsed with native engine');
          return this.domTree;
        } catch (err) {
          console.warn('Native parser failed, using fallback:', err.message);
        }
      }
      
      // Fallback: Build DOM tree from HTML string
      this.domTree = this.buildDOMTree(html);
      
      // Update document object in JS context
      this.updateDocumentAPI();
      
      console.log('✓ DOM tree parsed (fallback mode)');
      return this.domTree;
    } catch (error) {
      console.error('DOM parsing error:', error);
      return null;
    }
  }

  /**
   * Build internal DOM representation from HTML
   */
  buildDOMTree(html) {
    const tree = {
      type: 'document',
      children: [],
      elements: new Map(),
    };

    // Parse the HTML using a simple recursive parser
    const parsed = this.parseHTMLRecursive(html);
    if (parsed && parsed.length > 0) {
      tree.children = parsed;
      
      // Build elements map for getElementById
      const buildElementsMap = (node) => {
        if (node.attributes && node.attributes.id) {
          tree.elements.set(node.attributes.id, node);
        }
        if (node.children) {
          node.children.forEach(buildElementsMap);
        }
      };
      
      parsed.forEach(buildElementsMap);
    }

    return tree;
  }

  /**
   * Recursive HTML parser that handles nested structures
   */
  parseHTMLRecursive(html) {
    const elements = [];
    let pos = 0;

    while (pos < html.length) {
      // Find next tag
      const tagStart = html.indexOf('<', pos);
      if (tagStart === -1) break;

      // Handle text content before tag
      if (tagStart > pos) {
        const text = html.substring(pos, tagStart).trim();
        if (text) {
          // Text nodes are not included in this simple implementation
        }
      }

      // Check if it's a closing tag
      if (html[tagStart + 1] === '/') {
        // This is a closing tag, stop parsing at this level
        break;
      }

      // Check if it's a comment or doctype
      if (html.substr(tagStart, 4) === '<!--') {
        const commentEnd = html.indexOf('-->', tagStart);
        if (commentEnd !== -1) {
          pos = commentEnd + 3;
          continue;
        }
      }

      if (html.substr(tagStart, 9).toLowerCase() === '<!doctype') {
        const doctypeEnd = html.indexOf('>', tagStart);
        if (doctypeEnd !== -1) {
          pos = doctypeEnd + 1;
          continue;
        }
      }

      // Parse opening tag
      const tagEnd = html.indexOf('>', tagStart);
      if (tagEnd === -1) break;

      const tagContent = html.substring(tagStart + 1, tagEnd);
      const parts = tagContent.trim().split(/\s+/);
      const tagName = parts[0].toLowerCase();

      // Self-closing tag
      if (tagContent.endsWith('/')) {
        const attrs = this.parseAttributes(tagContent.substring(tagName.length, tagContent.length - 1));
        const element = this.createElement(tagName, attrs, '');
        elements.push(element);
        pos = tagEnd + 1;
        continue;
      }

      // Void elements (self-closing by nature)
      const voidElements = ['area', 'base', 'br', 'col', 'embed', 'hr', 'img', 'input', 'link', 'meta', 'param', 'source', 'track', 'wbr'];
      if (voidElements.includes(tagName)) {
        const attrs = this.parseAttributes(tagContent.substring(tagName.length));
        const element = this.createElement(tagName, attrs, '');
        elements.push(element);
        pos = tagEnd + 1;
        continue;
      }

      // Find matching closing tag
      const closingTag = `</${tagName}>`;
      let closingPos = html.indexOf(closingTag, tagEnd + 1);
      
      // Handle nested tags with same name
      let depth = 1;
      let searchPos = tagEnd + 1;
      while (depth > 0 && searchPos < html.length) {
        const nextOpen = html.indexOf(`<${tagName}`, searchPos);
        const nextClose = html.indexOf(closingTag, searchPos);
        
        if (nextClose === -1) {
          closingPos = -1;
          break;
        }
        
        if (nextOpen !== -1 && nextOpen < nextClose) {
          // Another opening tag found before closing
          depth++;
          searchPos = nextOpen + tagName.length + 1;
        } else {
          depth--;
          if (depth === 0) {
            closingPos = nextClose;
          } else {
            searchPos = nextClose + closingTag.length;
          }
        }
      }

      if (closingPos === -1) {
        // No closing tag found, treat as self-closing
        const attrs = this.parseAttributes(tagContent.substring(tagName.length));
        const element = this.createElement(tagName, attrs, '');
        elements.push(element);
        pos = tagEnd + 1;
        continue;
      }

      // Parse content between opening and closing tags
      const innerContent = html.substring(tagEnd + 1, closingPos);
      const attrs = this.parseAttributes(tagContent.substring(tagName.length));
      const element = this.createElement(tagName, attrs, '');

      // Recursively parse children
      const children = this.parseHTMLRecursive(innerContent);
      element.children = children;
      children.forEach(child => child.parentNode = element);

      elements.push(element);
      pos = closingPos + closingTag.length;
    }

    return elements;
  }

  /**
   * Parse HTML attributes
   */
  parseAttributes(attrString) {
    const attrs = {};
    const attrRegex = /(\w+)(?:="([^"]*)")?/g;
    let match;

    while ((match = attrRegex.exec(attrString)) !== null) {
      attrs[match[1]] = match[2] || true;
    }

    return attrs;
  }

  /**
   * Create element object
   */
  createElement(tagName, attributes = {}, content = '') {
    const id = this.nextElementId++;
    
    const element = {
      id,
      tagName: tagName.toLowerCase(),
      attributes,
      textContent: content,
      children: [],
      parentNode: null,
      style: {},
      classList: this.createClassList(attributes.class || ''),
      
      // Layout properties
      clientWidth: 800,
      clientHeight: 600,
      offsetWidth: 800,
      offsetHeight: 600,
      scrollWidth: 800,
      scrollHeight: 600,
      offsetLeft: 0,
      offsetTop: 0,
      scrollLeft: 0,
      scrollTop: 0,
      
      // Event handler properties
      onclick: null,
      onload: null,
      onerror: null,
      onsubmit: null,
      
      // Methods
      getAttribute: (name) => element.attributes[name],
      setAttribute: (name, value) => {
        element.attributes[name] = value;
        this.notifyElementChanged(id);
      },
      removeAttribute: (name) => {
        delete element.attributes[name];
        this.notifyElementChanged(id);
      },
      appendChild: (child) => {
        child.parentNode = element;
        element.children.push(child);
        this.notifyElementChanged(id);
      },
      removeChild: (child) => {
        const index = element.children.indexOf(child);
        if (index !== -1) {
          element.children.splice(index, 1);
          child.parentNode = null;
          this.notifyElementChanged(id);
        }
      },
      addEventListener: (event, handler, options) => {
        this.addEventListener(id, event, handler, options);
      },
      removeEventListener: (event, handler) => {
        this.removeEventListener(id, event, handler);
      },
      dispatchEvent: (event) => {
        this.dispatchEvent(id, event);
      },
      querySelector: (selector) => {
        return this.querySelector(element, selector);
      },
      querySelectorAll: (selector) => {
        return this.querySelectorAll(element, selector);
      },
    };

    this.elementMap.set(id, element);
    return element;
  }

  /**
   * Create classList object
   */
  createClassList(classString) {
    const classes = new Set(classString.split(/\s+/).filter(c => c));
    
    return {
      add: (...tokens) => {
        tokens.forEach(t => classes.add(t));
      },
      remove: (...tokens) => {
        tokens.forEach(t => classes.delete(t));
      },
      toggle: (token, force) => {
        if (force === undefined) {
          classes.has(token) ? classes.delete(token) : classes.add(token);
        } else {
          force ? classes.add(token) : classes.delete(token);
        }
        return classes.has(token);
      },
      contains: (token) => classes.has(token),
      replace: (oldToken, newToken) => {
        if (classes.has(oldToken)) {
          classes.delete(oldToken);
          classes.add(newToken);
          return true;
        }
        return false;
      },
      toString: () => Array.from(classes).join(' '),
      get length() { return classes.size; },
      item: (index) => Array.from(classes)[index] || null,
    };
  }

  /**
   * Update document API with parsed DOM
   */
  updateDocumentAPI() {
    if (!this.jsContext || !this.domTree) return;

    const document = this.jsContext.context.document;
    
    // Update document methods to query actual DOM
    document.getElementById = (id) => {
      return this.domTree.elements.get(id) || null;
    };

    document.getElementsByTagName = (tagName) => {
      return this.getElementsByTagName(this.domTree, tagName.toLowerCase());
    };

    document.getElementsByClassName = (className) => {
      return this.getElementsByClassName(this.domTree, className);
    };

    document.querySelector = (selector) => {
      return this.querySelector(this.domTree, selector);
    };

    document.querySelectorAll = (selector) => {
      return this.querySelectorAll(this.domTree, selector);
    };

    // Update document properties
    const titleElement = this.querySelector(this.domTree, 'title');
    if (titleElement) {
      document.title = titleElement.textContent;
    }

    // Set documentElement (html) and body
    const htmlElement = this.querySelector(this.domTree, 'html');
    if (htmlElement) {
      document.documentElement = htmlElement;
      
      // Find head and body
      const headElement = this.querySelector(htmlElement, 'head');
      if (headElement) {
        document.head = headElement;
      }
      
      const bodyElement = this.querySelector(htmlElement, 'body');
      if (bodyElement) {
        document.body = bodyElement;
      }
    }
  }

  /**
   * Query methods
   */
  getElementsByTagName(root, tagName) {
    const results = [];
    
    const traverse = (node) => {
      if (node.tagName === tagName) {
        results.push(node);
      }
      if (node.children) {
        node.children.forEach(traverse);
      }
    };

    if (root.children) {
      root.children.forEach(traverse);
    }

    return results;
  }

  getElementsByClassName(root, className) {
    const results = [];
    
    const traverse = (node) => {
      if (node.classList && node.classList.contains(className)) {
        results.push(node);
      }
      if (node.children) {
        node.children.forEach(traverse);
      }
    };

    if (root.children) {
      root.children.forEach(traverse);
    }

    return results;
  }

  querySelector(root, selector) {
    // Simple selector support (id, class, tag)
    if (selector.startsWith('#')) {
      const id = selector.substring(1);
      return this.domTree ? this.domTree.elements.get(id) || null : null;
    } else if (selector.startsWith('.')) {
      const className = selector.substring(1);
      return this.getElementsByClassName(root, className)[0] || null;
    } else {
      return this.getElementsByTagName(root, selector)[0] || null;
    }
  }

  querySelectorAll(root, selector) {
    if (selector.startsWith('#')) {
      const element = this.querySelector(root, selector);
      return element ? [element] : [];
    } else if (selector.startsWith('.')) {
      const className = selector.substring(1);
      return this.getElementsByClassName(root, className);
    } else {
      return this.getElementsByTagName(root, selector);
    }
  }

  /**
   * Event handling
   */
  addEventListener(elementId, event, handler, options) {
    if (!this.eventListeners.has(elementId)) {
      this.eventListeners.set(elementId, new Map());
    }
    
    const elementListeners = this.eventListeners.get(elementId);
    if (!elementListeners.has(event)) {
      elementListeners.set(event, []);
    }
    
    elementListeners.get(event).push({ handler, options });
  }

  removeEventListener(elementId, event, handler) {
    const elementListeners = this.eventListeners.get(elementId);
    if (!elementListeners) return;
    
    const listeners = elementListeners.get(event);
    if (!listeners) return;
    
    const index = listeners.findIndex(l => l.handler === handler);
    if (index !== -1) {
      listeners.splice(index, 1);
    }
  }

  dispatchEvent(elementId, event) {
    const elementListeners = this.eventListeners.get(elementId);
    if (!elementListeners) return;
    
    const eventType = typeof event === 'string' ? event : event.type;
    const listeners = elementListeners.get(eventType);
    if (!listeners) return;
    
    listeners.forEach(({ handler, options }) => {
      try {
        handler(event);
        if (options && options.once) {
          this.removeEventListener(elementId, eventType, handler);
        }
      } catch (error) {
        console.error('Event handler error:', error);
      }
    });
  }

  /**
   * Notify engine of DOM changes
   */
  notifyElementChanged(elementId) {
    const element = this.elementMap.get(elementId);
    if (!element) return;
    
    // TODO: Update engine's DOM tree
    console.log('Element changed:', elementId);
  }

  /**
   * Extract and execute inline scripts
   */
  extractScripts(html) {
    const scripts = [];
    const scriptRegex = /<script([^>]*)>(.*?)<\/script>/gs;
    let match;

    while ((match = scriptRegex.exec(html)) !== null) {
      const attrs = this.parseAttributes(match[1]);
      const code = match[2];
      
      scripts.push({
        src: attrs.src,
        type: attrs.type || 'text/javascript',
        code,
        async: 'async' in attrs,
        defer: 'defer' in attrs,
      });
    }

    return scripts;
  }

  /**
   * Execute page scripts
   */
  async executeScripts(html) {
    const scripts = this.extractScripts(html);
    
    for (const script of scripts) {
      if (script.src) {
        // TODO: Fetch and execute external script
        console.log('External script:', script.src);
      } else if (script.code && script.type === 'text/javascript') {
        try {
          this.jsContext.execute(script.code, 'inline-script');
        } catch (error) {
          console.error('Script execution error:', error);
        }
      }
    }
  }

  destroy() {
    this.elementMap.clear();
    this.eventListeners.clear();
    this.domTree = null;
  }
}

module.exports = DOMBridge;
