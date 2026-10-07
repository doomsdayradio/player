#!/usr/bin/env bash
set -euo pipefail

source_dir="$(cd -- "$(dirname -- "$0")" && pwd)"
data_home="${XDG_DATA_HOME:-$HOME/.local/share}"
app_dir="$data_home/doomsday-radio"
applications_dir="$data_home/applications"

install -Dm755 "$source_dir/doomsday-radio-linux-x64" "$app_dir/doomsday-radio"
install -Dm644 "$source_dir/doomsday-radio.png" "$app_dir/doomsday-radio.png"
mkdir -p "$applications_dir"
exec_path="$(printf '%s' "$app_dir/doomsday-radio" | sed 's/\\/\\\\\\\\/g; s/[\x22\x24\x60]/\\\\&/g; s/%/%%/g')"
icon_path="$(printf '%s' "$app_dir/doomsday-radio.png" | sed 's/\\/\\\\/g')"
printf '%s\n' \
  '[Desktop Entry]' \
  'Type=Application' \
  'Name=Doomsday Radio' \
  'Comment=Doomsday Radio live stream player' \
  "Exec=\"$exec_path\"" \
  "Icon=$icon_path" \
  'Terminal=false' \
  'Categories=AudioVideo;Audio;' \
  > "$applications_dir/doomsday-radio.desktop"

if command -v update-desktop-database > /dev/null 2>&1; then
  update-desktop-database "$applications_dir"
fi
printf 'Doomsday Radio installed in %s\n' "$app_dir"
printf 'Desktop entry: %s/doomsday-radio.desktop\n' "$applications_dir"
