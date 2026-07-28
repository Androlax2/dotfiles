#!/bin/bash
#
# Launch the local Satisfactory dedicated server on demand.
#
# The server reads its saves from ~/.config/Epic/FactoryGame/Saved/SaveGames/.
# Steam keeps the build auto-updated when --update is passed; the save must
# match the server version (the Eros save is 1.2.x).
#
# Usage:
#   satisfactory-server.sh            # launch the server
#   satisfactory-server.sh --update   # run steamcmd app_update first, then launch
#
# Friend connects to <public-ip>:7777 (UDP+TCP forwarded on the router to this
# host). You connect locally to 127.0.0.1:7777.

set -euo pipefail

SERVER_DIR="$HOME/satisfactory-server"
LAUNCHER="$SERVER_DIR/FactoryServer.sh"
STEAM_APP_ID=1690800

PORT=7777
RELIABLE_PORT=8888
MAX_MEMORY_MB=6144

if [[ ! -x "$LAUNCHER" ]]; then
	echo "error: server launcher not found at $LAUNCHER" >&2
	echo "install/update first with: $0 --update" >&2
	exit 1
fi

if [[ "${1:-}" == "--update" ]]; then
	echo ">> Updating Satisfactory dedicated server via steamcmd..."
	steamcmd \
		+force_install_dir "$SERVER_DIR" \
		+login anonymous \
		+app_update "$STEAM_APP_ID" validate \
		+quit
fi

echo ">> Starting Satisfactory dedicated server on port $PORT..."
echo "   Local client:  127.0.0.1:$PORT"
echo "   Saves:         $HOME/.config/Epic/FactoryGame/Saved/SaveGames/"
echo "   Stop with Ctrl-C."

exec "$LAUNCHER" \
	-Port="$PORT" \
	-ReliablePort="$RELIABLE_PORT" \
	-USEALLAVAILABLECORES \
	-NoVerifyGC \
	-UseMultithreadForDS \
	-MaxMemoryUsage="$MAX_MEMORY_MB"
