/**
 * DevTools Network Monitor
 * Tracks network requests and responses
 */
class NetworkMonitor {
  constructor(tab) {
    this.tab = tab;
    this.requests = [];
    this.maxRequests = 500;
    this.nextRequestId = 1;
  }

  logRequest(url, options = {}) {
    const request = {
      id: this.nextRequestId++,
      url,
      method: options.method || 'GET',
      headers: options.headers || {},
      timestamp: Date.now(),
      status: 'pending',
      startTime: performance.now(),
    };

    this.requests.push(request);

    if (this.requests.length > this.maxRequests) {
      this.requests.shift();
    }

    return request.id;
  }

  logResponse(requestId, response) {
    const request = this.requests.find(r => r.id === requestId);
    if (!request) return;

    request.status = 'completed';
    request.statusCode = response.status;
    request.statusText = response.statusText;
    request.responseHeaders = response.headers;
    request.endTime = performance.now();
    request.duration = request.endTime - request.startTime;
    request.size = response.size || 0;
  }

  logError(requestId, error) {
    const request = this.requests.find(r => r.id === requestId);
    if (!request) return;

    request.status = 'failed';
    request.error = error.message;
    request.endTime = performance.now();
    request.duration = request.endTime - request.startTime;
  }

  clear() {
    this.requests = [];
  }

  getRequests() {
    return this.requests;
  }

  getRequestById(id) {
    return this.requests.find(r => r.id === id);
  }

  getStatistics() {
    const completed = this.requests.filter(r => r.status === 'completed');
    const failed = this.requests.filter(r => r.status === 'failed');
    const totalSize = completed.reduce((sum, r) => sum + (r.size || 0), 0);
    const avgDuration = completed.length > 0
      ? completed.reduce((sum, r) => sum + r.duration, 0) / completed.length
      : 0;

    return {
      total: this.requests.length,
      completed: completed.length,
      failed: failed.length,
      pending: this.requests.filter(r => r.status === 'pending').length,
      totalSize,
      avgDuration,
    };
  }
}

module.exports = NetworkMonitor;
