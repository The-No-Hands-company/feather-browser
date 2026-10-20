# PronBrow Engine v0.1.9 - Quick Feature Reference

## 🎨 Color Features

### RGB Colors
```css
/* Basic RGB */
color: rgb(255, 100, 50);
background-color: rgb(0, 128, 255);
```

### RGBA Colors (with transparency)
```css
/* RGBA with 50% opacity */
color: rgba(255, 100, 50, 0.5);

/* Fully transparent */
background-color: rgba(0, 0, 0, 0.0);

/* Fully opaque */
border-color: rgba(255, 0, 0, 1.0);
```

### Short Hex Colors
```css
/* 3-digit hex (auto-expands) */
color: #f00;     /* Same as #ff0000 */
color: #0f0;     /* Same as #00ff00 */
color: #369;     /* Same as #336699 */
```

---

## 📝 Text Features

### Word Spacing
```css
/* Add extra space between words */
p {
    word-spacing: 10px;
}

/* Tighter word spacing */
.tight {
    word-spacing: -2px;
}
```

### Font Variant (Small-Caps)
```css
/* Make lowercase letters small capitals */
h2 {
    font-variant: small-caps;
}

/* Back to normal */
p {
    font-variant: normal;
}
```

### White-Space Modes
```css
/* Collapse whitespace, wrap normally (default) */
.normal {
    white-space: normal;
}

/* Collapse whitespace, prevent wrapping */
.nowrap {
    white-space: nowrap;
}

/* Preserve all whitespace, no wrapping */
.pre {
    white-space: pre;
}

/* Preserve whitespace, allow wrapping */
.pre-wrap {
    white-space: pre-wrap;
}

/* Collapse spaces, preserve newlines */
.pre-line {
    white-space: pre-line;
}
```

---

## 💡 Usage Examples

### Colorful Text with Transparency
```html
<style>
    .highlight {
        color: rgba(255, 100, 50, 0.8);
        background-color: rgba(255, 255, 200, 0.3);
    }
</style>

<p class="highlight">Semi-transparent text on semi-transparent background</p>
```

### Spaced-Out Heading
```html
<style>
    h1 {
        font-variant: small-caps;
        word-spacing: 15px;
        letter-spacing: 3px;
        color: rgb(0, 102, 204);
    }
</style>

<h1>Stylish Heading Text</h1>
```

### Code Block with Preserved Formatting
```html
<style>
    .code {
        white-space: pre;
        color: #369;
        background-color: #f5f5f5;
        font-variant: small-caps;
    }
</style>

<pre class="code">
function example() {
    const data = {
        name:  "value",
        count: 42
    };
}
</pre>
```

### Colorful Buttons
```html
<style>
    .btn-primary {
        background-color: rgb(0, 123, 255);
        color: #fff;
        padding: 10px;
    }
    
    .btn-danger {
        background-color: rgba(220, 53, 69, 0.9);
        color: #fff;
        padding: 10px;
    }
</style>
```

---

## 🔍 White-Space Mode Comparison

```html
<!-- Input HTML -->
<p>This    has     multiple     spaces</p>

<!-- white-space: normal -->
Result: "This has multiple spaces"

<!-- white-space: nowrap -->
Result: "This has multiple spaces" (no wrapping)

<!-- white-space: pre -->
Result: "This    has     multiple     spaces"

<!-- white-space: pre-wrap -->
Result: "This    has     multiple     spaces" (wraps if needed)

<!-- white-space: pre-line -->
Result: "This has multiple spaces" (preserves \n)
```

---

## ⚡ Performance Tips

1. **RGBA Colors**: Use sparingly for transparency, prefer opaque colors when possible
2. **Word Spacing**: Large values can impact layout performance
3. **White-Space Pre**: Can create very wide layouts, use with caution
4. **Small-Caps**: Slightly slower than normal rendering

---

## 🐛 Known Limitations

### Current Constraints
- No named colors (e.g., "red", "blue")
- No HSL/HSLA support yet
- Small-caps uses scaled uppercase (not dedicated glyphs)
- RGB values must be 0-255 (no percentage support)
- Alpha values must be 0.0-1.0

### Coming Soon
- Named color support (v0.2.0)
- HSL/HSLA colors (v0.2.0)
- Dedicated small-caps fonts
- Percentage-based RGB values
- More sophisticated typography

---

## 📚 CSS Properties Supported

### Complete List
```css
/* Colors */
color: #rrggbb;
color: #rgb;
color: rgb(r, g, b);
color: rgba(r, g, b, a);
background-color: /* same as above */
border-color: /* same as above */

/* Text Styling */
font-size: 16px;
font-weight: bold | 400-900;
font-style: italic | normal;
font-variant: small-caps | normal;
text-align: left | center | right | justify;
text-decoration: underline | overline | line-through | none;
text-transform: uppercase | lowercase | capitalize | none;

/* Spacing */
letter-spacing: 2px;
word-spacing: 5px;
line-height: 1.5;
white-space: normal | nowrap | pre | pre-wrap | pre-line;

/* Layout */
margin: 10px;
padding: 20px;
width: 400px;
height: 200px;
border-width: 2px;
border-style: solid | dashed | dotted;

/* Visual Effects */
opacity: 0.5;
border-radius: 5px;
box-shadow: 2px 2px 4px #000;
visibility: visible | hidden;
```

---

## 🎯 Next Features Coming

1. **Named Colors** - red, blue, green, etc.
2. **HSL/HSLA** - Hue-Saturation-Lightness colors
3. **Text Shadow** - Drop shadows on text
4. **Font Stretch** - Condensed/expanded fonts
5. **Gradients** - Linear and radial gradients
6. **Transforms** - Rotate, scale, translate
7. **Animations** - CSS animations and transitions

---

*Last Updated: v0.1.9 (November 12, 2024)*
