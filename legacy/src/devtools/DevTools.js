const DevToolsConsole = require('./Console');
const NetworkMonitor = require('./NetworkMonitor');
const PerformanceProfiler = require('./PerformanceProfiler');

/**
 * DevTools Manager
 * Coordinates all developer tools for a tab
 */
class DevTools {
  constructor(tab) {
    this.tab = tab;
    this.console = new DevToolsConsole(tab);
    this.network = new NetworkMonitor(tab);
    this.performance = new PerformanceProfiler(tab);
    this.isOpen = false;
  }

  open() {
    this.isOpen = true;
    console.log('✓ DevTools opened for tab:', this.tab.id);
  }

  close() {
    this.isOpen = false;
    console.log('✓ DevTools closed for tab:', this.tab.id);
  }

  toggle() {
    this.isOpen ? this.close() : this.open();
  }

  // Console methods
  executeConsoleCommand(command) {
    return this.console.executeCommand(command);
  }

  getConsoleLogs() {
    return this.console.getLogs();
  }

  clearConsole() {
    this.console.clear();
  }

  // Network methods
  getNetworkRequests() {
    return this.network.getRequests();
  }

  getNetworkStatistics() {
    return this.network.getStatistics();
  }

  clearNetwork() {
    this.network.clear();
  }

  // Performance methods
  getPerformanceMetrics() {
    return this.performance.getMetrics();
  }

  getPerformanceMeasures() {
    return this.performance.getMeasures();
  }

  clearPerformance() {
    this.performance.clear();
  }

  // DOM inspector
  inspectElement(element) {
    // TODO: Implement DOM inspection
    console.log('Inspecting element:', element);
  }

  // Overall stats
  getStats() {
    return {
      console: {
        logs: this.console.getLogs().length,
      },
      network: this.network.getStatistics(),
      performance: this.performance.getMetrics(),
    };
  }

  destroy() {
    this.console.destroy();
    this.network = null;
    this.performance = null;
  }
}

module.exports = DevTools;
