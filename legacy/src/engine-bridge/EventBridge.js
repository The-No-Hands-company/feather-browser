/**
 * EventBridge - Bridges user events from Electron canvas to engine DOM
 * Handles mouse clicks, keyboard input, and user interactions
 */

class EventBridge {
  constructor(engine, canvas) {
    if (!engine) {
      throw new Error('PronBrow engine is required');
    }
    if (!canvas) {
      throw new Error('Canvas element is required');
    }

    this.engine = engine;
    this.canvas = canvas;
    this.isEnabled = false;
    this.currentUrl = null;
    
    // Event listeners storage
    this.listeners = {
      mouse: [],
      keyboard: [],
      focus: []
    };

    // Current DOM state
    this.domElements = [];
    this.focusedElement = null;
    this.hoveredElement = null;

    // Bind methods
    this.handleMouseDown = this.handleMouseDown.bind(this);
    this.handleMouseMove = this.handleMouseMove.bind(this);
    this.handleMouseUp = this.handleMouseUp.bind(this);
    this.handleClick = this.handleClick.bind(this);
    this.handleKeyDown = this.handleKeyDown.bind(this);
    this.handleKeyUp = this.handleKeyUp.bind(this);
    this.handleKeyPress = this.handleKeyPress.bind(this);
  }

  /**
   * Enable event capture
   */
  enable() {
    if (this.isEnabled) return;
    
    console.log('🎮 EventBridge: Enabling event capture');
    
    // Mouse events
    this.canvas.addEventListener('mousedown', this.handleMouseDown);
    this.canvas.addEventListener('mousemove', this.handleMouseMove);
    this.canvas.addEventListener('mouseup', this.handleMouseUp);
    this.canvas.addEventListener('click', this.handleClick);
    
    // Keyboard events
    this.canvas.addEventListener('keydown', this.handleKeyDown);
    this.canvas.addEventListener('keyup', this.handleKeyUp);
    this.canvas.addEventListener('keypress', this.handleKeyPress);
    
    // Make canvas focusable for keyboard events
    this.canvas.tabIndex = 0;
    this.canvas.focus();
    
    this.isEnabled = true;
  }

  /**
   * Disable event capture
   */
  disable() {
    if (!this.isEnabled) return;
    
    console.log('🎮 EventBridge: Disabling event capture');
    
    this.canvas.removeEventListener('mousedown', this.handleMouseDown);
    this.canvas.removeEventListener('mousemove', this.handleMouseMove);
    this.canvas.removeEventListener('mouseup', this.handleMouseUp);
    this.canvas.removeEventListener('click', this.handleClick);
    this.canvas.removeEventListener('keydown', this.handleKeyDown);
    this.canvas.removeEventListener('keyup', this.handleKeyUp);
    this.canvas.removeEventListener('keypress', this.handleKeyPress);
    
    this.isEnabled = false;
  }

  /**
   * Update DOM element positions for hit testing
   * Call this after each page render
   */
  updateDOMLayout(elements) {
    this.domElements = elements || [];
    console.log(`🎮 EventBridge: Updated layout with ${this.domElements.length} elements`);
  }

  /**
   * Handle mouse down event
   */
  handleMouseDown(e) {
    const coords = this.getCanvasCoordinates(e);
    const element = this.hitTest(coords.x, coords.y);
    
    if (element) {
      console.log(`🖱️ Mouse down on element: ${element.tag} at (${coords.x}, ${coords.y})`);
      this.notifyEngine('mousedown', element, coords);
    }
  }

  /**
   * Handle mouse move event
   */
  handleMouseMove(e) {
    const coords = this.getCanvasCoordinates(e);
    const element = this.hitTest(coords.x, coords.y);
    
    // Update hover state
    if (element !== this.hoveredElement) {
      if (this.hoveredElement) {
        this.notifyEngine('mouseout', this.hoveredElement, coords);
      }
      if (element) {
        this.notifyEngine('mouseover', element, coords);
      }
      this.hoveredElement = element;
    }

    // Update cursor style
    if (element && element.cursor) {
      this.canvas.style.cursor = element.cursor;
    } else if (element && (element.tag === 'a' || element.tag === 'button')) {
      this.canvas.style.cursor = 'pointer';
    } else if (element && (element.tag === 'input' || element.tag === 'textarea')) {
      this.canvas.style.cursor = 'text';
    } else {
      this.canvas.style.cursor = 'default';
    }
  }

  /**
   * Handle mouse up event
   */
  handleMouseUp(e) {
    const coords = this.getCanvasCoordinates(e);
    const element = this.hitTest(coords.x, coords.y);
    
    if (element) {
      console.log(`🖱️ Mouse up on element: ${element.tag}`);
      this.notifyEngine('mouseup', element, coords);
    }
  }

  /**
   * Handle click event - CRITICAL FOR NAVIGATION
   */
  handleClick(e) {
    const coords = this.getCanvasCoordinates(e);
    const element = this.hitTest(coords.x, coords.y);
    
    if (!element) {
      console.log('🖱️ Click on empty area');
      return;
    }

    console.log(`🖱️ Click on ${element.tag}`, element);
    
    // Handle link navigation
    if (element.tag === 'a' && element.href) {
      e.preventDefault();
      console.log(`🔗 Navigating to: ${element.href}`);
      this.navigateToURL(element.href);
      return;
    }

    // Handle button clicks
    if (element.tag === 'button' || element.type === 'submit') {
      console.log('🔘 Button clicked');
      this.notifyEngine('click', element, coords);
      
      // Handle form submission
      if (element.form) {
        this.submitForm(element.form);
      }
      return;
    }

    // Handle input focus
    if (element.tag === 'input' || element.tag === 'textarea') {
      console.log('📝 Input element focused');
      this.focusElement(element);
      return;
    }

    // Generic click event
    this.notifyEngine('click', element, coords);
  }

  /**
   * Handle keyboard key down
   */
  handleKeyDown(e) {
    if (!this.focusedElement) {
      // Handle global keyboard shortcuts
      this.handleGlobalKeyboard(e);
      return;
    }

    // Handle input in focused element
    if (this.focusedElement.tag === 'input' || 
        this.focusedElement.tag === 'textarea') {
      this.handleTextInput(e);
    }
  }

  /**
   * Handle keyboard key up
   */
  handleKeyUp(e) {
    if (this.focusedElement) {
      this.notifyEngine('keyup', this.focusedElement, {
        key: e.key,
        code: e.code,
        ctrlKey: e.ctrlKey,
        shiftKey: e.shiftKey,
        altKey: e.altKey
      });
    }
  }

  /**
   * Handle keyboard key press
   */
  handleKeyPress(e) {
    if (this.focusedElement) {
      this.notifyEngine('keypress', this.focusedElement, {
        key: e.key,
        charCode: e.charCode
      });
    }
  }

  /**
   * Handle text input in focused element
   */
  handleTextInput(e) {
    if (!this.focusedElement) return;

    const element = this.focusedElement;
    
    // Get current value
    const currentValue = element.value || '';
    let newValue = currentValue;

    // Handle special keys
    if (e.key === 'Backspace') {
      newValue = currentValue.slice(0, -1);
      e.preventDefault();
    } else if (e.key === 'Enter') {
      // Submit form if this is a submit input
      if (element.type === 'submit' || element.form) {
        this.submitForm(element.form);
        e.preventDefault();
      }
    } else if (e.key === 'Tab') {
      // Focus next element
      this.focusNext();
      e.preventDefault();
    } else if (e.key.length === 1) {
      // Regular character
      newValue = currentValue + e.key;
      e.preventDefault();
    }

    // Update value
    if (newValue !== currentValue) {
      element.value = newValue;
      this.notifyEngine('input', element, { value: newValue });
      console.log(`📝 Input updated: "${newValue}"`);
    }
  }

  /**
   * Handle global keyboard shortcuts
   */
  handleGlobalKeyboard(e) {
    // F5 or Ctrl+R: Reload
    if (e.key === 'F5' || (e.ctrlKey && e.key === 'r')) {
      console.log('🔄 Reload shortcut detected');
      this.notifyEngine('reload');
      e.preventDefault();
    }
    
    // Ctrl+L: Focus URL bar
    if (e.ctrlKey && e.key === 'l') {
      console.log('📍 Focus URL bar shortcut');
      this.notifyEngine('focus-urlbar');
      e.preventDefault();
    }

    // Backspace: Go back
    if (e.key === 'Backspace') {
      console.log('⬅️ Back shortcut');
      this.notifyEngine('navigate-back');
      e.preventDefault();
    }
  }

  /**
   * Convert canvas event coordinates to page coordinates
   */
  getCanvasCoordinates(event) {
    const rect = this.canvas.getBoundingClientRect();
    const scaleX = this.canvas.width / rect.width;
    const scaleY = this.canvas.height / rect.height;
    
    return {
      x: (event.clientX - rect.left) * scaleX,
      y: (event.clientY - rect.top) * scaleY
    };
  }

  /**
   * Hit test - find which DOM element is at the given coordinates
   * Returns the topmost element at (x, y) or null
   */
  hitTest(x, y) {
    // Search from end (topmost elements first)
    for (let i = this.domElements.length - 1; i >= 0; i--) {
      const element = this.domElements[i];
      
      if (!element.bounds) continue;

      const { left, top, width, height } = element.bounds;
      
      if (x >= left && x <= left + width &&
          y >= top && y <= top + height) {
        return element;
      }
    }
    
    return null;
  }

  /**
   * Focus an element
   */
  focusElement(element) {
    if (this.focusedElement) {
      this.notifyEngine('blur', this.focusedElement);
    }
    
    this.focusedElement = element;
    this.notifyEngine('focus', element);
    
    console.log(`🎯 Focused: ${element.tag}#${element.id || 'no-id'}`);
  }

  /**
   * Focus next focusable element (Tab key)
   */
  focusNext() {
    const focusable = this.domElements.filter(el => 
      el.tag === 'input' || 
      el.tag === 'textarea' || 
      el.tag === 'button' ||
      el.tag === 'a' ||
      el.tabIndex >= 0
    );

    if (focusable.length === 0) return;

    const currentIndex = focusable.indexOf(this.focusedElement);
    const nextIndex = (currentIndex + 1) % focusable.length;
    
    this.focusElement(focusable[nextIndex]);
  }

  /**
   * Submit a form
   */
  submitForm(form) {
    if (!form) return;

    console.log('📤 Submitting form:', form);
    
    // Collect form data
    const formData = {};
    const formElements = this.domElements.filter(el => el.form === form);
    
    for (const element of formElements) {
      if (element.name && element.value !== undefined) {
        formData[element.name] = element.value;
      }
    }

    const method = form.method || 'GET';
    const action = form.action || this.currentUrl;

    console.log(`📤 Form submission: ${method} ${action}`, formData);
    
    this.notifyEngine('form-submit', form, {
      method,
      action,
      data: formData
    });
  }

  /**
   * Navigate to URL (from link click)
   */
  navigateToURL(url) {
    // Resolve relative URLs
    if (this.currentUrl && !url.match(/^[a-z]+:\/\//)) {
      url = this.resolveURL(this.currentUrl, url);
    }

    console.log(`🌐 Navigation requested: ${url}`);
    this.notifyEngine('navigate', null, { url });
  }

  /**
   * Resolve relative URL against base URL
   */
  resolveURL(base, relative) {
    // Simple implementation - can be enhanced
    if (relative.startsWith('/')) {
      const urlObj = new URL(base);
      return `${urlObj.protocol}//${urlObj.host}${relative}`;
    }
    
    const baseDir = base.substring(0, base.lastIndexOf('/') + 1);
    return baseDir + relative;
  }

  /**
   * Notify engine of event
   */
  notifyEngine(eventType, element, data) {
    // Emit event to any registered listeners
    const event = {
      type: eventType,
      element,
      data,
      timestamp: Date.now()
    };

    // Call registered callbacks
    const listeners = this.listeners[this.getEventCategory(eventType)] || [];
    for (const listener of listeners) {
      try {
        listener(event);
      } catch (error) {
        console.error('EventBridge listener error:', error);
      }
    }

    // Also try to call engine methods directly
    try {
      if (typeof this.engine.handleEvent === 'function') {
        this.engine.handleEvent(event);
      }
    } catch (error) {
      console.warn('Engine handleEvent not available:', error.message);
    }
  }

  /**
   * Get event category for listener organization
   */
  getEventCategory(eventType) {
    if (['mousedown', 'mouseup', 'click', 'mouseover', 'mouseout', 'mousemove'].includes(eventType)) {
      return 'mouse';
    }
    if (['keydown', 'keyup', 'keypress', 'input'].includes(eventType)) {
      return 'keyboard';
    }
    if (['focus', 'blur'].includes(eventType)) {
      return 'focus';
    }
    return 'other';
  }

  /**
   * Register event listener
   */
  on(category, callback) {
    if (!this.listeners[category]) {
      this.listeners[category] = [];
    }
    this.listeners[category].push(callback);
  }

  /**
   * Set current URL (for relative URL resolution)
   */
  setCurrentURL(url) {
    this.currentUrl = url;
  }

  /**
   * Cleanup
   */
  destroy() {
    this.disable();
    this.listeners = { mouse: [], keyboard: [], focus: [] };
    this.domElements = [];
    this.focusedElement = null;
    this.hoveredElement = null;
  }
}

module.exports = EventBridge;
