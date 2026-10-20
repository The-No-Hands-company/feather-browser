const EventEmitter = require('events');

/**
 * Developer Tools Console
 * Provides JavaScript console for debugging
 */
class DevToolsConsole extends EventEmitter {
  constructor(tab) {
    super();
    this.tab = tab;
    this.logs = [];
    this.maxLogs = 1000;
    this.filterLevel = 'all'; // all, log, warn, error
    
    // Listen to tab console events
    this.tab.on('console', (data) => {
      this.addLog(data);
    });

    this.tab.on('js-error', (data) => {
      this.addLog({
        type: 'error',
        args: [data.error.message],
        stack: data.error.stack,
        filename: data.filename,
      });
    });
  }

  addLog(logData) {
    const log = {
      ...logData,
      timestamp: Date.now(),
      id: this.logs.length,
    };

    this.logs.push(log);

    // Limit log size
    if (this.logs.length > this.maxLogs) {
      this.logs.shift();
    }

    this.emit('log', log);
  }

  executeCommand(command) {
    try {
      // Execute in page context
      const result = this.tab.renderer.evaluateExpression(command);
      
      this.addLog({
        type: 'command',
        args: [command],
        result: result,
      });

      this.addLog({
        type: 'result',
        args: [result],
      });

      return result;
    } catch (error) {
      this.addLog({
        type: 'error',
        args: [error.message],
        stack: error.stack,
      });
      throw error;
    }
  }

  clear() {
    this.logs = [];
    this.emit('cleared');
  }

  setFilter(level) {
    this.filterLevel = level;
    this.emit('filter-changed', level);
  }

  getFilteredLogs() {
    if (this.filterLevel === 'all') {
      return this.logs;
    }
    return this.logs.filter(log => log.type === this.filterLevel);
  }

  getLogs() {
    return this.logs;
  }

  destroy() {
    this.removeAllListeners();
    this.logs = [];
  }
}

module.exports = DevToolsConsole;
