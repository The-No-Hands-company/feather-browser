const webview = document.getElementById('webview');
const urlBar = document.getElementById('url-bar');
const backBtn = document.getElementById('back-btn');
const forwardBtn = document.getElementById('forward-btn');
const reloadBtn = document.getElementById('reload-btn');
const goBtn = document.getElementById('go-btn');

webview.addEventListener('did-start-loading', () => {
  console.log('Loading...');
});

webview.addEventListener('did-stop-loading', () => {
  urlBar.value = webview.getURL();
  console.log('Loaded:', webview.getURL());
});

webview.addEventListener('page-title-updated', (e) => {
  document.title = e.title + ' - PronBrow';
});

function navigate(url) {
  if (!url) return;
  
  if (!url.match(/^https?:\/\//)) {
    if (url.includes('.') && !url.includes(' ')) {
      url = 'https://' + url;
    } else {
      url = 'https://www.google.com/search?q=' + encodeURIComponent(url);
    }
  }
  
  webview.src = url;
}

goBtn.addEventListener('click', () => {
  navigate(urlBar.value);
});

urlBar.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    navigate(urlBar.value);
  }
});

backBtn.addEventListener('click', () => {
  if (webview.canGoBack()) {
    webview.goBack();
  }
});

forwardBtn.addEventListener('click', () => {
  if (webview.canGoForward()) {
    webview.goForward();
  }
});

reloadBtn.addEventListener('click', () => {
  webview.reload();
});
