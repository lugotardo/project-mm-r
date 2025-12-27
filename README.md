# MM - Multiplayer Adventure Game

Jogo online inspirado em Dwarf Fortress Adventure Mode.

## 🚀 Quick Start

```bash
# Dar permissão
chmod +x *.sh

# Iniciar tudo
./start-all.sh
```

Acesse: http://127.0.0.1:8080

## 🔧 DEBUG MODE

**Atualmente ativo por padrão!**

No modo debug:
- ✅ Login sem autenticação
- ✅ Qualquer nome funciona
- ✅ Senha opcional
- ✅ Reconexão automática
- ✅ Logs detalhados
- ✅ **CORS habilitado** (permite requisições de qualquer origem)

⚠️ **Em produção**, configure CORS restritivo:

```rust
let cors = warp::cors()
    .allow_origin("https://seu-dominio.com")
    .allow_methods(vec!["GET", "POST"])
    .allow_headers(vec!["Content-Type"]);
```

Para **desativar** (produção):

**web-client/game.js:**
```javascript
const DEBUG_MODE = false;
```

**game-server/src/main.rs:**
```rust
const DEBUG_MODE: bool = false;
```

## 🔐 Autenticação (Produção)

Login padrão quando DEBUG_MODE = false:
- Username: `admin`
- Password: `admin123`

## 📋 Servidores

| Servidor | Porta | Descrição |
|----------|-------|-----------|
| Game Server | 8080 | Cliente web do jogo |
| Admin Panel | 3030 | Painel administrativo |
| Backend | - | Simulação do mundo |

## 🎮 Como Jogar

1. Abra http://127.0.0.1:8080
2. Digite seu nome (qualquer um)
3. Clique "Start Adventure"
4. Use WASD ou HJKL para mover

## 🛠️ Comandos

```bash
make start      # Inicia tudo
make stop       # Para tudo
make logs       # Ver logs
```

## 🔧 Troubleshooting

### Erro: "future cannot be sent between threads"
Já corrigido! Se persistir:
```bash
cargo clean
cargo build
```

### Erro: "no such file or directory"
```bash
./check-structure.sh
./fix-structure.sh
```

### Erro: "Address already in use"
```bash
./stop-all.sh
./start-all.sh
```

### Recompilar do zero
```bash
./quick-fix.sh
```

## 📁 Estrutura do Projeto

## 🌐 Arquitetura de Rede

