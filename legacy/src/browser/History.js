const fs = require('fs');
const path = require('path');
const { app } = require('electron');

class History {
  constructor() {
    this.historyFile = path.join(app.getPath('userData'), 'history.json');
    this.items = this.load();
  }

  load() {
    try {
      if (fs.existsSync(this.historyFile)) {
        const data = fs.readFileSync(this.historyFile, 'utf8');
        return JSON.parse(data);
      }
    } catch (error) {
      console.error('Failed to load history:', error);
    }
    return [];
  }

  save() {
    try {
      fs.writeFileSync(this.historyFile, JSON.stringify(this.items, null, 2));
    } catch (error) {
      console.error('Failed to save history:', error);
    }
  }

  add(item) {
    this.items.unshift({
      url: item.url,
      title: item.title,
      timestamp: item.timestamp || Date.now()
    });
    
    // Keep only last 1000 items
    if (this.items.length > 1000) {
      this.items = this.items.slice(0, 1000);
    }
    
    this.save();
  }

  getRecent(count = 50) {
    return this.items.slice(0, count);
  }

  search(query) {
    const lowerQuery = query.toLowerCase();
    return this.items.filter(item => 
      item.url.toLowerCase().includes(lowerQuery) ||
      item.title.toLowerCase().includes(lowerQuery)
    ).slice(0, 50);
  }

  clear() {
    this.items = [];
    this.save();
  }

  remove(url) {
    this.items = this.items.filter(item => item.url !== url);
    this.save();
  }
}

module.exports = History;
