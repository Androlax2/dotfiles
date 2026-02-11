#!/bin/bash
# Generates 60-airpods-priority.conf from its template,
# substituting the AirPods MAC address from the local secrets file.

SCRIPT_DIR="$(dirname "$(realpath "$0")")"
SECRETS_FILE="$HOME/.secrets/airpods"

if [ ! -f "$SECRETS_FILE" ]; then
  echo "Error: $SECRETS_FILE not found" >&2
  echo "Create it with: echo 'AIRPODS_MAC=\"XX:XX:XX:XX:XX:XX\"' > ~/.secrets/airpods" >&2
  exit 1
fi

source "$SECRETS_FILE"

# Convert colon-separated MAC to underscore-separated for wireplumber node matching
AIRPODS_MAC_UNDERSCORED="${AIRPODS_MAC//:/_}"

sed "s/@@AIRPODS_MAC_UNDERSCORED@@/$AIRPODS_MAC_UNDERSCORED/g" \
  "$SCRIPT_DIR/60-airpods-priority.conf.tmpl" \
  > "$SCRIPT_DIR/60-airpods-priority.conf"

echo "Generated $SCRIPT_DIR/60-airpods-priority.conf"
