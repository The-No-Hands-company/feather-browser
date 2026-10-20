const { ipcRenderer } = require('electron');

// Add error handling
window.addEventListener('error', (e) => {
  console.error('Global error:', e.error);
  console.error('Stack:', e.error?.stack);
});

window.addEventListener('unhandledrejection', (e) => {
  console.error('Unhandled promise rejection:', e.reason);
});

class BrowserUI {
  constructor() {
    try {
      this.tabs = [];
      this.activeTabIndex = 0;
      this.init();
    } catch (error) {
      console.error('BrowserUI constructor error:', error);
      throw error;
    }
  }

  init() {
    this.setupWindowControls();
    this.setupNavigation();
    this.setupTabs();
    this.setupMenu();
    this.setupKeyboardShortcuts();
    this.setupIPCListeners();
    
    // Create initial tab
    this.createTab('pronbrow://newtab');
  }

  setupWindowControls() {
    document.getElementById('minimize-btn')?.addEventListener('click', () => {
      ipcRenderer.send('window-minimize');
    });

    document.getElementById('maximize-btn')?.addEventListener('click', () => {
      ipcRenderer.send('window-maximize');
    });

    document.getElementById('close-btn')?.addEventListener('click', () => {
      ipcRenderer.send('window-close');
    });
  }

  setupNavigation() {
    const urlBar = document.getElementById('url-bar');
    const backBtn = document.getElementById('back-btn');
    const forwardBtn = document.getElementById('forward-btn');
    const reloadBtn = document.getElementById('reload-btn');
    const homeBtn = document.getElementById('home-btn');
    const bookmarkBtn = document.getElementById('bookmark-btn');

    urlBar.addEventListener('keypress', (e) => {
      if (e.key === 'Enter') {
        this.navigate(urlBar.value);
      }
    });

    urlBar.addEventListener('input', (e) => {
      this.showSuggestions(e.target.value);
    });

    urlBar.addEventListener('focus', () => {
      urlBar.select();
    });

    backBtn.addEventListener('click', () => {
      ipcRenderer.send('go-back');
    });

    forwardBtn.addEventListener('click', () => {
      ipcRenderer.send('go-forward');
    });

    reloadBtn.addEventListener('click', () => {
      ipcRenderer.send('reload');
    });

    homeBtn.addEventListener('click', () => {
      this.navigate('pronbrow://newtab');
    });

    bookmarkBtn.addEventListener('click', () => {
      this.toggleBookmark();
    });
  }

  setupTabs() {
    const newTabBtn = document.getElementById('new-tab-btn');
    
    newTabBtn.addEventListener('click', () => {
      this.createTab('pronbrow://newtab');
    });
  }

  setupMenu() {
    const menuBtn = document.getElementById('menu-btn');
    const menuPanel = document.getElementById('menu-panel');
    
    menuBtn.addEventListener('click', () => {
      menuPanel.classList.toggle('hidden');
    });

    // Close menu when clicking outside
    document.addEventListener('click', (e) => {
      if (!menuBtn.contains(e.target) && !menuPanel.contains(e.target)) {
        menuPanel.classList.add('hidden');
      }
    });

    // Menu actions
    menuPanel.querySelectorAll('.menu-item').forEach(item => {
      item.addEventListener('click', () => {
        const action = item.dataset.action;
        this.handleMenuAction(action);
        menuPanel.classList.add('hidden');
      });
    });

    // DevTools
    document.getElementById('devtools-btn').addEventListener('click', () => {
      ipcRenderer.send('open-devtools');
    });
  }

  setupKeyboardShortcuts() {
    document.addEventListener('keydown', (e) => {
      // Ctrl+T - New Tab
      if (e.ctrlKey && e.key === 't') {
        e.preventDefault();
        this.createTab('pronbrow://newtab');
      }
      
      // Ctrl+W - Close Tab
      if (e.ctrlKey && e.key === 'w') {
        e.preventDefault();
        this.closeTab(this.activeTabIndex);
      }
      
      // Ctrl+Tab - Next Tab
      if (e.ctrlKey && e.key === 'Tab') {
        e.preventDefault();
        const nextIndex = (this.activeTabIndex + 1) % this.tabs.length;
        this.switchTab(nextIndex);
      }
      
      // Ctrl+Shift+Tab - Previous Tab
      if (e.ctrlKey && e.shiftKey && e.key === 'Tab') {
        e.preventDefault();
        const prevIndex = this.activeTabIndex === 0 ? this.tabs.length - 1 : this.activeTabIndex - 1;
        this.switchTab(prevIndex);
      }
      
      // Ctrl+L - Focus URL bar
      if (e.ctrlKey && e.key === 'l') {
        e.preventDefault();
        document.getElementById('url-bar').focus();
      }
      
      // Ctrl+R or F5 - Reload
      if ((e.ctrlKey && e.key === 'r') || e.key === 'F5') {
        e.preventDefault();
        ipcRenderer.send('reload');
      }
      
      // Ctrl+D - Bookmark
      if (e.ctrlKey && e.key === 'd') {
        e.preventDefault();
        this.toggleBookmark();
      }
      
      // F12 - DevTools
      if (e.key === 'F12') {
        e.preventDefault();
        ipcRenderer.send('open-devtools');
      }
    });
  }

  setupIPCListeners() {
    ipcRenderer.on('tab-created', (event, data) => {
      this.renderTab(data);
    });

    ipcRenderer.on('tab-closed', (event, data) => {
      this.removeTabUI(data.index);
      this.switchTab(data.newActiveIndex);
    });

    ipcRenderer.on('tab-switched', (event, data) => {
      this.updateUIForTab(data.tab);
    });

    ipcRenderer.on('tab-updated', (event, data) => {
      this.updateTabUI(data);
    });
  }

  createTab(url) {
    ipcRenderer.send('new-tab', url);
  }

  closeTab(index) {
    ipcRenderer.send('close-tab', index);
  }

  switchTab(index) {
    this.activeTabIndex = index;
    
    // Update UI
    document.querySelectorAll('.tab').forEach((tab, i) => {
      tab.classList.toggle('active', i === index);
    });
    
    ipcRenderer.send('switch-tab', index);
  }

  navigate(url) {
    ipcRenderer.send('navigate', url);
  }

  renderTab(data) {
    const tabsContainer = document.getElementById('tabs-container');
    const tab = document.createElement('div');
    tab.className = 'tab';
    if (data.index === this.activeTabIndex) {
      tab.classList.add('active');
    }
    
    tab.innerHTML = `
      <div class="tab-favicon">🌐</div>
      <span class="tab-title">${data.title}</span>
      <button class="tab-close">×</button>
    `;
    
    tab.addEventListener('click', (e) => {
      if (!e.target.classList.contains('tab-close')) {
        this.switchTab(data.index);
      }
    });
    
    tab.querySelector('.tab-close').addEventListener('click', (e) => {
      e.stopPropagation();
      this.closeTab(data.index);
    });
    
    tabsContainer.appendChild(tab);
    this.tabs.push({ element: tab, data });
  }

  removeTabUI(index) {
    if (this.tabs[index]) {
      this.tabs[index].element.remove();
      this.tabs.splice(index, 1);
    }
  }

  updateTabUI(data) {
    const tab = this.tabs[this.activeTabIndex];
    if (tab) {
      const titleEl = tab.element.querySelector('.tab-title');
      const faviconEl = tab.element.querySelector('.tab-favicon');
      
      if (titleEl) titleEl.textContent = data.title || 'New Tab';
      if (faviconEl && data.favicon) {
        faviconEl.innerHTML = `<img src="${data.favicon}" width="16" height="16" alt="">`;
      }
      
      // Update URL bar
      document.getElementById('url-bar').value = data.url || '';
      
      // Update navigation buttons
      document.getElementById('back-btn').disabled = !data.canGoBack;
      document.getElementById('forward-btn').disabled = !data.canGoForward;
      
      // Update title
      document.getElementById('page-title').textContent = data.title || '';
      
      // Update engine indicator
      const engineIndicator = document.getElementById('engine-indicator');
      if (data.usePronBrowEngine) {
        engineIndicator.textContent = '🚀 PronBrow';
        engineIndicator.style.color = '#8ab4f8';
      } else {
        engineIndicator.textContent = '⚡ Chromium';
        engineIndicator.style.color = '#9aa0a6';
      }
    }
  }

  updateUIForTab(tabData) {
    this.updateTabUI(tabData);
  }

  showSuggestions(query) {
    if (!query || query.length < 2) {
      document.getElementById('suggestions').classList.remove('visible');
      return;
    }
    
    const suggestions = document.getElementById('suggestions');
    const history = ipcRenderer.sendSync('get-history');
    const bookmarks = ipcRenderer.sendSync('get-bookmarks');
    
    const matches = [...history, ...bookmarks]
      .filter(item => 
        item.url.toLowerCase().includes(query.toLowerCase()) ||
        item.title.toLowerCase().includes(query.toLowerCase())
      )
      .slice(0, 8);
    
    if (matches.length > 0) {
      suggestions.innerHTML = matches.map(item => `
        <div class="suggestion-item" data-url="${item.url}">
          <div class="suggestion-icon">🔗</div>
          <div class="suggestion-content">
            <div class="suggestion-title">${item.title}</div>
            <div class="suggestion-url">${item.url}</div>
          </div>
        </div>
      `).join('');
      
      suggestions.querySelectorAll('.suggestion-item').forEach(item => {
        item.addEventListener('click', () => {
          this.navigate(item.dataset.url);
          suggestions.classList.remove('visible');
        });
      });
      
      suggestions.classList.add('visible');
    } else {
      suggestions.classList.remove('visible');
    }
  }

  toggleBookmark() {
    const tab = this.tabs[this.activeTabIndex];
    if (tab && tab.data) {
      const bookmarks = ipcRenderer.sendSync('get-bookmarks');
      const exists = bookmarks.some(b => b.url === tab.data.url);
      
      if (exists) {
        // Remove bookmark
        ipcRenderer.send('remove-bookmark', tab.data.url);
      } else {
        // Add bookmark
        ipcRenderer.send('add-bookmark', {
          url: tab.data.url,
          title: tab.data.title
        });
      }
    }
  }

  handleMenuAction(action) {
    switch (action) {
      case 'new-tab':
        this.createTab('pronbrow://newtab');
        break;
      case 'new-window':
        // TODO: Open new window
        break;
      case 'new-private':
        // TODO: Open private window
        break;
      case 'history':
        this.navigate('pronbrow://history');
        break;
      case 'bookmarks':
        this.navigate('pronbrow://bookmarks');
        break;
      case 'downloads':
        this.navigate('pronbrow://downloads');
        break;
      case 'settings':
        this.navigate('pronbrow://settings');
        break;
      case 'about':
        this.navigate('pronbrow://about');
        break;
      case 'exit':
        ipcRenderer.send('window-close');
        break;
    }
  }
}

// Initialize browser UI
const browserUI = new BrowserUI();
