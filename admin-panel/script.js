const API_BASE = 'http://127.0.0.1:8080/api';
const WS_URL = 'ws://127.0.0.1:3030/ws/events';

let ws = null;
let reconnectInterval = null;
let startTime = Date.now();
let playerCount = 0;
let totalEntities = 0;
let eventCount = 0;

// Conectar ao WebSocket do admin-server
function connectWebSocket() {
    console.log('🔌 Conectando ao WebSocket admin...');
    ws = new WebSocket(WS_URL);
    
    ws.onopen = () => {
        console.log('✅ WebSocket admin conectado');
        const statusDot = document.getElementById('connection-status');
        if (statusDot) {
            statusDot.classList.remove('disconnected');
            statusDot.classList.add('connected');
        }
        
        if (reconnectInterval) {
            clearInterval(reconnectInterval);
            reconnectInterval = null;
        }
    };
    
    ws.onmessage = (event) => {
        try {
            const data = JSON.parse(event.data);
            console.log('📡 Evento recebido:', data);
            handleEvent(data);
        } catch (error) {
            console.error('Erro ao processar evento:', error);
        }
    };
    
    ws.onclose = () => {
        console.log('❌ WebSocket admin desconectado');
        const statusDot = document.getElementById('connection-status');
        if (statusDot) {
            statusDot.classList.remove('connected');
            statusDot.classList.add('disconnected');
        }
        
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

// Processar eventos recebidos
function handleEvent(event) {
    eventCount++;
    
    if (event.PlayerConnected) {
        playerCount++;
        totalEntities++;
        addEventToLog(`🟢 ${event.PlayerConnected.name} conectou (ID: ${event.PlayerConnected.id})`);
        updateStats();
    }
    
    if (event.PlayerDisconnected) {
        playerCount = Math.max(0, playerCount - 1);
        totalEntities = Math.max(0, totalEntities - 1);
        addEventToLog(`🔴 ${event.PlayerDisconnected.name} desconectou`);
        updateStats();
    }
    
    if (event.PlayerMoved) {
        const { name, from, to } = event.PlayerMoved;
        addEventToLog(`🚶 ${name} moveu de (${from.x},${from.y}) → (${to.x},${to.y})`);
    }
    
    if (event.PlayerSpawned) {
        addEventToLog(`⭐ ${event.PlayerSpawned.name} spawnou em (${event.PlayerSpawned.pos.x},${event.PlayerSpawned.pos.y})`);
    }
    
    if (event.WorldTick) {
        const worldTick = document.getElementById('world-tick');
        if (worldTick) {
            worldTick.textContent = event.WorldTick.tick;
        }
        
        totalEntities = event.WorldTick.total_entities || totalEntities;
        playerCount = event.WorldTick.active_players || playerCount;
        updateStats();
    }
}

// Adicionar evento ao log
function addEventToLog(message) {
    const stream = document.getElementById('event-stream');
    if (!stream) return;
    
    const time = new Date().toLocaleTimeString();
    const entry = document.createElement('div');
    entry.className = 'event-item info';
    entry.textContent = `[${time}] ${message}`;
    
    stream.insertBefore(entry, stream.firstChild);
    
    // Limita a 100 eventos
    while (stream.children.length > 100) {
        stream.removeChild(stream.lastChild);
    }
}

// Atualizar estatísticas na UI
function updateStats() {
    const uptimeEl = document.getElementById('uptime');
    const playerCountEl = document.getElementById('player-count');
    const totalEntitiesEl = document.getElementById('total-entities');
    const eventsEl = document.getElementById('events');
    
    if (uptimeEl) {
        const uptime = Math.floor((Date.now() - startTime) / 1000);
        uptimeEl.textContent = formatUptime(uptime);
    }
    
    if (playerCountEl) playerCountEl.textContent = playerCount;
    if (totalEntitiesEl) totalEntitiesEl.textContent = totalEntities;
    if (eventsEl) eventsEl.textContent = eventCount;
}

// Formatar uptime
function formatUptime(seconds) {
    const hours = Math.floor(seconds / 3600);
    const minutes = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours}h ${minutes}m ${secs}s`;
}

// Buscar dados reais das APIs.
async function fetchWorldMap() {
    try {
        const response = await fetch(`${API_BASE}/world/map`);
        const data = await response.json();
        renderWorldMap(data);
    } catch (error) {
        console.error('Erro ao buscar mapa:', error);
    }
}

// Renderizar mapa ASCII
function renderWorldMap(mapData) {
    const mapEl = document.getElementById('world-map');
    if (!mapEl) return;
    
    const { width, height, tiles } = mapData;
    
    let mapText = '';
    for (let y = 0; y < Math.min(height, 40); y++) {
        for (let x = 0; x < Math.min(width, 80); x++) {
            const tile = tiles.find(t => t.x === x && t.y === y);
            mapText += tile ? tile.glyph : ' ';
        }
        mapText += '\n';
    }
    
    mapEl.textContent = mapText;
}

// Buscar jogadores online
async function fetchPlayers() {
    try {
        const response = await fetch(`${API_BASE}/players`);
        const players = await response.json();
        renderPlayerList(players);
    } catch (error) {
        console.error('Erro ao buscar jogadores:', error);
    }
}

// Renderizar lista de jogadores
function renderPlayerList(players) {
    const listEl = document.getElementById('player-list');
    if (!listEl) return;
    
    if (players.length === 0) {
        listEl.innerHTML = '<div class="player-card">Nenhum jogador online</div>';
        return;
    }
    
    listEl.innerHTML = players.map(player => `
        <div class="player-card">
            <div><strong>${player.name}</strong></div>
            <div>ID: ${player.id.substring(0, 8)}...</div>
            <div>Entidade: #${player.entity_id}</div>
            ${player.position ? `<div>Posição: (${player.position.x}, ${player.position.y})</div>` : ''}
        </div>
    `).join('');
}

// Sistema de tabs atualizado
function switchTab(tabName) {
    // Remove active de todas as tabs
    document.querySelectorAll('.nav-tab').forEach(t => t.classList.remove('active'));
    document.querySelectorAll('.tab-panel').forEach(p => p.classList.remove('active'));
    
    // Ativa a tab clicada
    const tabButton = document.querySelector(`[data-tab="${tabName}"]`);
    const tabPanel = document.getElementById(`tab-${tabName}`);
    
    if (tabButton) tabButton.classList.add('active');
    if (tabPanel) tabPanel.classList.add('active');
    
    // Carregar dados específicos da tab
    if (tabName === 'world') {
        fetchWorldMap();
    } else if (tabName === 'players') {
        fetchPlayers();
    }
}

// Atualizar relógio
function updateClock() {
    updateStats();
}

// Atualizar dados periodicamente
function startPeriodicUpdates() {
    // Atualiza jogadores a cada 5 segundos
    setInterval(fetchPlayers, 5000);
    
    // Atualiza mapa a cada 30 segundos
    setInterval(fetchWorldMap, 30000);
}

// Inicialização
function init() {
    console.log('🚀 Inicializando Admin Panel...');
    
    // Configurar tabs
    const navTabs = document.querySelectorAll('.nav-tab');
    navTabs.forEach(tab => {
        tab.addEventListener('click', () => {
            const tabName = tab.dataset.tab;
            switchTab(tabName);
        });
    });
    
    // Conectar WebSocket
    connectWebSocket();
    
    // Atualizar stats a cada segundo
    setInterval(updateClock, 1000);
    
    // Buscar dados iniciais
    fetchPlayers();
    fetchWorldMap();
    
    // Iniciar updates periódicos
    startPeriodicUpdates();
    
    // Update inicial
    updateStats();
    
    console.log('✅ Admin Panel inicializado');
}

// Iniciar quando DOM estiver pronto
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', init);
} else {
    init();
}
