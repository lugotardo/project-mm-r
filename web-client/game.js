const API_BASE = 'http://127.0.0.1:8080';
const WS_URL = 'ws://127.0.0.1:8080/ws/game';
const DEBUG_MODE = true; // ← Modo debug ativo

let ws = null;
let sessionToken = null;
let canvas, ctx;
let viewport = null;
let playerName = '';

const TILE_SIZE = 16;
const FONT_SIZE = 14;

// Inicialização
document.addEventListener('DOMContentLoaded', () => {
    canvas = document.getElementById('game-canvas');
    ctx = canvas.getContext('2d');
    
    // Configurar canvas
    ctx.font = `${FONT_SIZE}px monospace`;
    ctx.textAlign = 'center';
    ctx.textBaseline = 'middle';
    
    // Login
    document.getElementById('login-btn').addEventListener('click', login);
    
    // Controles
    document.addEventListener('keydown', handleKeyPress);
    
    resizeCanvas();
    window.addEventListener('resize', resizeCanvas);
});

function login() {
    const nameInput = document.getElementById('player-name-input');
    const passwordInput = document.getElementById('player-password-input');
    const username = nameInput.value.trim() || "Adventurer";
    const password = passwordInput.value.trim() || "debug";
    
    playerName = username;
    
    if (DEBUG_MODE) {
        console.log('🔧 DEBUG MODE: Login automático');
        // Em modo debug, cria token fake local
        sessionToken = 'debug-token-' + Date.now();
        document.getElementById('login-modal').style.display = 'none';
        document.getElementById('player-name').textContent = username;
        connectWebSocket();
        addMessage(`Conectado como ${username} (debug mode)`, 'info');
        return;
    }
    
    // Login real (para produção)
    attemptLogin(username, password);
}

async function attemptLogin(username, password) {
    try {
        const response = await fetch(`${API_BASE}/auth/login`, {
            method: 'POST',
            headers: { 'Content-Type': 'application/json' },
            body: JSON.stringify({ username, password })
        });

        const data = await response.json();

        if (data.success && data.token) {
            sessionToken = data.token;
            document.getElementById('login-modal').style.display = 'none';
            document.getElementById('player-name').textContent = username;
            connectWebSocket();
            addMessage(data.message, 'info');
        } else {
            alert(data.message || 'Erro no login');
        }
    } catch (error) {
        console.error('Erro no login:', error);
        alert('Erro ao conectar ao servidor');
    }
}

function connectWebSocket() {
    ws = new WebSocket(WS_URL);
    
    ws.onopen = () => {
        console.log('✅ WebSocket conectado');
        
        if (DEBUG_MODE) {
            // Em modo debug, envia mensagem de login direto
            const loginMsg = {
                Login: { player_name: playerName }
            };
            ws.send(JSON.stringify(loginMsg));
        } else {
            // Em produção, envia token de autenticação
            const authMsg = {
                token: sessionToken,
                message: { Login: { player_name: playerName } }
            };
            ws.send(JSON.stringify(authMsg));
        }
    };
    
    ws.onmessage = (event) => {
        const data = JSON.parse(event.data);
        
        if (data.error) {
            alert(data.error);
            if (!DEBUG_MODE) {
                window.location.reload();
            }
        } else {
            viewport = data.viewport;
            document.getElementById('tick').textContent = data.tick;
            
            if (viewport) {
                document.getElementById('position').textContent = 
                    `@(${viewport.player_pos.x},${viewport.player_pos.y})`;
                renderViewport();
                updateNearbyList();
            }
        }
    };
    
    ws.onerror = (error) => {
        console.error('Erro WebSocket:', error);
        addMessage('Erro de conexão', 'combat');
    };
    
    ws.onclose = () => {
        console.log('WebSocket desconectado');
        addMessage('Desconectado do servidor', 'combat');
        
        if (DEBUG_MODE) {
            setTimeout(() => {
                addMessage('Tentando reconectar...', 'info');
                connectWebSocket();
            }, 3000);
        }
    };
}

function handleKeyPress(e) {
    if (!ws || !viewport) return;
    
    let dx = 0, dy = 0;
    
    // Movimento
    switch(e.key.toLowerCase()) {
        case 'w': case 'k': case 'arrowup': dy = -1; break;
        case 's': case 'j': case 'arrowdown': dy = 1; break;
        case 'a': case 'h': case 'arrowleft': dx = -1; break;
        case 'd': case 'l': case 'arrowright': dx = 1; break;
        case '.': return; // Wait
        default: return;
    }
    
    // Envia apenas o movimento, servidor responderá no próximo tick
    if (dx !== 0 || dy !== 0) {
        const moveMsg = DEBUG_MODE ? 
            { Move: { dx, dy } } :
            { token: sessionToken, message: { Move: { dx, dy } } };
        
        ws.send(JSON.stringify(moveMsg));
        e.preventDefault();
    }
}

function resizeCanvas() {
    const container = canvas.parentElement;
    canvas.width = container.clientWidth;
    canvas.height = container.clientHeight;
    
    if (viewport) renderViewport();
}

function renderViewport() {
    if (!viewport) return;
    
    ctx.clearRect(0, 0, canvas.width, canvas.height);
    
    const tilesX = Math.floor(canvas.width / TILE_SIZE);
    const tilesY = Math.floor(canvas.height / TILE_SIZE);
    const offsetX = Math.floor((canvas.width - tilesX * TILE_SIZE) / 2);
    const offsetY = Math.floor((canvas.height - tilesY * TILE_SIZE) / 2);
    
    const centerX = viewport.player_pos.x;
    const centerY = viewport.player_pos.y;
    
    // Renderizar tiles
    for (let y = 0; y < tilesY; y++) {
        for (let x = 0; x < tilesX; x++) {
            const worldX = centerX - Math.floor(tilesX / 2) + x;
            const worldY = centerY - Math.floor(tilesY / 2) + y;
            
            const tile = viewport.tiles.find(t => t.x === worldX && t.y === worldY);
            
            const screenX = offsetX + x * TILE_SIZE;
            const screenY = offsetY + y * TILE_SIZE;
            
            if (tile) {
                // Background
                ctx.fillStyle = tile.bg_color;
                ctx.fillRect(screenX, screenY, TILE_SIZE, TILE_SIZE);
                
                // Glyph
                ctx.fillStyle = tile.fg_color;
                ctx.fillText(
                    tile.glyph,
                    screenX + TILE_SIZE / 2,
                    screenY + TILE_SIZE / 2
                );
            } else {
                // Void
                ctx.fillStyle = '#000';
                ctx.fillRect(screenX, screenY, TILE_SIZE, TILE_SIZE);
            }
        }
    }
    
    // Renderizar entidades
    viewport.entities.forEach(entity => {
        const screenX = offsetX + (entity.x - centerX + Math.floor(tilesX / 2)) * TILE_SIZE;
        const screenY = offsetY + (entity.y - centerY + Math.floor(tilesY / 2)) * TILE_SIZE;
        
        if (screenX >= 0 && screenX < canvas.width && screenY >= 0 && screenY < canvas.height) {
            ctx.fillStyle = entity.color;
            ctx.font = `bold ${FONT_SIZE}px monospace`;
            ctx.fillText(
                entity.glyph,
                screenX + TILE_SIZE / 2,
                screenY + TILE_SIZE / 2
            );
            ctx.font = `${FONT_SIZE}px monospace`;
        }
    });
}

function updateNearbyList() {
    if (!viewport) return;
    
    const list = document.getElementById('nearby-list');
    list.innerHTML = '';
    
    viewport.entities.forEach(entity => {
        if (entity.glyph !== '@') {
            const item = document.createElement('div');
            item.className = 'nearby-item';
            item.innerHTML = `<span style="color:${entity.color}">${entity.glyph}</span> ${entity.name}`;
            list.appendChild(item);
        }
    });
    
    if (viewport.entities.length <= 1) {
        list.innerHTML = '<div class="nearby-item" style="color:#666">No one nearby</div>';
    }
}

function addMessage(text, type = '') {
    const log = document.getElementById('message-log');
    const item = document.createElement('div');
    item.className = `message-item ${type}`;
    item.textContent = `> ${text}`;
    
    log.insertBefore(item, log.firstChild);
    
    while (log.children.length > 50) {
        log.removeChild(log.lastChild);
    }
}

// Permitir Enter para login
document.getElementById('player-name-input').addEventListener('keypress', (e) => {
    if (e.key === 'Enter') login();
});

document.getElementById('player-password-input').addEventListener('keypress', (e) => {
    if (e.key === 'Enter') login();
});

document.addEventListener('DOMContentLoaded', init);
