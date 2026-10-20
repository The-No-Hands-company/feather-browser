// Optional canvas module for Node.js testing
let nodeCanvas = null;
try {
  nodeCanvas = require('canvas');
} catch (e) {
  // Canvas module not available (will use browser canvas instead)
}

/**
 * Canvas-based renderer for pronbrow-engine output
 * Renders RGBA pixel data to an HTML5 canvas
 * Works in both Node.js (with node-canvas) and Electron renderer
 */
class CanvasRenderer {
  constructor(width = 1024, height = 768) {
    this.width = width;
    this.height = height;
    this.canvas = null;
    this.ctx = null;
    this.offscreenCanvas = null;
    this.offscreenCtx = null;
    this.isNodeEnvironment = typeof window === 'undefined';
    
    // Only initialize offscreen canvas in Node.js environment
    if (this.isNodeEnvironment && nodeCanvas) {
      this.initializeOffscreenCanvas();
    }
  }

  initializeOffscreenCanvas() {
    try {
      if (nodeCanvas) {
        this.offscreenCanvas = nodeCanvas.createCanvas(this.width, this.height);
        this.offscreenCtx = this.offscreenCanvas.getContext('2d');
        console.log(`✓ Canvas initialized: ${this.width}x${this.height}`);
      }
    } catch (error) {
      console.warn('Could not create offscreen canvas:', error.message);
    }
  }

  setTargetCanvas(canvasElement) {
    this.canvas = canvasElement;
    this.ctx = canvasElement.getContext('2d');
    this.canvas.width = this.width;
    this.canvas.height = this.height;
    console.log('✓ Target canvas set');
  }

  renderPixels(rgba_bytes, width, height) {
    if (!rgba_bytes || rgba_bytes.length === 0) {
      console.warn('No pixel data to render');
      return;
    }

    const expectedSize = width * height * 4;
    if (rgba_bytes.length !== expectedSize) {
      console.error(`Pixel data size mismatch: expected ${expectedSize}, got ${rgba_bytes.length}`);
      return;
    }

    // Skip rendering if no canvas context available (e.g., in Node.js main process)
    if (!this.ctx) {
      console.log(`✓ Pixel data prepared (${width}x${height}) - no canvas context`);
      return;
    }

    try {
      const imageData = this.ctx.createImageData(width, height);
      
      if (rgba_bytes instanceof Uint8Array) {
        imageData.data.set(rgba_bytes);
      } else {
        for (let i = 0; i < rgba_bytes.length; i++) {
          imageData.data[i] = rgba_bytes[i];
        }
      }

      this.ctx.fillStyle = 'white';
      this.ctx.fillRect(0, 0, this.width, this.height);
      this.ctx.putImageData(imageData, 0, 0);
      
      console.log(`✓ Rendered ${width}x${height} pixels`);
    } catch (error) {
      console.error('Error rendering pixels:', error);
      this.renderError(error.message);
    }
  }

  renderError(message) {
    if (!this.ctx) return;
    this.ctx.fillStyle = '#f5f5f5';
    this.ctx.fillRect(0, 0, this.width, this.height);
    this.ctx.fillStyle = '#d32f2f';
    this.ctx.font = 'bold 24px Arial';
    this.ctx.textAlign = 'center';
    this.ctx.fillText('⚠️ Rendering Error', this.width / 2, this.height / 2 - 30);
    this.ctx.fillStyle = '#333';
    this.ctx.font = '16px Arial';
    this.ctx.fillText(message, this.width / 2, this.height / 2 + 10);
  }

  renderLoading() {
    if (!this.ctx) return;
    this.ctx.fillStyle = '#f5f5f5';
    this.ctx.fillRect(0, 0, this.width, this.height);
    this.ctx.fillStyle = '#666';
    this.ctx.font = '20px Arial';
    this.ctx.textAlign = 'center';
    this.ctx.fillText('Loading...', this.width / 2, this.height / 2);
  }

  renderBlank() {
    if (!this.ctx) return;
    this.ctx.fillStyle = 'white';
    this.ctx.fillRect(0, 0, this.width, this.height);
  }

  resize(width, height) {
    this.width = width;
    this.height = height;
    if (this.canvas) {
      this.canvas.width = width;
      this.canvas.height = height;
    }
    if (this.isNodeEnvironment && nodeCanvas && this.offscreenCanvas) {
      this.offscreenCanvas = nodeCanvas.createCanvas(width, height);
      this.offscreenCtx = this.offscreenCanvas.getContext('2d');
    }
    console.log(`✓ Canvas resized to ${width}x${height}`);
  }

  toDataURL(format = 'image/png') {
    if (!this.canvas) {
      return this.offscreenCanvas?.toDataURL(format);
    }
    return this.canvas.toDataURL(format);
  }

  clear() {
    if (this.ctx) {
      this.ctx.clearRect(0, 0, this.width, this.height);
    }
  }

  destroy() {
    this.clear();
    this.canvas = null;
    this.ctx = null;
    this.offscreenCanvas = null;
    this.offscreenCtx = null;
  }
}

module.exports = CanvasRenderer;
