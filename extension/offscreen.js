chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.target !== 'offscreen') return;

  const canvas = document.getElementById('canvas');
  const ctx = canvas.getContext('2d');

  if (message.type === 'init') {
    canvas.width = message.width;
    canvas.height = message.height;
    sendResponse({ success: true });
  } else if (message.type === 'draw') {
    const { dataUrl, x, y, width, height } = message;
    const img = new Image();
    img.onload = () => {
      ctx.drawImage(img, 0, 0, img.width, img.height, x, y, width, height);
      sendResponse({ success: true });
    };
    img.src = dataUrl;
    return true; // Keep channel open
  } else if (message.type === 'getResult') {
    // Return base64 JPEG
    const result = canvas.toDataURL('image/jpeg', 0.8);
    sendResponse({ result });
  }
});