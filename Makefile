.PHONY: all start stop dev clean check logs help

# Comandos principais
all: build start

start:
	@./start-all.sh

stop:
	@./stop-all.sh

dev:
	@./start-dev.sh

# Utilitários
build:
	@echo "📦 Compilando projeto..."
	@cargo build --release

clean:
	@echo "🧹 Limpando build..."
	@cargo clean
	@rm -rf logs/*.log

check:
	@./check-ports.sh

logs:
	@echo "📋 Logs recentes:"
	@echo "\n=== GAME SERVER ==="
	@tail -n 20 logs/game-server.log 2>/dev/null || echo "Sem logs"
	@echo "\n=== ADMIN SERVER ==="
	@tail -n 20 logs/admin-server.log 2>/dev/null || echo "Sem logs"
	@echo "\n=== BACKEND ==="
	@tail -n 20 logs/backend.log 2>/dev/null || echo "Sem logs"

test:
	@cargo test --all

help:
	@echo "🎮 MM - Comandos Disponíveis"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
	@echo "  make start    - Inicia todos os servidores"
	@echo "  make stop     - Para todos os servidores"
	@echo "  make dev      - Inicia em modo desenvolvimento"
	@echo "  make build    - Compila o projeto"
	@echo "  make clean    - Limpa arquivos de build"
	@echo "  make check    - Verifica portas em uso"
	@echo "  make logs     - Exibe logs recentes"
	@echo "  make test     - Roda testes"
	@echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
