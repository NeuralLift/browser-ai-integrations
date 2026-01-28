// Side Panel script for Browser AI Assistant

// Simple Markdown parser
function parseMarkdown(text) {
  if (!text) return '';

  let html = text;

  // Escape HTML first
  html = html
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;');

  // Code blocks (```language\ncode```)
  html = html.replace(/```(\w*)\n([\s\S]*?)```/g, (match, lang, code) => {
    return `<pre><code class="language-${lang}">${code.trim()}</code></pre>`;
  });

  // Inline code (`code`)
  html = html.replace(/`([^`]+)`/g, '<code>$1</code>');

  // Headers
  html = html.replace(/^### (.+)$/gm, '<h3>$1</h3>');
  html = html.replace(/^## (.+)$/gm, '<h2>$1</h2>');
  html = html.replace(/^# (.+)$/gm, '<h1>$1</h1>');

  // Bold
  html = html.replace(/\*\*(.+?)\*\*/g, '<strong>$1</strong>');
  html = html.replace(/__(.+?)__/g, '<strong>$1</strong>');

  // Italic
  html = html.replace(/\*(.+?)\*/g, '<em>$1</em>');
  html = html.replace(/_(.+?)_/g, '<em>$1</em>');

  // Blockquotes
  html = html.replace(/^> (.+)$/gm, '<blockquote>$1</blockquote>');

  // Unordered lists
  html = html.replace(/^[\-\*] (.+)$/gm, '<li>$1</li>');
  html = html.replace(/(<li>.*<\/li>\n?)+/g, '<ul>$&</ul>');

  // Ordered lists
  html = html.replace(/^\d+\. (.+)$/gm, '<li>$1</li>');

  // Horizontal rule
  html = html.replace(/^---$/gm, '<hr>');

  // Links
  html = html.replace(/\[([^\]]+)\]\(([^)]+)\)/g, '<a href="$2" target="_blank">$1</a>');

  // Paragraphs - wrap remaining text blocks
  html = html.replace(/^(?!<[a-z])(.*[^\n])$/gm, '<p>$1</p>');

  // Clean up empty paragraphs and fix nesting
  html = html.replace(/<p><\/p>/g, '');
  html = html.replace(/<p>(<h[123]>)/g, '$1');
  html = html.replace(/(<\/h[123]>)<\/p>/g, '$1');
  html = html.replace(/<p>(<ul>)/g, '$1');
  html = html.replace(/(<\/ul>)<\/p>/g, '$1');
  html = html.replace(/<p>(<pre>)/g, '$1');
  html = html.replace(/(<\/pre>)<\/p>/g, '$1');
  html = html.replace(/<p>(<blockquote>)/g, '$1');
  html = html.replace(/(<\/blockquote>)<\/p>/g, '$1');
  html = html.replace(/<p>(<hr>)<\/p>/g, '$1');

  // Line breaks for remaining newlines
  html = html.replace(/\n/g, '');

  return html;
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
  const closeSettingsBtn = document.getElementById('close-settings');
  const closeSessionsBtn = document.getElementById('close-sessions');
  const saveSettingsBtn = document.getElementById('save-settings-btn');
  const sessionListEl = document.getElementById('session-list');
  const quickActionsEl = document.querySelector('.quick-actions');

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
      bubbleDiv.textContent = message.text;
    } else {
      bubbleDiv.innerHTML = parseMarkdown(message.text);

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
    const text = messageText || messageInput.value.trim();
    if (!text || isProcessing) return;

    // Ensure we have a valid session
    if (!currentSession) {
      currentSession = SessionManager.createSession();
      updateSessionUI();
    }

    isProcessing = true;

    // Add user message to UI and Session
    const userMsg = { role: 'user', text: text, timestamp: Date.now() };
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
        if (confirm('Hapus sesi ini?')) {
          SessionManager.deleteSession(session.id);
          if (session.id === currentSession.id) {
            initSession(); // Switch to another
          }
          renderSessionList();
        }
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
