// Side Panel script for Browser AI Assistant

// Simple Markdown parser using marked + DOMPurify + highlight.js
function parseMarkdown(text) {
  if (!text) return '';

  // Fallback if libraries are not loaded
  if (typeof marked === 'undefined' || typeof DOMPurify === 'undefined') {
    console.warn('Markdown libraries not loaded');
    return text; 
  }

  // Configure marked if not already configured
  if (!window.markedConfigured && typeof hljs !== 'undefined') {
    marked.setOptions({
      highlight: function(code, lang) {
        const language = hljs.getLanguage(lang) ? lang : 'plaintext';
        return hljs.highlight(code, { language }).value;
      },
      langPrefix: 'hljs language-',
      breaks: true,
      gfm: true
    });
    window.markedConfigured = true;
  }

  // Parse
  let html = marked.parse(text);

  // Replace Emojis
  html = replaceEmojis(html);

  // Sanitize
  html = DOMPurify.sanitize(html);

  return html;
}

// Basic Emoji Replacement (Common subset)
function replaceEmojis(text) {
  const emojiMap = {
    ':smile:': '😄', ':smiley:': '😃', ':grinning:': '😀', ':blush:': '😊', ':wink:': '😉',
    ':heart_eyes:': '😍', ':kissing_heart:': '😘', ':stuck_out_tongue:': '😛',
    ':sunglasses:': '😎', ':nerd_face:': '🤓', ':thinking_face:': '🤔',
    ':neutral_face:': '😐', ':expressionless:': '😑', ':no_mouth:': '😶',
    ':smirk:': '😏', ':persevere:': '😣', ':disappointed_relieved:': '😥',
    ':open_mouth:': '😮', ':zipper_mouth_face:': '🤐', ':hushed:': '😯',
    ':sleepy:': '😪', ':tired_face:': '😫', ':sleeping:': '😴', ':relieved:': '😌',
    ':stuck_out_tongue_winking_eye:': '😜', ':stuck_out_tongue_closed_eyes:': '😝',
    ':drooling_face:': '🤤', ':unamused:': '😒', ':sweat:': '😓', ':pensive:': '😔',
    ':confused:': '😕', ':upside_down_face:': '🙃', ':money_mouth_face:': '🤑',
    ':astonished:': '😲', ':frowning:': 'frowning', ':slight_frown:': '🙁',
    ':confounded:': '😖', ':disappointed:': '😞', ':worried:': '😟',
    ':triumph:': '😤', ':cry:': '😢', ':sob:': '😭', ':frowning_face:': '😦',
    ':anguished:': '😧', ':fearful:': '😨', ':weary:': '😩', ':exploding_head:': '🤯',
    ':grimacing:': '😬', ':anxious_face_with_sweat:': '😰', ':scream:': '😱',
    ':flushed:': '😳', ':dizzy_face:': '😵', ':rage:': '😡', ':angry:': '😠',
    ':mask:': '😷', ':thermometer_face:': '🤒', ':head_bandage:': '🤕',
    ':nauseated_face:': '🤢', ':sneezing_face:': '🤧', ':innocent:': '😇',
    ':cowboy_hat_face:': '🤠', ':clown_face:': '🤡', ':lying_face:': '🤥',
    ':shushing_face:': '🤫', ':hand_over_mouth:': '🤭', ':monocle_face:': '🧐',
    ':thumbsup:': '👍', ':thumbsdown:': '👎', ':ok_hand:': '👌', ':point_up:': '☝️',
    ':point_down:': '👇', ':point_left:': '👈', ':point_right:': '👉',
    ':raised_hands:': '🙌', ':pray:': '🙏', ':clap:': '👏', ':muscle:': '💪',
    ':metal:': '🤘', ':fu:': '🖕', ':top:': '🔝', ':soon:': '🔜',
    ':on:': '🔛', ':end:': '🔚', ':back:': '🔙', ':fire:': '🔥',
    ':rocket:': '🚀', ':sparkles:': '✨', ':star:': '⭐', ':heart:': '❤️',
    ':broken_heart:': '💔', ':warning:': '⚠️', ':check:': '✅', ':x:': '❌',
    ':question:': '❓', ':exclamation:': '❗', ':bulb:': '💡', ':zzz:': '💤'
  };

  return text.replace(/:[a-z0-9_]+:/g, (match) => {
    return emojiMap[match] || match;
  });
}

// Enhance code blocks with copy button and language label
function enhanceCodeBlocks(container) {
  const pres = container.querySelectorAll('pre');
  pres.forEach(pre => {
    // Check if already processed
    if (pre.parentNode.classList.contains('code-block-wrapper')) return;

    const code = pre.querySelector('code');
    let lang = 'text';
    if (code) {
        // hljs adds class like 'hljs language-javascript'
        const classes = code.className.split(' ');
        const langClass = classes.find(c => c.startsWith('language-'));
        if (langClass) lang = langClass.replace('language-', '');
    }

    // Create wrapper
    const wrapper = document.createElement('div');
    wrapper.className = 'code-block-wrapper';
    pre.parentNode.insertBefore(wrapper, pre);
    
    // Create Header
    const header = document.createElement('div');
    header.className = 'code-header';
    
    const langSpan = document.createElement('span');
    langSpan.textContent = lang;
    
    const copyBtn = document.createElement('button');
    copyBtn.className = 'copy-btn';
    copyBtn.innerHTML = `
        <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
        Copy
    `;
    
    copyBtn.addEventListener('click', async () => {
        const text = code ? code.innerText : pre.innerText;
        try {
            await navigator.clipboard.writeText(text);
            copyBtn.innerHTML = `
                <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"></polyline></svg>
                Copied!
            `;
            copyBtn.classList.add('copied');
            setTimeout(() => {
                copyBtn.innerHTML = `
                    <svg viewBox="0 0 24 24" width="14" height="14" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"></rect><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"></path></svg>
                    Copy
                `;
                copyBtn.classList.remove('copied');
            }, 2000);
        } catch (err) {
            console.error('Failed to copy', err);
        }
    });

    header.appendChild(langSpan);
    header.appendChild(copyBtn);
    
    wrapper.appendChild(header);
    wrapper.appendChild(pre);
  });
}

// Utils
function generateId() {
  return Date.now().toString(36) + Math.random().toString(36).substr(2);
}

// Session Manager
const SessionManager = {
  getSessions() {
    try {
      return JSON.parse(localStorage.getItem('ai-sessions') || '[]');
    } catch (e) {
      console.error('Error parsing sessions:', e);
      return [];
    }
  },

  saveSessions(sessions) {
    try {
      localStorage.setItem('ai-sessions', JSON.stringify(sessions));
    } catch (e) {
      console.error('Error saving sessions:', e);
    }
  },

  getCurrentSessionId() {
    return localStorage.getItem('ai-current-session-id');
  },

  setCurrentSessionId(id) {
    localStorage.setItem('ai-current-session-id', id);
  },

  createSession() {
    const sessions = this.getSessions();
    const newSession = {
      id: generateId(),
      name: 'Chat Baru',
      messages: [],
      createdAt: Date.now(),
      customInstruction: ''
    };

    // Limit to 10 sessions
    if (sessions.length >= 10) {
      sessions.sort((a, b) => a.createdAt - b.createdAt);
      sessions.shift(); // Remove oldest
    }

    sessions.push(newSession);
    this.saveSessions(sessions);
    this.setCurrentSessionId(newSession.id);
    return newSession;
  },

  getSession(id) {
    const sessions = this.getSessions();
    return sessions.find(s => s.id === id);
  },

  updateSession(id, updates) {
    const sessions = this.getSessions();
    const index = sessions.findIndex(s => s.id === id);
    if (index !== -1) {
      sessions[index] = { ...sessions[index], ...updates };
      this.saveSessions(sessions);
      return sessions[index];
    }
    return null;
  },

  deleteSession(id) {
    let sessions = this.getSessions();
    sessions = sessions.filter(s => s.id !== id);
    this.saveSessions(sessions);
  },

  addMessageToSession(id, message) {
    const session = this.getSession(id);
    if (session) {
      session.messages.push(message);

      // Auto-name if first user message and name is default
      if (message.role === 'user' && session.name === 'Chat Baru') {
        let name = message.text.slice(0, 30);
        if (message.text.length > 30) name += '...';
        session.name = name;
      }

      this.updateSession(id, session);
    }
  }
};

// Settings Manager
const SettingsManager = {
  getSettings() {
    try {
      return JSON.parse(localStorage.getItem('ai-settings') || JSON.stringify({
        globalInstruction: '',
        screenshotDefault: false
      }));
    } catch (e) {
      console.error('Error parsing settings:', e);
      return { globalInstruction: '', screenshotDefault: false };
    }
  },

  saveSettings(settings) {
    try {
      localStorage.setItem('ai-settings', JSON.stringify(settings));
    } catch (e) {
      console.error('Error saving settings:', e);
    }
  }
};

// Wait for DOM to be ready
document.addEventListener('DOMContentLoaded', () => {
  const chatContainer = document.getElementById('chat-container');
  const messageInput = document.getElementById('message-input');
  const sendBtn = document.getElementById('send-btn');
  const statusDot = document.getElementById('status-dot');
  const pageTitle = document.getElementById('page-title');
  const refreshBtn = document.getElementById('refresh-context');
  const themeToggle = document.getElementById('theme-toggle');
  const themeIconLight = document.getElementById('theme-icon-light');
  const themeIconDark = document.getElementById('theme-icon-dark');

  // New UI Elements
  const newChatBtn = document.getElementById('new-chat-btn');
  const sessionSelectBtn = document.getElementById('session-select-btn');
  const currentSessionNameEl = document.getElementById('current-session-name');
  const settingsBtn = document.getElementById('settings-btn');
  const screenshotToggle = document.getElementById('screenshot-toggle');

  // Modals
  const settingsModal = document.getElementById('settings-modal');
  const sessionModal = document.getElementById('session-modal');
  const deleteModal = document.getElementById('delete-modal');
  const closeSettingsBtn = document.getElementById('close-settings');
  const closeSessionsBtn = document.getElementById('close-sessions');
  const closeDeleteBtn = document.getElementById('close-delete');
  const saveSettingsBtn = document.getElementById('save-settings-btn');
  const confirmDeleteBtn = document.getElementById('confirm-delete-btn');
  const cancelDeleteBtn = document.getElementById('cancel-delete-btn');
  const sessionListEl = document.getElementById('session-list');
  const quickActionsEl = document.querySelector('.quick-actions');
  
  let sessionToDelete = null; // Track which session to delete

  // Inputs in Settings
  const globalInstructionInput = document.getElementById('global-instruction');
  const sessionInstructionInput = document.getElementById('session-instruction');
  const settingScreenshotDefault = document.getElementById('setting-screenshot-default');

  // Input & Attach Elements
  const imageUploadInput = document.getElementById('image-upload');
  const imagePreviewDiv = document.getElementById('image-preview');
  const previewImg = document.getElementById('preview-img');
  const clearImageBtn = document.getElementById('clear-image');
  const toolsBtn = document.getElementById('tools-btn');
  const toolsMenu = document.getElementById('tools-menu');
  const menuUploadImage = document.getElementById('menu-upload-image');

  let isProcessing = false;
  let currentSession = null;
  let currentImage = null; // Base64 string

  // Theme management
  function getTheme() {
    return localStorage.getItem('ai-assistant-theme') || 'dark';
  }

  function setTheme(theme) {
    document.body.setAttribute('data-theme', theme);
    localStorage.setItem('ai-assistant-theme', theme);

    if (theme === 'dark') {
      themeIconLight.style.display = 'none';
      themeIconDark.style.display = 'block';
    } else {
      themeIconLight.style.display = 'block';
      themeIconDark.style.display = 'none';
    }
  }

  function toggleTheme() {
    const current = getTheme();
    setTheme(current === 'dark' ? 'light' : 'dark');
  }

  // Initialize theme
  setTheme(getTheme());

  themeToggle.addEventListener('click', toggleTheme);

  // Update connection status
  function updateStatus(connected) {
    if (connected) {
      statusDot.classList.add('connected');
    } else {
      statusDot.classList.remove('connected');
    }
  }

  // Update page context display
  function updatePageInfo(title, url) {
    if (title && pageTitle) {
      pageTitle.textContent = title;
      pageTitle.title = url || '';
    } else if (pageTitle) {
      pageTitle.textContent = 'Tidak ada halaman terdeteksi';
    }
  }

  // Add message to chat UI
  function renderMessage(message, autoScroll = true) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `message ${message.role === 'user' ? 'user' : 'assistant'}`;

    const bubbleDiv = document.createElement('div');
    bubbleDiv.className = 'bubble';

    if (message.role === 'user') {
      if (message.image) {
        const img = document.createElement('img');
        img.src = message.image;
        img.style.maxHeight = '150px';
        img.style.maxWidth = '100%';
        img.style.display = 'block';
        img.style.marginBottom = '8px';
        img.style.borderRadius = '4px';
        bubbleDiv.appendChild(img);
      }
      const textSpan = document.createElement('div');
      textSpan.textContent = message.text;
      bubbleDiv.appendChild(textSpan);
    } else {
      bubbleDiv.innerHTML = parseMarkdown(message.text);

      // Render Math (KaTeX)
      if (typeof renderMathInElement !== 'undefined') {
        renderMathInElement(bubbleDiv, {
          delimiters: [
            {left: '$$', right: '$$', display: true},
            {left: '$', right: '$', display: false},
            {left: '\\(', right: '\\)', display: false},
            {left: '\\[', right: '\\]', display: true}
          ],
          throwOnError: false
        });
      }

      // Enhance Code Blocks (Copy button, etc)
      enhanceCodeBlocks(bubbleDiv);

      // Token display
      if (message.tokens) {
        const tokenDiv = document.createElement('div');
        tokenDiv.style.fontSize = '10px';
        tokenDiv.style.color = 'var(--text-muted)';
        tokenDiv.style.marginTop = '4px';
        tokenDiv.style.textAlign = 'right';
        tokenDiv.textContent = `Tokens: ${message.tokens.prompt || 0} in / ${message.tokens.response || 0} out`;
        bubbleDiv.appendChild(tokenDiv);
      }
    }

    messageDiv.appendChild(bubbleDiv);
    chatContainer.appendChild(messageDiv);
    
    if (autoScroll) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }

    // Hide quick actions if messages exist
    if (quickActionsEl) {
      quickActionsEl.style.display = 'none';
    }
  }

  function clearChat() {
    chatContainer.innerHTML = '';
    // Show quick actions when cleared
    if (quickActionsEl) {
      quickActionsEl.style.display = 'flex';
    }
  }

  // Initialize Session
  function initSession() {
    const sessions = SessionManager.getSessions();
    const currentId = SessionManager.getCurrentSessionId();
    const settings = SettingsManager.getSettings();

    // Restore screenshot default
    screenshotToggle.checked = settings.screenshotDefault;

    if (currentId) {
      currentSession = SessionManager.getSession(currentId);
    }

    if (!currentSession) {
      if (sessions.length > 0) {
        // Fallback to last session
        currentSession = sessions[sessions.length - 1];
        SessionManager.setCurrentSessionId(currentSession.id);
      } else {
        // Create new
        currentSession = SessionManager.createSession();
      }
    }

    updateSessionUI();
  }

  function updateSessionUI() {
    if (!currentSession) return;

    currentSessionNameEl.textContent = currentSession.name;
    clearChat();

    // If we have messages, hide quick actions
    if (currentSession.messages.length > 0) {
      if (quickActionsEl) quickActionsEl.style.display = 'none';
    } else {
      if (quickActionsEl) quickActionsEl.style.display = 'flex';
    }

    // Render messages without auto-scroll for each
    currentSession.messages.forEach(msg => renderMessage(msg, false));
    
    // Scroll to bottom once after all messages rendered
    if (currentSession.messages.length > 0) {
      chatContainer.scrollTop = chatContainer.scrollHeight;
    }

    // Update session instruction input in settings
    sessionInstructionInput.value = currentSession.customInstruction || '';
  }

  function switchSession(id) {
    currentSession = SessionManager.getSession(id);
    SessionManager.setCurrentSessionId(id);
    updateSessionUI();
    closeModal(sessionModal);
  }

  function createNewSession() {
    currentSession = SessionManager.createSession();
    updateSessionUI();

    // Reset screenshot toggle to default
    const settings = SettingsManager.getSettings();
    screenshotToggle.checked = settings.screenshotDefault;
  }

  // Show typing indicator
  function showTyping() {
    const messageDiv = document.createElement('div');
    messageDiv.className = 'message assistant typing';
    messageDiv.id = 'typing-indicator';

    const bubbleDiv = document.createElement('div');
    bubbleDiv.className = 'bubble';
    bubbleDiv.textContent = 'Berpikir';

    messageDiv.appendChild(bubbleDiv);
    chatContainer.appendChild(messageDiv);
    chatContainer.scrollTop = chatContainer.scrollHeight;
  }

  // Remove typing indicator
  function hideTyping() {
    const typing = document.getElementById('typing-indicator');
    if (typing) typing.remove();
  }

  // Send message to backend
  async function sendMessage(messageText = null) {
    let text = messageText || messageInput.value.trim();
    
    // Allow sending if there is text OR an image
    // If only image is present, add default text
    if ((!text && !currentImage) || isProcessing) return;

    if (!text && currentImage) {
      text = "Jelaskan gambar ini";
    }

    // Ensure we have a valid session
    if (!currentSession) {
      currentSession = SessionManager.createSession();
      updateSessionUI();
    }

    isProcessing = true;

    // Add user message to UI and Session
    const userMsg = { 
      role: 'user', 
      text: text, 
      image: currentImage,
      timestamp: Date.now() 
    };
    renderMessage(userMsg);
    SessionManager.addMessageToSession(currentSession.id, userMsg);
    
    // Refresh session from storage to get updated name
    currentSession = SessionManager.getSession(currentSession.id);
    currentSessionNameEl.textContent = currentSession.name;

    messageInput.value = '';
    sendBtn.disabled = true;

    showTyping();

    try {
      // Handle screenshot toggle
      if (screenshotToggle.checked) {
        console.log('[Sidepanel] Capturing context with screenshot...');
        await chrome.runtime.sendMessage({ action: 'forceContextUpdate' });
        await new Promise(r => setTimeout(r, 200));
      } else {
        console.log('[Sidepanel] Skipping screenshot capture (toggle OFF)');
        await chrome.runtime.sendMessage({ action: 'updateContextNoScreenshot' });
        await new Promise(r => setTimeout(r, 200));
      }

      // Prepare Custom Instruction
      const settings = SettingsManager.getSettings();
      let instruction = settings.globalInstruction;
      if (currentSession.customInstruction) {
        instruction = currentSession.customInstruction;
      }

      const response = await fetch('http://localhost:3000/api/chat', {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          message: text,
          custom_instruction: instruction || undefined,
          image: currentImage || undefined
        }),
      });

      // Clear image after sending
      if (currentImage) clearImage();

      hideTyping();

      if (!response.ok) {
        throw new Error('Failed to get response');
      }

      const data = await response.json();

      const assistantMsg = {
        role: 'assistant',
        text: data.response || 'Tidak ada respons',
        timestamp: Date.now(),
        tokens: {
          prompt: data.prompt_tokens,
          response: data.response_tokens,
          total: data.total_tokens
        }
      };

      renderMessage(assistantMsg);
      SessionManager.addMessageToSession(currentSession.id, assistantMsg);
      updateStatus(true);

    } catch (error) {
      hideTyping();
      console.error('Error sending message:', error);
      const errorMsg = { role: 'assistant', text: '**Error:** Tidak bisa terhubung ke backend. Pastikan server berjalan di `localhost:3000`', timestamp: Date.now() };
      renderMessage(errorMsg);
      updateStatus(false);
    } finally {
      sendBtn.disabled = false;
      isProcessing = false;
      messageInput.focus();
    }
  }

  // Get and display current tab info
  async function refreshTabInfo() {
    try {
      const [tab] = await chrome.tabs.query({ active: true, currentWindow: true });
      if (tab && tab.title) {
        updatePageInfo(tab.title, tab.url);
      }
    } catch (e) {
      console.error('[Sidepanel] Error getting tab info:', e);
    }
  }

  // Check backend connection and get current tab info
  async function initialize() {
    console.log('[Sidepanel] Initializing...');

    // Check backend health
    await checkConnectionStatus();

    initSession();
    await refreshTabInfo();
    
    // Start periodic connection check (every 5 seconds)
    setInterval(checkConnectionStatus, 5000);
  }
  
  // Check connection status
  async function checkConnectionStatus() {
    try {
      const response = await fetch('http://localhost:3000/health', { 
        method: 'GET',
        signal: AbortSignal.timeout(3000) // 3 second timeout
      });
      updateStatus(response.ok);
    } catch {
      updateStatus(false);
    }
  }

  // Event listeners
  sendBtn.addEventListener('click', () => {
    sendMessage();
    messageInput.style.height = '';
  });

  document.querySelectorAll('.quick-btn').forEach(btn => {
    btn.addEventListener('click', () => {
      const prompt = btn.getAttribute('data-prompt');
      if (prompt) {
        sendMessage(prompt);
      }
    });
  });

  // Refresh context button
  if (refreshBtn) {
    refreshBtn.addEventListener('click', async () => {
      refreshBtn.classList.add('spinning');

      try {
        const action = screenshotToggle.checked ? 'forceContextUpdate' : 'updateContextNoScreenshot';
        await chrome.runtime.sendMessage({ action });
        await refreshTabInfo();
        await new Promise(r => setTimeout(r, 500));
      } catch (e) {
        console.error('Error refreshing context:', e);
      }

      refreshBtn.classList.remove('spinning');
    });
  }

  // Modal Logic
  function openModal(modal) {
    modal.classList.add('show');
  }

  function closeModal(modal) {
    modal.classList.remove('show');
  }

  window.onclick = function(event) {
    if (event.target.classList.contains('modal')) {
      closeModal(event.target);
    }
  }

  // Settings Modal
  if (settingsBtn) {
    settingsBtn.addEventListener('click', () => {
      console.log('Settings button clicked');
      try {
        const settings = SettingsManager.getSettings();
        globalInstructionInput.value = settings.globalInstruction || '';
        // Sync modal toggle with current dropdown state
        settingScreenshotDefault.checked = screenshotToggle.checked;
        sessionInstructionInput.value = (currentSession && currentSession.customInstruction) ? currentSession.customInstruction : '';
        openModal(settingsModal);
      } catch (e) {
        console.error('Error opening settings:', e);
      }
    });
  } else {
    console.error('Settings button not found');
  }

  closeSettingsBtn.addEventListener('click', () => closeModal(settingsModal));

  saveSettingsBtn.addEventListener('click', () => {
    const settings = {
      globalInstruction: globalInstructionInput.value,
      screenshotDefault: settingScreenshotDefault.checked
    };
    SettingsManager.saveSettings(settings);

    // Sync dropdown toggle with the new default setting
    screenshotToggle.checked = settingScreenshotDefault.checked;

    // Save session specific instruction
    if (currentSession) {
      currentSession.customInstruction = sessionInstructionInput.value;
      SessionManager.updateSession(currentSession.id, currentSession);
    }

    closeModal(settingsModal);
  });

  // Session Modal
  sessionSelectBtn.addEventListener('click', () => {
    renderSessionList();
    openModal(sessionModal);
  });

  closeSessionsBtn.addEventListener('click', () => closeModal(sessionModal));

  // Delete Confirmation Modal
  closeDeleteBtn.addEventListener('click', () => {
    sessionToDelete = null;
    closeModal(deleteModal);
  });

  cancelDeleteBtn.addEventListener('click', () => {
    sessionToDelete = null;
    closeModal(deleteModal);
  });

  confirmDeleteBtn.addEventListener('click', () => {
    if (sessionToDelete) {
      SessionManager.deleteSession(sessionToDelete);
      if (currentSession && sessionToDelete === currentSession.id) {
        initSession();
      }
      renderSessionList();
      sessionToDelete = null;
    }
    closeModal(deleteModal);
  });

  newChatBtn.addEventListener('click', () => {
    createNewSession();
  });

  function renderSessionList() {
    const sessions = SessionManager.getSessions().reverse(); // Newest first
    sessionListEl.innerHTML = '';

    if (sessions.length === 0) {
      sessionListEl.innerHTML = '<div style="padding: 10px; color: var(--text-muted); text-align: center;">Belum ada riwayat chat</div>';
      return;
    }

    sessions.forEach(session => {
      const el = document.createElement('div');
      el.style.padding = '10px';
      el.style.border = '1px solid var(--border)';
      el.style.borderRadius = '6px';
      el.style.cursor = 'pointer';
      el.style.backgroundColor = session.id === currentSession.id ? 'var(--bg-tertiary)' : 'var(--bg-primary)';
      el.style.display = 'flex';
      el.style.justifyContent = 'space-between';
      el.style.alignItems = 'center';

      const title = document.createElement('div');
      title.style.fontWeight = '500';
      title.textContent = session.name;

      const date = document.createElement('div');
      date.style.fontSize = '11px';
      date.style.color = 'var(--text-muted)';
      date.textContent = new Date(session.createdAt).toLocaleDateString();

      const info = document.createElement('div');
      info.appendChild(title);
      info.appendChild(date);

      const deleteBtn = document.createElement('button');
      deleteBtn.innerHTML = '&times;';
      deleteBtn.style.background = 'none';
      deleteBtn.style.border = 'none';
      deleteBtn.style.fontSize = '18px';
      deleteBtn.style.cursor = 'pointer';
      deleteBtn.style.color = 'var(--text-muted)';
      deleteBtn.style.padding = '0 5px';

      deleteBtn.onclick = (e) => {
        e.stopPropagation();
        sessionToDelete = session.id;
        openModal(deleteModal);
      };

      el.onclick = () => switchSession(session.id);

      el.appendChild(info);
      el.appendChild(deleteBtn);
      sessionListEl.appendChild(el);
    });
  }

  // Image Handling
  function handleImageSelect(file) {
    if (!file || !file.type.startsWith('image/')) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      // Compress image
      const img = new Image();
      img.onload = () => {
        const canvas = document.createElement('canvas');
        const ctx = canvas.getContext('2d');

        // Resize if too large (max 1024px)
        let width = img.width;
        let height = img.height;
        const maxSize = 1024;

        if (width > maxSize || height > maxSize) {
          if (width > height) {
            height = Math.round((height * maxSize) / width);
            width = maxSize;
          } else {
            width = Math.round((width * maxSize) / height);
            height = maxSize;
          }
        }

        canvas.width = width;
        canvas.height = height;
        ctx.drawImage(img, 0, 0, width, height);

        // Compress to JPEG 80%
        currentImage = canvas.toDataURL('image/jpeg', 0.8);
        previewImg.src = currentImage;
        imagePreviewDiv.style.display = 'block';
      };
      img.src = e.target.result;
    };
    reader.readAsDataURL(file);
  }

  function clearImage() {
    currentImage = null;
    imagePreviewDiv.style.display = 'none';
    imageUploadInput.value = '';
  }

  // Attach Menu Logic
  toolsBtn.addEventListener('click', (e) => {
    e.stopPropagation();
    toolsMenu.classList.toggle('show');
  });

  // Close menu when clicking outside
  document.addEventListener('click', (e) => {
    if (!toolsMenu.contains(e.target) && e.target !== toolsBtn) {
      toolsMenu.classList.remove('show');
    }
  });

  menuUploadImage.addEventListener('click', () => {
    imageUploadInput.click();
    toolsMenu.classList.remove('show');
  });

  screenshotToggle.addEventListener('click', (e) => {
    e.stopPropagation();
  });

  imageUploadInput.addEventListener('change', (e) => {
    if (e.target.files.length > 0) {
      handleImageSelect(e.target.files[0]);
    }
  });

  clearImageBtn.addEventListener('click', clearImage);

  // Auto-resize textarea
  messageInput.addEventListener('input', function() {
    this.style.height = 'auto';
    this.style.height = (this.scrollHeight) + 'px';
    if (this.value === '') this.style.height = '';
  });

  // Handle Enter key for textarea
  messageInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      sendMessage();
      // Reset height
      messageInput.style.height = '';
    }
  });

  // Drag and Drop
  document.addEventListener('dragover', (e) => {
    e.preventDefault();
  });

  document.addEventListener('drop', (e) => {
    e.preventDefault();
    if (e.dataTransfer.files.length > 0) {
      handleImageSelect(e.dataTransfer.files[0]);
    }
  });

  // Listen for tab changes
  chrome.tabs.onActivated.addListener(async (activeInfo) => {
    await refreshTabInfo();
  });

  chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
    if (changeInfo.status === 'complete' || changeInfo.title) {
      chrome.tabs.query({ active: true, currentWindow: true }, async ([activeTab]) => {
        if (activeTab && activeTab.id === tabId) {
          updatePageInfo(tab.title, tab.url);
        }
      });
    }
  });

  // Initialize on load
  initialize();
});
