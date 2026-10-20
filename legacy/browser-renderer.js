const { ipcRenderer } = require('electron');
const EventBridge = require('./src/engine-bridge/EventBridge');

// UI Elements
const urlBar = document.getElementById('url-bar');
const goBtn = document.getElementById('go-btn');
const backBtn = document.getElementById('back-btn');
const forwardBtn = document.getElementById('forward-btn');
const reloadBtn = document.getElementById('reload-btn');
const canvas = document.getElementById('render-canvas');
const statusIndicator = document.getElementById('status-indicator');
const statusText = document.getElementById('status-text');
const loadingOverlay = document.getElementById('loading-overlay');
const performanceInfo = document.getElementById('performance-info');

// State
let history = [];
let historyIndex = -1;
let isLoading = false;
let currentUrl = '';

// Event bridge for user interactions
let eventBridge = null;

// Initialize canvas size
function resizeCanvas() {
  const container = canvas.parentElement;
  canvas.width = container.clientWidth;
  canvas.height = container.clientHeight;
  console.log(`Canvas resized to ${canvas.width}x${canvas.height}`);
}

window.addEventListener('resize', resizeCanvas);
resizeCanvas();

// Navigation functions
async function navigate(url) {
  if (!url || isLoading) return;
  
  // Normalize URL
  url = normalizeURL(url);
  
  console.log(`🌐 Navigating to: ${url}`);
  setLoading(true);
  setStatus('loading', 'Loading...');
  
  const startTime = performance.now();
  
  try {
    // Request rendering from main process
    const result = await ipcRenderer.invoke('render-page', {
      url,
      width: canvas.width,
      height: canvas.height
    });
    
    if (result.success) {
      // Store current URL
      currentUrl = url;
      
      // Update event bridge with current URL
      if (eventBridge) {
        eventBridge.setCurrentURL(url);
      }
      
      // Update history
      if (historyIndex < history.length - 1) {
        history = history.slice(0, historyIndex + 1);
      }
      history.push(url);
      historyIndex = history.length - 1;
      updateNavigationButtons();
      
      // Update URL bar
      urlBar.value = url;
      
      // Update DOM layout for hit testing
      await updateDOMLayout();
      
      const renderTime = Math.round(performance.now() - startTime);
      performanceInfo.textContent = `${renderTime}ms render time`;
      setStatus('ready', 'Ready');
    } else {
      throw new Error(result.error || 'Unknown error');
    }
  } catch (error) {
    console.error('Navigation error:', error);
    setStatus('error', `Error: ${error.message}`);
  } finally {
    setLoading(false);
  }
}

function normalizeURL(url) {
  url = url.trim();
  
  // Built-in pages
  if (url.startsWith('pronbrow://')) {
    return url;
  }
  
  // Local files
  if (url.startsWith('file://')) {
    return url;
  }
  
  // Add https if no protocol
  if (!url.match(/^https?:\/\//)) {
    // If it looks like a domain
    if (url.includes('.') && !url.includes(' ')) {
      return 'https://' + url;
    }
    // Otherwise, search
    return 'https://www.google.com/search?q=' + encodeURIComponent(url);
  }
  
  return url;
}

function goBack() {
  if (historyIndex > 0) {
    historyIndex--;
    navigate(history[historyIndex]);
  }
}

function goForward() {
  if (historyIndex < history.length - 1) {
    historyIndex++;
    navigate(history[historyIndex]);
  }
}

function reload() {
  if (historyIndex >= 0 && history[historyIndex]) {
    navigate(history[historyIndex]);
  }
}

function updateNavigationButtons() {
  backBtn.disabled = historyIndex <= 0;
  forwardBtn.disabled = historyIndex >= history.length - 1;
}

function setLoading(loading) {
  isLoading = loading;
  if (loading) {
    loadingOverlay.classList.add('active');
    statusIndicator.classList.add('loading');
  } else {
    loadingOverlay.classList.remove('active');
    statusIndicator.classList.remove('loading');
  }
}

function setStatus(type, message) {
  statusText.textContent = message;
  statusIndicator.className = 'status-indicator';
  if (type === 'loading') {
    statusIndicator.classList.add('loading');
  } else if (type === 'error') {
    statusIndicator.classList.add('error');
  }
}

// Event listeners
goBtn.addEventListener('click', () => {
  navigate(urlBar.value);
});

urlBar.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    navigate(urlBar.value);
  }
});

backBtn.addEventListener('click', goBack);
forwardBtn.addEventListener('click', goForward);
reloadBtn.addEventListener('click', reload);

// Handle rendered frame from main process
ipcRenderer.on('frame-rendered', (event, data) => {
  console.log('📊 Received rendered frame');
  
  const ctx = canvas.getContext('2d');
  
  // Create ImageData from the RGBA bytes
  const imageData = ctx.createImageData(data.width, data.height);
  const pixels = new Uint8ClampedArray(data.pixels);
  imageData.data.set(pixels);
  
  // Clear and draw
  ctx.fillStyle = 'white';
  ctx.fillRect(0, 0, canvas.width, canvas.height);
  ctx.putImageData(imageData, 0, 0);
  
  console.log('✓ Frame rendered to canvas');
});

// Update DOM layout for hit testing
async function updateDOMLayout() {
  try {
    const result = await ipcRenderer.invoke('get-dom-layout');
    if (result.success && eventBridge) {
      eventBridge.updateDOMLayout(result.layout);
      console.log(`🎯 DOM layout updated: ${result.layout.length} elements`);
    }
  } catch (error) {
    console.warn('Failed to update DOM layout:', error);
  }
}

// Initialize event bridge
function initializeEventBridge() {
  if (eventBridge) {
    eventBridge.destroy();
  }
  
  // Create a mock engine for the event bridge (we'll handle events via IPC)
  const mockEngine = {
    handleEvent: (event) => {
      console.log('🎮 Event:', event.type, event);
    }
  };
  
  eventBridge = new EventBridge(mockEngine, canvas);
  eventBridge.setCurrentURL(currentUrl);
  
  // Handle navigation events
  eventBridge.on('mouse', async (event) => {
    if (event.type === 'navigate') {
      console.log('🔗 Navigation from click:', event.data.url);
      await navigate(event.data.url);
    }
  });
  
  // Handle form submission events
  eventBridge.on('keyboard', async (event) => {
    if (event.type === 'form-submit') {
      console.log('📤 Form submission:', event.data);
      // Handle form submission (will implement in next iteration)
    }
  });
  
  // Enable event capture
  eventBridge.enable();
  console.log('✅ Event bridge initialized and enabled');
}

// Listen for canvas resize requests
ipcRenderer.on('resize-viewport', (event, { width, height }) => {
  canvas.width = width;
  canvas.height = height;
});

// Keyboard shortcuts
document.addEventListener('keydown', (e) => {
  // Ctrl/Cmd + R: Reload
  if ((e.ctrlKey || e.metaKey) && e.key === 'r') {
    e.preventDefault();
    reload();
  }
  
  // Ctrl/Cmd + L: Focus URL bar
  if ((e.ctrlKey || e.metaKey) && e.key === 'l') {
    e.preventDefault();
    urlBar.select();
  }
  
  // Alt + Left: Back
  if (e.altKey && e.key === 'ArrowLeft') {
    e.preventDefault();
    goBack();
  }
  
  // Alt + Right: Forward
  if (e.altKey && e.key === 'ArrowRight') {
    e.preventDefault();
    goForward();
  }
});

// Initialize
console.log('🚀 PronBrow browser renderer initialized');
setStatus('ready', 'Ready');

// Initialize event bridge for user interactions
initializeEventBridge();

// Load initial page
setTimeout(() => {
  navigate(urlBar.value);
}, 100);
