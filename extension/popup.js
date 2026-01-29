// Popup script for Browser AI Assistant

const chatContainer = document.getElementById('chat-container');
const messageInput = document.getElementById('message-input');
const sendBtn = document.getElementById('send-btn');
const statusDot = document.getElementById('status-dot');
const statusText = document.getElementById('status-text');

// Update connection status
function updateStatus(connected) {
  if (connected) {
    statusDot.classList.remove('disconnected');
    statusDot.classList.add('connected');
    statusText.textContent = 'Connected';
  } else {
    statusDot.classList.remove('connected');
    statusDot.classList.add('disconnected');
    statusText.textContent = 'Disconnected';
  }
}

// Add message to chat
function addMessage(text, isUser = false) {
  const messageDiv = document.createElement('div');
  messageDiv.className = `message ${isUser ? 'user' : 'assistant'}`;
  messageDiv.textContent = text;
  chatContainer.appendChild(messageDiv);
  chatContainer.scrollTop = chatContainer.scrollHeight;
}

// Send message to backend
async function sendMessage() {
  const message = messageInput.value.trim();
  if (!message) return;

  // Add user message to chat
  addMessage(message, true);
  messageInput.value = '';
  sendBtn.disabled = true;

  try {
    const response = await fetch('http://localhost:3000/api/chat', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ message }),
    });

    if (!response.ok) {
      throw new Error('Failed to get response');
    }

    const data = await response.json();
    addMessage(data.response || 'No response received');
    updateStatus(true);
  } catch (error) {
    console.error('Error sending message:', error);
    addMessage(
      'Sorry, I could not connect to the backend. Make sure the server is running.'
    );
    updateStatus(false);
  } finally {
    sendBtn.disabled = false;
  }
}

// Event listeners
sendBtn.addEventListener('click', sendMessage);
messageInput.addEventListener('keypress', (e) => {
  if (e.key === 'Enter') {
    sendMessage();
  }
});

// Check backend connection on load
async function checkConnection() {
  try {
    const response = await fetch('http://localhost:3000/health');
    updateStatus(response.ok);
  } catch {
    updateStatus(false);
  }
}

// Initialize
checkConnection();
