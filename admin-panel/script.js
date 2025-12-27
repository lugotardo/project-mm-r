const API_BASE = 'http://127.0.0.1:3030/api';
const WS_URL = 'ws://127.0.0.1:3030/ws/events';

let ws = null;
let reconnectInterval = null;

// Conectar ao WebSocket
function connectWebSocket() {
    const statusEl = document.getElementById('connection-status');
    
    ws = new WebSocket(WS_URL);
    
    ws.onopen = () => {
        console.log('✅ WebSocket conectado');
        statusEl.textContent = '🟢 Conectado';
        statusEl.className = 'status connected';
        
        if (reconnectInterval) {
            clearInterval(reconnectInterval);
            reconnectInterval = null;
        }
    };
    
    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        handleEvent(data);
    };
    
    ws.onclose = () => {
        console.log('❌ WebSocket desconectado');
        statusEl.textContent = '⚫ Desconectado';
        statusEl.className = 'status disconnected';
        
        // Tentar reconectar
        if (!reconnectInterval) {
            reconnectInterval = setInterval(() => {
                console.log('🔄 Tentando reconectar...');
                connectWebSocket();
            }, 3000);
        }
    };
    
    ws.onerror = (error) => {
        console.error('Erro WebSocket:', error);
    };
}

// Buscar estatísticas
async function fetchStats() {
    try {
        const response = await fetch(`${API_BASE}/stats`);
        const stats = await response.json();
        updateStats(stats);
    } catch (error) {
        console.error('Erro ao buscar stats:', error);
    }
}

// Atualizar estatísticas na UI
function updateStats(stats) {
    document.getElementById('uptime').textContent = formatUptime(stats.uptime_seconds);
    document.getElementById('active-players').textContent = stats.active_players;
    document.getElementById('total-entities').textContent = stats.total_entities;
    document.getElementById('world-size').textContent = `${stats.world_size[0]}×${stats.world_size[1]}`;
    document.getElementById('ticks').textContent = stats.ticks_processed.toLocaleString();
}

// Formatar uptime
function formatUptime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours}h ${minutes}m ${secs}s`;
}

// Manipular eventos do WebSocket
function handleEvent(event) {
    const eventsContainer = document.getElementById('events-container');
    const eventEl = document.createElement('div');
    eventEl.className = 'event-item';
    
    const time = new Date().toLocaleTimeString();
    let message = '';
    let eventType = '';
    
    if (event.PlayerConnected) {
        eventType = 'player-connected';
        message = `🟢 Jogador "${event.PlayerConnected.name}" conectou (ID: ${event.PlayerConnected.id})`;
    } else if (event.PlayerDisconnected) {
        eventType = 'player-disconnected';
        message = `🔴 Jogador "${event.PlayerDisconnected.name}" desconectou (ID: ${event.PlayerDisconnected.id})`;
    } else if (event.PlayerMoved) {
        eventType = 'player-moved';
        const { name, from, to } = event.PlayerMoved;
        message = `🚶 ${name} moveu de (${from.x}, ${from.y}) → (${to.x}, ${to.y})`;
    } else if (event.WorldTick) {
        eventType = 'world-tick';
        message = `⏱️ Tick ${event.WorldTick.tick}`;
    } else if (event.ServerStats) {
        updateStats(event.ServerStats);
        return; // Não adiciona aos eventos
    }
    
    eventEl.classList.add(eventType);
    eventEl.innerHTML = `
        <span class="event-time">${time}</span>
        <span class="event-message">${message}</span>
    `;
    
    eventsContainer.insertBefore(eventEl, eventsContainer.firstChild);
    
    // Limita a 50 eventos
    while (eventsContainer.children.length > 50) {
        eventsContainer.removeChild(eventsContainer.lastChild);
    }
}

// Inicialização
function init() {
    connectWebSocket();
    fetchStats();
    setInterval(fetchStats, 5000); // Atualiza stats a cada 5s
}

// Iniciar quando a página carregar
document.addEventListener('DOMContentLoaded', init);
