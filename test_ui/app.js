// Configuration
let apiUrl = window.location.origin; // Use relative path based on current origin
let apiKey = 'very-secure-key-one';
let entityId = 'acme-corp';
let chatHistory = [];

// DOM Elements
const apiUrlInput = document.getElementById('api-url');
const apiKeyInput = document.getElementById('api-key');
const entityIdInput = document.getElementById('entity-id');
const chatMessages = document.getElementById('chat-messages');
const chatInput = document.getElementById('chat-input');
const sendButton = document.getElementById('send-button');
const clearChatButton = document.getElementById('clear-chat');
const documentContent = document.getElementById('document-content');
const addDocumentButton = document.getElementById('add-document');
const documentResponse = document.getElementById('document-response');
const healthIndicator = document.getElementById('health-indicator');
const healthText = document.getElementById('health-text');

// Set the API URL input to show current origin
apiUrlInput.value = apiUrl;
apiUrlInput.placeholder = apiUrl;

// Tab switching
document.querySelectorAll('.tab-button').forEach(button => {
    button.addEventListener('click', () => {
        const tabName = button.dataset.tab;
        
        // Update buttons
        document.querySelectorAll('.tab-button').forEach(b => b.classList.remove('active'));
        button.classList.add('active');
        
        // Update content
        document.querySelectorAll('.tab-content').forEach(content => {
            content.classList.remove('active');
        });
        document.getElementById(`${tabName}-tab`).classList.add('active');
    });
});

// Configuration updates
apiUrlInput.addEventListener('change', (e) => {
    apiUrl = e.target.value;
    checkHealth();
});

apiKeyInput.addEventListener('change', (e) => {
    apiKey = e.target.value;
});

entityIdInput.addEventListener('change', (e) => {
    entityId = e.target.value;
});

// Health check
async function checkHealth() {
    try {
        const response = await fetch(`${apiUrl}/health`);
        const data = await response.json();
        
        if (data.status === 'ok') {
            healthIndicator.className = 'status-indicator healthy';
            healthText.textContent = 'Healthy';
        } else {
            healthIndicator.className = 'status-indicator unhealthy';
            healthText.textContent = 'Unhealthy';
        }
    } catch (error) {
        healthIndicator.className = 'status-indicator unhealthy';
        healthText.textContent = 'Offline';
    }
}

// Chat functionality
function addMessageToUI(role, text, isError = false) {
    const messageDiv = document.createElement('div');
    messageDiv.className = `message message-${role}`;
    
    const roleLabel = document.createElement('div');
    roleLabel.className = 'message-role';
    roleLabel.textContent = role === 'user' ? 'You' : 'Assistant';
    
    const contentDiv = document.createElement('div');
    contentDiv.className = 'message-content';
    contentDiv.textContent = text;
    
    messageDiv.appendChild(roleLabel);
    messageDiv.appendChild(contentDiv);
    
    if (isError) {
        const errorDiv = document.createElement('div');
        errorDiv.className = 'error-message';
        errorDiv.textContent = text;
        chatMessages.appendChild(errorDiv);
    } else {
        chatMessages.appendChild(messageDiv);
    }
    
    chatMessages.scrollTop = chatMessages.scrollHeight;
}

async function sendMessage() {
    const message = chatInput.value.trim();
    
    if (!message) {
        return;
    }
    
    if (!entityId) {
        addMessageToUI('system', 'Please enter an Entity ID in the configuration section.', true);
        return;
    }
    
    // Add user message to UI
    addMessageToUI('user', message);
    
    // Add to chat history
    chatHistory.push({
        role: 'user',
        text: message
    });
    
    // Clear input
    chatInput.value = '';
    
    // Disable send button
    sendButton.disabled = true;
    sendButton.innerHTML = 'Sending<span class="loading"></span>';
    
    try {
        const headers = {
            'Content-Type': 'application/json'
        };
        
        if (apiKey) {
            headers['Authorization'] = `Bearer ${apiKey}`;
        }
        
        const response = await fetch(`${apiUrl}/api/chat`, {
            method: 'POST',
            headers: headers,
            body: JSON.stringify({
                entity_id: entityId,
                messages: chatHistory
            })
        });
        
        if (!response.ok) {
            if (response.status === 403) {
                throw new Error('Authentication required. Please provide a valid API key.');
            }
            throw new Error(`HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        
        if (data.error) {
            addMessageToUI('system', `Error: ${data.error}`, true);
        } else {
            addMessageToUI('assistant', data.text);
            chatHistory.push({
                role: 'assistant',
                text: data.text
            });
        }
    } catch (error) {
        addMessageToUI('system', `Error: ${error.message}`, true);
    } finally {
        sendButton.disabled = false;
        sendButton.textContent = 'Send';
    }
}

sendButton.addEventListener('click', sendMessage);

chatInput.addEventListener('keydown', (e) => {
    if (e.key === 'Enter' && !e.shiftKey) {
        e.preventDefault();
        sendMessage();
    }
});

clearChatButton.addEventListener('click', () => {
    chatHistory = [];
    chatMessages.innerHTML = '<div class="system-message">Chat cleared. Start a new conversation.</div>';
});

// Document functionality
async function addDocument() {
    const content = documentContent.value.trim();
    
    if (!content) {
        showDocumentResponse('Please enter document content.', 'error');
        return;
    }
    
    if (!entityId) {
        showDocumentResponse('Please enter an Entity ID in the configuration section.', 'error');
        return;
    }
    
    addDocumentButton.disabled = true;
    addDocumentButton.innerHTML = 'Adding<span class="loading"></span>';
    
    try {
        const headers = {
            'Content-Type': 'application/json'
        };
        
        if (apiKey) {
            headers['Authorization'] = `Bearer ${apiKey}`;
        }
        
        const response = await fetch(`${apiUrl}/api/documents`, {
            method: 'POST',
            headers: headers,
            body: JSON.stringify({
                entity_id: entityId,
                content: content
            })
        });
        
        if (!response.ok) {
            if (response.status === 403) {
                throw new Error('Authentication required. Please provide a valid API key.');
            }
            const errorData = await response.json();
            throw new Error(errorData.error || `HTTP error! status: ${response.status}`);
        }
        
        const data = await response.json();
        showDocumentResponse(`Success! Document ID: ${data.document_id}`, 'success');
        documentContent.value = '';
    } catch (error) {
        showDocumentResponse(`Error: ${error.message}`, 'error');
    } finally {
        addDocumentButton.disabled = false;
        addDocumentButton.textContent = 'Add Document';
    }
}

function showDocumentResponse(message, type) {
    documentResponse.textContent = message;
    documentResponse.className = `response-message ${type}`;
    
    setTimeout(() => {
        documentResponse.className = 'response-message';
    }, 5000);
}

addDocumentButton.addEventListener('click', addDocument);

// Example buttons
document.querySelectorAll('.example-button').forEach(button => {
    button.addEventListener('click', () => {
        documentContent.value = button.dataset.content;
    });
});

// Initialize
checkHealth();
setInterval(checkHealth, 30000); // Check health every 30 seconds
