// Content script for Browser AI Assistant
// Runs on every page to extract context

// Maximum text length to capture (increased for fuller context)
const MAX_TEXT_LENGTH = 15000;

// Site-specific extractors for better content capture
const siteExtractors = {
  // WhatsApp Web - extract chat messages
  'web.whatsapp.com': () => {
    const messages = [];

    // Get chat header (who you're talking to)
    const header = document.querySelector('header span[title]');
    if (header) {
      messages.push(`Chat with: ${header.getAttribute('title')}`);
    }

    // Get messages - WhatsApp uses specific data attributes and classes
    const messageElements = document.querySelectorAll(
      '[data-pre-plain-text], .message-in, .message-out, [class*="message"]'
    );

    messageElements.forEach((el) => {
      // Try to get the pre-plain-text attribute (contains timestamp and sender)
      const prePlain = el.getAttribute('data-pre-plain-text');
      const text = el.innerText?.trim();

      if (text && text.length > 0 && text.length < 1000) {
        if (prePlain) {
          messages.push(`${prePlain} ${text}`);
        } else {
          messages.push(text);
        }
      }
    });

    // Also try to get any visible text in the main chat area
    const chatArea =
      document.querySelector('[data-tab="8"]') ||
      document.querySelector('.copyable-area') ||
      document.querySelector('[role="application"]');

    if (chatArea && messages.length < 5) {
      const allText = chatArea.innerText;
      if (allText) {
        messages.push('--- Chat Content ---');
        messages.push(allText);
      }
    }

    return messages.length > 0 ? messages.join('\n') : null;
  },

  // YouTube - extract video info
  'youtube.com': () => {
    const title = document.querySelector(
      'h1.ytd-video-primary-info-renderer, h1.title'
    )?.innerText;
    const channel = document.querySelector(
      '#channel-name a, .ytd-channel-name a'
    )?.innerText;
    const description = document.querySelector(
      '#description-inline-expander, #description'
    )?.innerText;

    let result = '';
    if (title) result += `Video: ${title}\n`;
    if (channel) result += `Channel: ${channel}\n`;
    if (description)
      result += `Description: ${description.substring(0, 1000)}\n`;

    return result || null;
  },

  // Twitter/X - extract tweets
  'twitter.com': () => extractTweets(),
  'x.com': () => extractTweets(),
};

function extractTweets() {
  const tweets = [];
  document.querySelectorAll('[data-testid="tweetText"]').forEach((el) => {
    tweets.push(el.innerText);
  });
  return tweets.length > 0 ? tweets.join('\n---\n') : null;
}

// Generic text extraction
function extractGenericContent() {
  const body = document.body;
  if (!body) return '';

  // Clone body and remove unwanted elements
  const clone = body.cloneNode(true);

  // Remove scripts, styles, and hidden elements
  const unwantedSelectors = [
    'script',
    'style',
    'noscript',
    'iframe',
    'svg',
    '[style*="display: none"]',
    '[style*="display:none"]',
    '[hidden]',
    '.hidden',
    '[aria-hidden="true"]',
    'header',
    'footer',
    'nav',
    '.sidebar',
    '#sidebar',
  ];

  unwantedSelectors.forEach((selector) => {
    try {
      clone.querySelectorAll(selector).forEach((el) => el.remove());
    } catch (e) {}
  });

  // Prefer main content areas
  const mainContent = clone.querySelector(
    'main, article, .content, #content, .post, .article'
  );
  const textSource = mainContent || clone;

  let text = textSource.innerText || textSource.textContent || '';

  // Clean up whitespace
  text = text
    .replace(/\t/g, ' ')
    .replace(/[ ]+/g, ' ')
    .replace(/\n\s*\n\s*\n/g, '\n\n')
    .trim();

  return text;
}

// Extract page content with site-specific handling
function extractPageContent(maxLength = MAX_TEXT_LENGTH) {
  let text = '';

  // Check for site-specific extractor
  const hostname = window.location.hostname.replace('www.', '');

  for (const [site, extractor] of Object.entries(siteExtractors)) {
    if (hostname.includes(site.replace('www.', ''))) {
      try {
        const siteContent = extractor();
        if (siteContent) {
          text = siteContent;
          console.log('[Content] Used site-specific extractor for:', site);
          break;
        }
      } catch (e) {
        console.error('[Content] Site extractor error:', e);
      }
    }
  }

  // Fallback to generic extraction
  if (!text) {
    text = extractGenericContent();
    console.log('[Content] Used generic extractor');
  }

  // Truncate if too long
  if (text.length > maxLength) {
    text = text.substring(0, maxLength) + '... [truncated]';
  }

  console.log('[Content] Extracted text length:', text.length);

  return {
    text: text || '',
    title: document.title,
    url: window.location.href,
  };
}

// --- DomTreeGenerator (Snapshot System) ---

// Store ref-to-element mapping from last snapshot
let refToElementMap = new Map();

const INTERACTIVE_SELECTORS = [
  'a[href]',
  'button',
  'input',
  'select',
  'textarea',
  '[role="button"]',
  '[role="link"]',
  '[role="checkbox"]',
  '[role="menuitem"]',
  '[role="tab"]',
  '[role="option"]',
  '[role="switch"]',
  '[role="gridcell"]',
  '[role="treeitem"]',
  '[role="listitem"]',
  '[tabindex]:not([tabindex="-1"])',
];

/**
 * Checks if an element is visible in the viewport and not hidden by styles
 */
function isElementVisible(el) {
  const rect = el.getBoundingClientRect();
  const style = window.getComputedStyle(el);

  const isVisible =
    style.display !== 'none' &&
    style.visibility !== 'hidden' &&
    style.opacity !== '0' &&
    rect.width > 0 &&
    rect.height > 0;

  if (!isVisible) return false;

  // Check if in viewport
  return (
    rect.top < window.innerHeight &&
    rect.bottom > 0 &&
    rect.left < window.innerWidth &&
    rect.right > 0
  );
}

/**
 * Determines if an element is interactive
 */
function isInteractive(el) {
  if (el.hasAttribute('data-browser-agent-ui')) return false;

  if (INTERACTIVE_SELECTORS.some((selector) => el.matches(selector))) {
    return true;
  }

  // Check for explicit onclick handler
  if (el.hasAttribute('onclick')) {
    return true;
  }

  // Also check for cursor: pointer as a fallback (careful with body/html)
  const style = window.getComputedStyle(el);
  if (
    style.cursor === 'pointer' &&
    el.tagName !== 'BODY' &&
    el.tagName !== 'HTML'
  ) {
    return true;
  }

  return false;
}

/**
 * Gets the accessible name of an element
 */
function getAccessibleName(el) {
  // Try aria-label
  let name = el.getAttribute('aria-label');
  if (name) return name.trim();

  // Try aria-labelledby
  const labelledBy = el.getAttribute('aria-labelledby');
  if (labelledBy) {
    const root = el.getRootNode();
    const labelEl =
      root && typeof root.getElementById === 'function'
        ? root.getElementById(labelledBy)
        : document.getElementById(labelledBy);

    if (labelEl) return labelEl.innerText.trim();
  }

  // Try alt (for images)
  name = el.getAttribute('alt');
  if (name) return name.trim();

  // Try title
  name = el.getAttribute('title');
  if (name) return name.trim();

  // Try placeholder (for inputs)
  name = el.getAttribute('placeholder');
  if (name) return name.trim();

  // Try value (for buttons)
  if (
    el.tagName === 'INPUT' &&
    (el.type === 'button' || el.type === 'submit')
  ) {
    name = el.value;
    if (name) return name.trim();
  }

  // Fallback to text content
  return el.innerText?.trim() || el.textContent?.trim() || '';
}

/**
 * Gets the ARIA role or implicit role of an element
 */
function getElementRole(el) {
  const role = el.getAttribute('role');
  if (role) return role;

  const tagName = el.tagName.toLowerCase();
  switch (tagName) {
    case 'a':
      return 'link';
    case 'button':
      return 'button';
    case 'input':
      if (el.type === 'checkbox') return 'checkbox';
      if (el.type === 'radio') return 'radio';
      return 'textbox';
    case 'textarea':
      return 'textbox';
    case 'select':
      return 'combobox';
    default:
      return tagName;
  }
}

/**
 * Gets the bounding box of an element relative to the viewport
 */
function getElementBounds(el) {
  const rect = el.getBoundingClientRect();
  return {
    x: Math.round(rect.left),
    y: Math.round(rect.top),
    width: Math.round(rect.width),
    height: Math.round(rect.height),
  };
}

/**
 * Generates a snapshot of the current page's interactive elements
 */
function generateSnapshot(limit = 300) {
  const tree = [];
  let refId = 1;
  refToElementMap.clear();

  function traverse(element) {
    if (!element || tree.length >= limit) return;

    // Skip our own UI
    if (element.hasAttribute('data-browser-agent-ui')) return;

    // Fast path: skip elements that are explicitly hidden
    const style = window.getComputedStyle(element);
    if (style.display === 'none' || style.visibility === 'hidden') return;

    if (isInteractive(element) && isElementVisible(element)) {
      const id = refId++;
      refToElementMap.set(id, element);
      tree.push({
        id: id,
        role: getElementRole(element),
        name: getAccessibleName(element),
        tag: element.tagName,
        bounds: getElementBounds(element),
      });
    }

    // Continue DFS even if current element is not interactive
    // (it might have interactive children)
    for (const child of element.children) {
      if (tree.length >= limit) break;
      traverse(child);
    }

    // Traverse Shadow DOM if present (open mode only)
    if (element.shadowRoot && tree.length < limit) {
      for (const child of element.shadowRoot.children) {
        if (tree.length >= limit) break;
        traverse(child);
      }
    }
  }

  traverse(document.body);
  return { tree };
}

// --- Visual Highlighting System ---

let activeHighlight = null;
let highlightTimeout = null;

function removeHighlight() {
  if (activeHighlight && activeHighlight.parentNode) {
    activeHighlight.parentNode.removeChild(activeHighlight);
  }
  activeHighlight = null;
  if (highlightTimeout) {
    clearTimeout(highlightTimeout);
    highlightTimeout = null;
  }
}

function highlightElement(ref) {
  // Remove any existing highlight
  removeHighlight();

  const element = refToElementMap.get(ref);
  if (!element) return false;

  const rect = element.getBoundingClientRect();

  // Create highlight overlay
  const highlight = document.createElement('div');
  highlight.setAttribute('data-browser-agent-ui', 'true');
  highlight.style.cssText = `
    position: fixed;
    top: ${rect.top}px;
    left: ${rect.left}px;
    width: ${rect.width}px;
    height: ${rect.height}px;
    border: 3px solid #ff0000;
    background-color: rgba(255, 0, 0, 0.1);
    pointer-events: none;
    z-index: 999999;
    box-sizing: border-box;
    border-radius: 4px;
  `;

  document.body.appendChild(highlight);
  activeHighlight = highlight;

  // Auto-remove after 2 seconds
  highlightTimeout = setTimeout(() => {
    removeHighlight();
  }, 2000);

  return true;
}

// --- Debug Mode Badges ---

let debugBadgeContainer = null;
let isDebugModeActive = false;
let debugRefreshTimeout = null;
let debugMutationObserver = null;

function showDebugBadges() {
  // Remove existing badges first
  hideDebugBadges(true); // true = keep debug mode active

  isDebugModeActive = true;

  // Generate fresh snapshot to populate refToElementMap
  generateSnapshot();

  // Create container
  debugBadgeContainer = document.createElement('div');
  debugBadgeContainer.setAttribute('data-browser-agent-ui', 'true');
  debugBadgeContainer.style.cssText =
    'position: fixed; top: 0; left: 0; width: 0; height: 0; z-index: 9999999; pointer-events: none;';
  document.body.appendChild(debugBadgeContainer);

  // Create badge for each element
  refToElementMap.forEach((element, id) => {
    // Check if element is still connected to DOM
    if (!element.isConnected) return;

    const rect = element.getBoundingClientRect();

    // Skip if element is not visible or has zero size (double check)
    if (rect.width === 0 || rect.height === 0) return;

    // Get element info for the label
    const name = getAccessibleName(element);
    const role = getElementRole(element);

    // Format: "[ID] name" or "[ID] role" if no name
    const shortName = name.length > 20 ? name.substring(0, 17) + '...' : name;
    const label = shortName ? `[${id}] ${shortName}` : `[${id}] ${role}`;

    const badge = document.createElement('div');
    badge.textContent = label;
    badge.style.cssText = `
      position: fixed;
      top: ${rect.top}px;
      left: ${rect.left}px;
      background: #dc2626;
      color: white;
      font-size: 10px;
      font-weight: bold;
      padding: 2px 4px;
      border-radius: 2px;
      pointer-events: none;
      font-family: monospace;
      line-height: 1;
      z-index: 9999999;
      box-shadow: 0 1px 2px rgba(0,0,0,0.2);
      max-width: 200px;
      white-space: nowrap;
      overflow: hidden;
      text-overflow: ellipsis;
    `;
    debugBadgeContainer.appendChild(badge);
  });

  console.log('[Content] Debug badges shown:', refToElementMap.size);

  // Setup real-time listeners if not already setup
  setupDebugListeners();
}

function hideDebugBadges(keepActive = false) {
  if (debugBadgeContainer && debugBadgeContainer.parentNode) {
    debugBadgeContainer.parentNode.removeChild(debugBadgeContainer);
  }
  debugBadgeContainer = null;

  if (!keepActive) {
    isDebugModeActive = false;
    cleanupDebugListeners();
    console.log('[Content] Debug badges hidden');
  }
}

/**
 * Debounced refresh of debug badges
 */
function refreshDebugBadges() {
  if (!isDebugModeActive) return;

  // Debounce to avoid too many refreshes
  clearTimeout(debugRefreshTimeout);
  debugRefreshTimeout = setTimeout(() => {
    if (isDebugModeActive) {
      showDebugBadges();
    }
  }, 150);
}

/**
 * Setup listeners for real-time debug badge updates
 */
function setupDebugListeners() {
  // Remove existing listeners first
  cleanupDebugListeners();

  // Scroll listener - badges need repositioning
  window.addEventListener('scroll', refreshDebugBadges, { passive: true });

  // Resize listener - viewport changes
  window.addEventListener('resize', refreshDebugBadges, { passive: true });

  // DOM mutation observer - new elements added/removed
  debugMutationObserver = new MutationObserver((mutations) => {
    // Only refresh if there are actual structural changes
    const hasRelevantChange = mutations.some(
      (m) =>
        m.type === 'childList' ||
        (m.type === 'attributes' &&
          (m.attributeName === 'style' ||
            m.attributeName === 'class' ||
            m.attributeName === 'hidden'))
    );
    if (hasRelevantChange) {
      refreshDebugBadges();
    }
  });

  debugMutationObserver.observe(document.body, {
    childList: true,
    subtree: true,
    attributes: true,
    attributeFilter: ['style', 'class', 'hidden', 'disabled'],
  });

  console.log('[Content] Debug listeners setup for real-time updates');
}

/**
 * Cleanup debug listeners
 */
function cleanupDebugListeners() {
  window.removeEventListener('scroll', refreshDebugBadges);
  window.removeEventListener('resize', refreshDebugBadges);

  if (debugMutationObserver) {
    debugMutationObserver.disconnect();
    debugMutationObserver = null;
  }

  clearTimeout(debugRefreshTimeout);
}

/**
 * Executes a browser action command
 */
function executeAction(command) {
  try {
    switch (command.type) {
      case 'navigate_to':
        window.location.assign(command.url);
        return { success: true };

      case 'click_element': {
        highlightElement(command.ref);
        const element = refToElementMap.get(command.ref);
        if (!element) {
          return {
            success: false,
            error: `Element with ref ${command.ref} not found`,
          };
        }
        element.click();
        return { success: true };
      }

      case 'type_text': {
        highlightElement(command.ref);
        const element = refToElementMap.get(command.ref);
        if (!element) {
          return {
            success: false,
            error: `Element with ref ${command.ref} not found`,
          };
        }
        element.focus();
        element.value = command.text;
        element.dispatchEvent(new Event('input', { bubbles: true }));
        element.dispatchEvent(new Event('change', { bubbles: true }));
        return { success: true };
      }

      case 'scroll_to':
        window.scrollTo(command.x, command.y);
        return { success: true };

      default:
        return {
          success: false,
          error: `Unknown action type: ${command.type}`,
        };
    }
  } catch (e) {
    return { success: false, error: e.message };
  }
}

// Listen for messages from background script

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'getContext' || message.action === 'getContent') {
    try {
      const maxLength = message.maxLength || 15000;
      const content = extractPageContent(maxLength);
      console.log(
        '[Content] Sending context, text length:',
        content.text.length
      );
      sendResponse(content);
    } catch (e) {
      console.error('[Content] Error extracting content:', e);
      sendResponse({
        text: '',
        title: document.title,
        url: window.location.href,
      });
    }
  } else if (message.action === 'getSnapshot') {
    try {
      const limit = message.limit || 300;
      const snapshot = generateSnapshot(limit);
      console.log(
        '[Content] Sending snapshot, elements found:',
        snapshot.tree.length
      );
      sendResponse(snapshot);
    } catch (e) {
      console.error('[Content] Error generating snapshot:', e);
      sendResponse({ tree: [] });
    }
  } else if (message.action === 'getMetrics') {
    sendResponse({
      width: document.documentElement.scrollWidth,
      height: document.documentElement.scrollHeight,
      viewportWidth: window.innerWidth,
      viewportHeight: window.innerHeight,
      devicePixelRatio: window.devicePixelRatio,
    });
  } else if (message.action === 'getScrollPosition') {
    sendResponse({
      x: window.scrollX,
      y: window.scrollY,
    });
  } else if (message.action === 'scrollTo') {
    window.scrollTo(message.x, message.y);
    sendResponse({ success: true });
  } else if (message.action === 'highlight') {
    const success = highlightElement(message.ref);
    sendResponse({ success });
  } else if (message.action === 'execute') {
    const result = executeAction(message.command);
    sendResponse(result);
  } else if (message.action === 'toggleDebug') {
    if (message.value) {
      showDebugBadges();
    } else {
      hideDebugBadges();
    }
    sendResponse({ success: true });
  }
  return true;
});

console.log(
  '[Content] Browser AI Assistant content script loaded on:',
  window.location.hostname
);
