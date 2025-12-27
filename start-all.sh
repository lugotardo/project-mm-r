#!/bin/bash
# filepath: /home/luancosta/Documentos/Code/Git/GitHub/MM/start-all.sh

echo "🚀 Iniciando MM Servers..."

# Detecta qual terminal emulator está disponível
if command -v gnome-terminal &> /dev/null; then
    TERM_CMD="gnome-terminal"
elif command -v konsole &> /dev/null; then
    TERM_CMD="konsole"
elif command -v xfce4-terminal &> /dev/null; then
    TERM_CMD="xfce4-terminal"
elif command -v xterm &> /dev/null; then
    TERM_CMD="xterm"
else
    echo "❌ Nenhum terminal emulator encontrado!"
    exit 1
fi

# Cria diretórios de logs se não existirem
mkdir -p logs

# Função para matar processos antigos
cleanup() {
    echo "🧹 Limpando processos antigos..."
    pkill -f game-server
    pkill -f admin-server
    pkill -f backend
    sleep 1
}

# Limpa processos antigos
cleanup

# Espera um pouco
sleep 1

echo "📦 Compilando projeto..."
cargo build --release 2>&1 | tee logs/build.log

if [ $? -ne 0 ]; then
    echo "❌ Erro na compilação!"
    exit 1
fi

echo "✅ Compilação concluída!"
echo ""

# Inicia Game Server (porta 8080)
if [ "$TERM_CMD" = "gnome-terminal" ]; then
    gnome-terminal --title="🎮 Game Server" -- bash -c "
        clear
        echo '🎮 Game Server Starting...'
        echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
        echo ''
        cargo run --release --bin game-server 2>&1 | tee logs/game-server.log
        echo ''
        echo '❌ Game Server encerrado. Pressione ENTER para fechar.'
        read
    " &
elif [ "$TERM_CMD" = "konsole" ]; then
    konsole --title "🎮 Game Server" -e bash -c "
        clear
        echo '🎮 Game Server Starting...'
        cargo run --release --bin game-server 2>&1 | tee logs/game-server.log
        read
    " &
else
    $TERM_CMD -e bash -c "
        clear
        echo '🎮 Game Server Starting...'
        cargo run --release --bin game-server 2>&1 | tee logs/game-server.log
        read
    " &
fi

sleep 2

# Inicia Admin Server (porta 3030)
if [ "$TERM_CMD" = "gnome-terminal" ]; then
    gnome-terminal --title="🖥️  Admin Panel" -- bash -c "
        clear
        echo '🖥️  Admin Panel Starting...'
        echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
        echo ''
        cargo run --release --bin admin-server 2>&1 | tee logs/admin-server.log
        echo ''
        echo '❌ Admin Server encerrado. Pressione ENTER para fechar.'
        read
    " &
elif [ "$TERM_CMD" = "konsole" ]; then
    konsole --title "🖥️  Admin Panel" -e bash -c "
        clear
        echo '🖥️  Admin Panel Starting...'
        cargo run --release --bin admin-server 2>&1 | tee logs/admin-server.log
        read
    " &
else
    $TERM_CMD -e bash -c "
        clear
        echo '🖥️  Admin Panel Starting...'
        cargo run --release --bin admin-server 2>&1 | tee logs/admin-server.log
        read
    " &
fi

sleep 2

# Inicia Backend (simulação do mundo - porta variável)
if [ "$TERM_CMD" = "gnome-terminal" ]; then
    gnome-terminal --title="🌍 World Simulator" -- bash -c "
        clear
        echo '🌍 World Simulator Starting...'
        echo '━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━'
        echo ''
        echo 'Este terminal roda a simulação do mundo.'
        echo 'NPCs, IA, eventos e ticks são processados aqui.'
        echo ''
        cargo run --release --bin backend 2>&1 | tee logs/backend.log
        echo ''
        echo '❌ Backend encerrado. Pressione ENTER para fechar.'
        read
    " &
elif [ "$TERM_CMD" = "konsole" ]; then
    konsole --title "🌍 World Simulator" -e bash -c "
        clear
        echo '🌍 World Simulator Starting...'
        cargo run --release --bin backend 2>&1 | tee logs/backend.log
        read
    " &
else
    $TERM_CMD -e bash -c "
        clear
        echo '🌍 World Simulator Starting...'
        cargo run --release --bin backend 2>&1 | tee logs/backend.log
        read
    " &
fi

sleep 3

echo ""
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo "✅ Todos os servidores foram iniciados!"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo ""
echo "📋 URLs disponíveis:"
echo "   🎮 Game Client:  http://127.0.0.1:8080"
echo "   🖥️  Admin Panel:  http://127.0.0.1:3030"
echo ""
echo "📁 Logs salvos em: ./logs/"
echo ""
echo "🛑 Para parar tudo: ./stop-all.sh"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"