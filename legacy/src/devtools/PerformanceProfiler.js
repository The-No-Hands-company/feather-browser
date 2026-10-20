/**
 * DevTools Performance Profiler
 * Tracks page performance metrics
 */
class PerformanceProfiler {
  constructor(tab) {
    this.tab = tab;
    this.metrics = {
      loadStart: null,
      domContentLoaded: null,
      loadComplete: null,
      firstPaint: null,
      firstContentfulPaint: null,
    };
    this.marks = new Map();
    this.measures = [];
  }

  recordLoadStart() {
    this.metrics.loadStart = performance.now();
  }

  recordDOMContentLoaded() {
    this.metrics.domContentLoaded = performance.now();
  }

  recordLoadComplete() {
    this.metrics.loadComplete = performance.now();
  }

  recordFirstPaint() {
    this.metrics.firstPaint = performance.now();
  }

  recordFirstContentfulPaint() {
    this.metrics.firstContentfulPaint = performance.now();
  }

  mark(name) {
    this.marks.set(name, performance.now());
  }

  measure(name, startMark, endMark) {
    const startTime = this.marks.get(startMark);
    const endTime = this.marks.get(endMark);

    if (startTime === undefined || endTime === undefined) {
      console.warn(`Cannot measure: mark not found`);
      return;
    }

    const measure = {
      name,
      startTime,
      endTime,
      duration: endTime - startTime,
      timestamp: Date.now(),
    };

    this.measures.push(measure);
    return measure;
  }

  getMetrics() {
    const loadTime = this.metrics.loadComplete && this.metrics.loadStart
      ? this.metrics.loadComplete - this.metrics.loadStart
      : null;

    const domLoadTime = this.metrics.domContentLoaded && this.metrics.loadStart
      ? this.metrics.domContentLoaded - this.metrics.loadStart
      : null;

    return {
      ...this.metrics,
      loadTime,
      domLoadTime,
    };
  }

  getMeasures() {
    return this.measures;
  }

  clear() {
    this.marks.clear();
    this.measures = [];
  }

  reset() {
    this.metrics = {
      loadStart: null,
      domContentLoaded: null,
      loadComplete: null,
      firstPaint: null,
      firstContentfulPaint: null,
    };
    this.clear();
  }
}

module.exports = PerformanceProfiler;
