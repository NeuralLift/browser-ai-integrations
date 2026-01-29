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
function extractPageContent() {
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
  if (text.length > MAX_TEXT_LENGTH) {
    text = text.substring(0, MAX_TEXT_LENGTH) + '... [truncated]';
  }

  console.log('[Content] Extracted text length:', text.length);

  return {
    text: text || '',
    title: document.title,
    url: window.location.href,
  };
}

// Listen for messages from background script
chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message.action === 'getContext') {
    try {
      const content = extractPageContent();
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
  }
  return true;
});

console.log(
  '[Content] Browser AI Assistant content script loaded on:',
  window.location.hostname
);
