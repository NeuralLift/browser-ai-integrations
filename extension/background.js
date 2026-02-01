// Background service worker for Browser AI Assistant

const BACKEND_WS_URL = 'ws://localhost:3000/ws';
const CONTEXT_UPDATE_INTERVAL = 5000; // 5 seconds

let ws = null;
let contextInterval = null;
let isConnected = false;
let lastTabId = null;
let lastUrl = null;
let wsSessionId = null;

// Setup side panel behavior
chrome.sidePanel
  .setPanelBehavior({ openPanelOnActionClick: true })
  .catch((error) => console.error('[Background] Side panel error:', error));

// Initialize WebSocket connection
function connectWebSocket() {
  if (ws && ws.readyState === WebSocket.OPEN) {
    return;
  }

  try {
    ws = new WebSocket(BACKEND_WS_URL);

    ws.onopen = () => {
      isConnected = true;
      startContextUpdates();
    };

    ws.onclose = () => {
      isConnected = false;
      wsSessionId = null;
      stopContextUpdates();
      // Attempt reconnection after 5 seconds
      setTimeout(connectWebSocket, 5000);
    };

    ws.onerror = (error) => {
      console.error('[Background] WebSocket error:', error);
    };

    ws.onmessage = async (event) => {
      try {
        const message = JSON.parse(event.data);

        if (message.type === 'session_init') {
          wsSessionId = message.data.session_id;
        } else if (message.type === 'action_request') {
          const { request_id, command } = message.data;
          // Forward action to sidepanel for UI display and execution
          const result = await forwardActionToSidepanel(command);
          // Send ActionResult back to backend
          const response = JSON.stringify({
            type: 'ActionResult',
            data: {
              request_id: request_id,
              success: result.success,
              error: result.error || null,
              data: result.data || null,
            },
          });
          ws.send(response);
        }
      } catch (e) {
        console.error('[Background] Error processing message:', e);
      }
    };
  } catch (error) {
    console.error('[Background] Failed to connect WebSocket:', error);
    setTimeout(connectWebSocket, 5000);
  }
}

// Start periodic context updates
function startContextUpdates() {
  if (contextInterval) return;

  contextInterval = setInterval(
    () => captureAndSendContext({ skipScreenshot: true }),
    CONTEXT_UPDATE_INTERVAL
  );
  captureAndSendContext({ skipScreenshot: true });
}

// Stop context updates
function stopContextUpdates() {
  if (contextInterval) {
    clearInterval(contextInterval);
    contextInterval = null;
  }
}

/**
 * Dispatches an action command to the active tab's content script.
 * If the content script is not loaded, it attempts to inject it.
 * @param {Object} command The action command from the backend
 * @returns {Promise<Object>} The ActionResult object
 */
async function dispatchToActiveTab(command) {
  try {
    // Get active tab
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });
    if (!tab || !tab.id) {
      return { success: false, error: 'No active tab found' };
    }

    // For navigation commands, we handle specially
    if (command.type === 'navigate_to') {
      try {
        // Send navigation command
        await chrome.tabs.sendMessage(tab.id, {
          action: 'execute',
          command: command,
        });
      } catch (e) {
        // Content script might not be loaded, try direct navigation via tabs API
        await chrome.tabs.update(tab.id, { url: command.url });
      }

      // Wait for page to finish loading (max 10 seconds)
      const waitForLoad = () =>
        new Promise((resolve) => {
          let resolved = false;
          const timeout = setTimeout(() => {
            if (!resolved) {
              resolved = true;
              resolve();
            }
          }, 10000);

          const listener = (tabId, changeInfo) => {
            if (tabId === tab.id && changeInfo.status === 'complete') {
              chrome.tabs.onUpdated.removeListener(listener);
              clearTimeout(timeout);
              if (!resolved) {
                resolved = true;
                resolve();
              }
            }
          };
          chrome.tabs.onUpdated.addListener(listener);
        });

      await waitForLoad();
      // Give the page a moment to fully render
      await new Promise((r) => setTimeout(r, 500));

      return { success: true, data: { navigated_to: command.url } };
    }

    // For other commands, send to content script with injection fallback
    return await sendToContentScript(tab.id, command);
  } catch (e) {
    return { success: false, error: e.message };
  }
}

/**
 * Sends a command to content script with injection fallback
 */
async function sendToContentScript(tabId, command, retries = 2) {
  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const result = await chrome.tabs.sendMessage(tabId, {
        action: 'execute',
        command: command,
      });
      return (
        result || { success: false, error: 'No response from content script' }
      );
    } catch (e) {
      try {
        await chrome.scripting.executeScript({
          target: { tabId: tabId },
          files: ['content.js'],
        });
        // Wait for the script to initialize
        await new Promise((resolve) =>
          setTimeout(resolve, 200 * (attempt + 1))
        );
      } catch (injectError) {
        if (attempt === retries) {
          return {
            success: false,
            error: `Could not inject content script: ${injectError.message}`,
          };
        }
      }
    }
  }

  return {
    success: false,
    error: 'Failed to communicate with content script after retries',
  };
}

// Capture current tab context and send to backend
async function captureAndSendContext(options = {}) {
  // Handle legacy boolean argument (backwards compatibility)
  const forceUpdate =
    typeof options === 'boolean' ? options : options.forceUpdate || false;
  const skipScreenshot =
    typeof options === 'object' ? options.skipScreenshot || false : false;
  const fullPage =
    typeof options === 'object' ? options.fullPage || false : false;

  if (!isConnected || !ws || ws.readyState !== WebSocket.OPEN) {
    return;
  }

  try {
    // Get active tab
    const [tab] = await chrome.tabs.query({
      active: true,
      currentWindow: true,
    });
    if (!tab || !tab.id) return;

    // Skip if same tab and URL (unless forced)
    if (!forceUpdate && tab.id === lastTabId && tab.url === lastUrl) {
      return;
    }

    lastTabId = tab.id;
    lastUrl = tab.url;

    // Get page content from content script
    let pageContent = null;
    try {
      const response = await chrome.tabs.sendMessage(tab.id, {
        action: 'getContext',
      });
      pageContent = response;
    } catch (e) {
      // Try to inject content script if not present
      try {
        await chrome.scripting.executeScript({
          target: { tabId: tab.id },
          files: ['content.js'],
        });
        // Retry after injection
        const response = await chrome.tabs.sendMessage(tab.id, {
          action: 'getContext',
        });
        pageContent = response;
      } catch {
        // Could not inject content script
      }
    }

    // Capture screenshot
    let screenshot = null;
    if (!skipScreenshot) {
      if (fullPage) {
        // Full page mode - try to capture entire page
        try {
          screenshot = await captureFullPage(tab.id);
        } catch {
          // Full page capture failed, will try viewport
        }
      }

      // If full page not requested or failed, capture viewport only
      if (!screenshot) {
        for (let attempt = 0; attempt < 3; attempt++) {
          try {
            if (attempt > 0)
              await new Promise((r) => setTimeout(r, 300 * attempt));
            screenshot = await chrome.tabs.captureVisibleTab(null, {
              format: 'jpeg',
              quality: 50,
            });
            if (screenshot) break;
          } catch {
            // Retry
          }
        }
      }
    }

    // Build context object
    const context = {
      type: 'context_update',
      timestamp: new Date().toISOString(),
      url: tab.url,
      title: tab.title,
      content: pageContent ? pageContent.text : null,
      screenshot: screenshot,
    };

    // Apply client-side privacy filter
    const sanitizedContext = sanitizeContext(context);

    // Send to backend
    ws.send(JSON.stringify(sanitizedContext));
  } catch (error) {
    console.error('[Background] Error capturing context:', error);
  }
}

// Client-side privacy filter
function sanitizeContext(context) {
  const emailRegex = /[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}/g;
  const creditCardRegex = /\b(?:\d[ -]*?){13,16}\b/g;
  const phoneRegex = /\b\d{3}[-.]?\d{3}[-.]?\d{4}\b/g;

  const sanitize = (text) => {
    if (!text) return text;
    return text
      .replace(emailRegex, '[EMAIL_REDACTED]')
      .replace(creditCardRegex, '[CC_REDACTED]')
      .replace(phoneRegex, '[PHONE_REDACTED]');
  };

  return {
    ...context,
    content: sanitize(context.content),
    title: sanitize(context.title),
  };
}

// Listen for tab activation (switching tabs)
chrome.tabs.onActivated.addListener((activeInfo) => {
  // Update context WITHOUT screenshot when switching tabs
  setTimeout(
    () => captureAndSendContext({ forceUpdate: true, skipScreenshot: true }),
    100
  );
});

// Listen for tab URL changes (navigation within tab)
chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.active) {
    // Update context WITHOUT screenshot when page loads
    setTimeout(
      () => captureAndSendContext({ forceUpdate: true, skipScreenshot: true }),
      500
    );
  }
});

// Listen for window focus changes
chrome.windows.onFocusChanged.addListener((windowId) => {
  if (windowId !== chrome.windows.WINDOW_ID_NONE) {
    // Update context WITHOUT screenshot when window focused
    setTimeout(
      () => captureAndSendContext({ forceUpdate: true, skipScreenshot: true }),
      100
    );
  }
});

// Listen for messages from popup/sidepanel
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'getConnectionStatus') {
    sendResponse({ connected: isConnected });
  } else if (message.action === 'getWsSessionId') {
    sendResponse({ sessionId: wsSessionId });
  } else if (message.action === 'forceContextUpdate') {
    const fullPage = message.fullPage || false;
    captureAndSendContext({ forceUpdate: true, fullPage }).then(() => {
      setTimeout(() => sendResponse({ success: true }), 100);
    });
    return true;
  } else if (message.action === 'updateContextNoScreenshot') {
    captureAndSendContext({ forceUpdate: true, skipScreenshot: true }).then(
      () => {
        setTimeout(() => sendResponse({ success: true }), 50);
      }
    );
    return true;
  }
  return true;
});

// Initialize connection when service worker starts
connectWebSocket();

/**
 * Forward action to sidepanel for UI display and execution.
 * Falls back to direct execution if sidepanel is not available.
 * @param {Object} command The action command from the backend
 * @returns {Promise<Object>} The ActionResult object
 */
async function forwardActionToSidepanel(command) {
  try {
    // Try to send to sidepanel first for UI display
    const result = await chrome.runtime.sendMessage({
      action: 'propose_action',
      data: command,
    });
    if (result) {
      return result;
    }
  } catch (e) {
    // Sidepanel might not be open, fall back to direct execution
    console.log('[Background] Sidepanel not available, executing directly');
  }

  // Fallback: execute directly without UI
  return await dispatchToActiveTab(command);
}

// --- Full Page Screenshot Logic ---

let offscreenCreating = null;

async function setupOffscreenDocument(path) {
  // Check if offscreen document exists
  const existingContexts = await chrome.runtime.getContexts({
    contextTypes: ['OFFSCREEN_DOCUMENT'],
    documentUrls: [chrome.runtime.getURL(path)],
  });

  if (existingContexts.length > 0) {
    return;
  }

  // Create offscreen document
  if (offscreenCreating) {
    await offscreenCreating;
  } else {
    offscreenCreating = chrome.offscreen.createDocument({
      url: path,
      reasons: ['BLOBS'],
      justification: 'Stitching full page screenshot',
    });
    await offscreenCreating;
    offscreenCreating = null;
  }
}

async function captureFullPage(tabId) {
  let originalScroll = null;
  try {
    originalScroll = await chrome.tabs.sendMessage(tabId, {
      action: 'getScrollPosition',
    });
  } catch {
    // Could not get original scroll position
  }

  let metrics = null;
  try {
    metrics = await chrome.tabs.sendMessage(tabId, { action: 'getMetrics' });
  } catch {
    return null;
  }

  if (!metrics) return null;

  if (metrics.height > 10000) {
    console.warn('[Background] Page too long (>10k px), using viewport only');
    return null;
  }

  if (metrics.height <= metrics.viewportHeight) {
    return await chrome.tabs.captureVisibleTab(null, {
      format: 'jpeg',
      quality: 50,
    });
  }

  try {
    await setupOffscreenDocument('offscreen.html');

    await chrome.runtime.sendMessage({
      target: 'offscreen',
      type: 'init',
      width: metrics.width,
      height: metrics.height,
    });

    let y = 0;
    const maxIterations = 50;
    let iterations = 0;

    while (y < metrics.height && iterations < maxIterations) {
      iterations++;

      await chrome.tabs.sendMessage(tabId, { action: 'scrollTo', x: 0, y: y });
      await new Promise((r) => setTimeout(r, 500)); // 500ms to avoid Chrome rate limit

      const dataUrl = await chrome.tabs.captureVisibleTab(null, {
        format: 'png',
      });

      await chrome.runtime.sendMessage({
        target: 'offscreen',
        type: 'draw',
        dataUrl,
        x: 0,
        y: y,
        width: metrics.viewportWidth,
        height: metrics.viewportHeight,
      });

      y += metrics.viewportHeight;
    }

    const response = await chrome.runtime.sendMessage({
      target: 'offscreen',
      type: 'getResult',
    });

    return response.result;
  } finally {
    const restoreX = originalScroll?.x || 0;
    const restoreY = originalScroll?.y || 0;
    try {
      await chrome.tabs.sendMessage(tabId, {
        action: 'scrollTo',
        x: restoreX,
        y: restoreY,
      });
    } catch {
      // Could not restore scroll position
    }
  }
}
