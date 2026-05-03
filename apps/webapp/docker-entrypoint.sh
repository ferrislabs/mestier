#!/bin/sh

set -eu

HTML_DIR=/usr/share/nginx/html
SRC_DIR=/usr/local/src/mestier
CONFIG_FILE="$HTML_DIR/config.json"

rm -rf -- "${HTML_DIR:?}/"* "${HTML_DIR:?}/".[!.]* "${HTML_DIR:?}/"..?*
cp -r "$SRC_DIR"/* "$HTML_DIR"

if [ -f "$CONFIG_FILE" ]; then
  api_url="${API_URL:-}"
  escaped_api_url=$(printf '%s' "$api_url" | sed -e 's/[\/&|]/\\&/g')
  # shellcheck disable=SC2016
  sed -i "s|\${API_URL}|$escaped_api_url|g" "$CONFIG_FILE"

  issuer_url="${ISSUER_URL:-}"
  escaped_issuer_url=$(printf '%s' "$issuer_url" | sed -e 's/[\/&|]/\\&/g')
  # shellcheck disable=SC2016
  sed -i "s|\${ISSUER_URL}|$escaped_issuer_url|g" "$CONFIG_FILE"
fi

exec "$@"
