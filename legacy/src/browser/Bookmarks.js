const fs = require('fs');
const path = require('path');
const { app } = require('electron');

class Bookmarks {
  constructor() {
    this.bookmarksFile = path.join(app.getPath('userData'), 'bookmarks.json');
    this.items = this.load();
  }

  load() {
    try {
      if (fs.existsSync(this.bookmarksFile)) {
        const data = fs.readFileSync(this.bookmarksFile, 'utf8');
        return JSON.parse(data);
      }
    } catch (error) {
      console.error('Failed to load bookmarks:', error);
    }
    return [];
  }

  save() {
    try {
      fs.writeFileSync(this.bookmarksFile, JSON.stringify(this.items, null, 2));
    } catch (error) {
      console.error('Failed to save bookmarks:', error);
    }
  }

  add(bookmark) {
    this.items.push({
      url: bookmark.url,
      title: bookmark.title,
      folder: bookmark.folder || 'default',
      timestamp: Date.now()
    });
    this.save();
  }

  remove(url) {
    this.items = this.items.filter(item => item.url !== url);
    this.save();
  }

  getAll() {
    return this.items;
  }

  getByFolder(folder) {
    return this.items.filter(item => item.folder === folder);
  }

  exists(url) {
    return this.items.some(item => item.url === url);
  }
}

module.exports = Bookmarks;
