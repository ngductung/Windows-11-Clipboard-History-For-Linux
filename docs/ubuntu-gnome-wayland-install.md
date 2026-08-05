# Hướng Dẫn Build Và Cài Đặt Ubuntu GNOME Wayland

Tài liệu này dành cho workflow local của Windows 11 Clipboard History For Linux trên Ubuntu/Debian GNOME Wayland:

- Build file `.deb` từ source.
- Cài đặt hoặc cài đè file `.deb`.
- Cài GNOME extension để `Super+V` mở popup tại vị trí con trỏ chuột.
- Cấp quyền `/dev/uinput` để app tự paste vào app đích.

## 1. Yêu Cầu Hệ Thống

Kiểm tra session hiện tại:

```bash
echo "$XDG_SESSION_TYPE"
gnome-shell --version
```

Nếu `XDG_SESSION_TYPE=wayland`, nên cài GNOME extension trong repo này để popup có thể mở tại vị trí con trỏ chuột.

## 2. Cài Dependency Build

Từ thư mục source:

```bash
cd /duong/dan/toi/Windows-11-Clipboard-History-For-Linux
make deps
make rust
make node
source "$HOME/.cargo/env"
npm ci
```

Kiểm tra nhanh:

```bash
node --version
npm --version
rustc --version
cargo --version
```

Nếu máy đã có Node.js/Rust sẵn, `make rust` và `make node` có thể báo đã cài sẵn, điều đó bình thường.

## 3. Build File `.deb`

Chạy verify trước khi build:

```bash
npm run lint
PATH="$HOME/.cargo/bin:$PATH" cargo fmt --manifest-path src-tauri/Cargo.toml --check
PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml --locked
```

Build package `.deb`:

```bash
PATH="$HOME/.cargo/bin:$PATH" npm run tauri:build -- --bundles deb
```

File `.deb` sẽ nằm tại:

```bash
src-tauri/target/release/bundle/deb/
```

Ví dụ:

```bash
ls -lh src-tauri/target/release/bundle/deb/*.deb
```

## 4. Cài Đặt File `.deb`

Cài lần đầu bằng `apt`:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/win11-clipboard-history_0.7.1_amd64.deb
```

Nếu đang cài đè cùng version, `apt` có thể báo package đã là newest version. Khi đó dùng `dpkg -i` để ép ghi đè binary mới:

```bash
sudo dpkg -i ./src-tauri/target/release/bundle/deb/win11-clipboard-history_0.7.1_amd64.deb
sudo apt -f install
```

Kiểm tra binary:

```bash
which win11-clipboard-history
win11-clipboard-history --help
```

Nếu đang debug tính năng mở theo tọa độ, output `--help` cần có flag:

```text
--show-at X Y
```

## 5. Cấp Quyền Tự Paste Bằng `/dev/uinput`

Trên Wayland, app cần `/dev/uinput` để mô phỏng phím paste. Nếu chọn item chỉ copy vào clipboard nhưng không tự paste, chạy:

```bash
sudo modprobe uinput
sudo setfacl -m u:$USER:rw /dev/uinput
```

Để quyền bền hơn sau reboot:

```bash
sudo usermod -aG input "$USER"
```

Sau khi thêm user vào group `input`, logout/login lại.

Kiểm tra:

```bash
ls -l /dev/uinput
getfacl /dev/uinput
groups
```

## 6. Cài GNOME Extension

Extension nằm trong:

```bash
gnome-extension/win11-clipboard-history@gustavosett.dev
```

Nó bắt `Super+V`, lấy vị trí con trỏ chuột từ GNOME Shell, rồi gọi:

```bash
win11-clipboard-history --show-at X Y
```

Cài dependency cho extension:

```bash
sudo apt install -y libglib2.0-bin gnome-shell-extension-prefs
```

Cài extension từ thư mục source:

```bash
scripts/install-gnome-extension.sh
```

Hoặc dùng Makefile:

```bash
make install-gnome-extension
```

Nếu command app không phải `win11-clipboard-history`, truyền command rõ ràng:

```bash
scripts/install-gnome-extension.sh "/duong/dan/toi/win11-clipboard-history"
```

Script sẽ:

- Copy extension vào `~/.local/share/gnome-shell/extensions/win11-clipboard-history@gustavosett.dev`.
- Compile schema bằng `glib-compile-schemas`.
- Set command app trong gsettings của extension.
- Giải phóng `Super+V` khỏi shortcut notification tray của GNOME nếu cần.
- Disable shortcut custom cũ của app nếu nó đang chiếm `Super+V`.
- Enable extension nếu `gnome-extensions` khả dụng.

Reload extension:

```bash
gnome-extensions disable win11-clipboard-history@gustavosett.dev
sleep 1
gnome-extensions enable win11-clipboard-history@gustavosett.dev
```

Kiểm tra:

```bash
gnome-extensions info win11-clipboard-history@gustavosett.dev
```

Kết quả mong đợi:

```text
Enabled: Yes
State: ACTIVE
```

Nếu `Super+V` chưa trigger sau khi cài/reload, logout/login lại để GNOME Shell nạp lại extension và schema.

## 7. Chạy App Nền

Dừng process cũ nếu đang chạy:

```bash
pids=$(pgrep -f '^/usr/bin/win11-clipboard-history-bin' || true)
if [ -n "$pids" ]; then
  kill $pids
  sleep 2
fi
```

Chạy app ở background:

```bash
nohup /usr/bin/win11-clipboard-history --background >/tmp/win11-clipboard-history.log 2>&1 &
```

Kiểm tra:

```bash
pgrep -af '^/usr/bin/win11-clipboard-history-bin'
tail -n 120 /tmp/win11-clipboard-history.log
```

## 8. Test Sau Khi Cài

1. Copy một đoạn text bất kỳ.
2. Click vào ô nhập text của app đích.
3. Đưa chuột đến vị trí muốn hiện popup.
4. Bấm `Super+V`.
5. Popup phải hiện gần vị trí con trỏ chuột.
6. Chọn item trong history.
7. Item phải được paste vào app đích.

Với terminal, app có thể cần paste bằng `Ctrl+Shift+V`. Cấu hình trong Settings:

```text
Paste Shortcuts -> Ctrl+Shift+V Targets
```

Mỗi pattern một dòng, ví dụ:

```text
terminal
gnome-terminal
kgx
konsole
alacritty
kitty
wezterm
tilix
terminator
xterm
```

## 9. Debug Nhanh

### Popup không mở tại vị trí con trỏ

Xem log GNOME extension:

```bash
journalctl --user -f -o cat | grep --line-buffered Win11ClipboardHistoryPointer
```

Khi bấm `Super+V`, log đúng sẽ có dạng:

```text
[Win11ClipboardHistoryPointer] keybinding fired at X, Y using win11-clipboard-history
```

Nếu không có `keybinding fired`, extension chưa bắt được shortcut hoặc shortcut bị ứng dụng khác chiếm.

### App không tự paste

Chạy app từ terminal để xem log:

```bash
pids=$(pgrep -f '^/usr/bin/win11-clipboard-history-bin' || true)
if [ -n "$pids" ]; then
  kill $pids
  sleep 2
fi
win11-clipboard-history
```

Chọn item trong popup. Log đúng thường có:

```text
[SimulatePaste] Sending Ctrl+V...
[uinput] Persistent virtual keyboard is ready
[SimulatePaste] Ctrl+V sent via uinput
```

Nếu log báo lỗi `/dev/uinput`, chạy lại:

```bash
sudo modprobe uinput
sudo setfacl -m u:$USER:rw /dev/uinput
```

### Extension đã cài nhưng không active

Kiểm tra danh sách extension:

```bash
gnome-extensions list | grep win11-clipboard
gnome-extensions info win11-clipboard-history@gustavosett.dev
```

Nếu cần cài lại:

```bash
scripts/install-gnome-extension.sh
gnome-extensions disable win11-clipboard-history@gustavosett.dev
sleep 1
gnome-extensions enable win11-clipboard-history@gustavosett.dev
```

Nếu vẫn không active, logout/login lại.

## 10. Gỡ Cài Đặt

Gỡ app `.deb`:

```bash
sudo apt remove win11-clipboard-history
```

Gỡ GNOME extension:

```bash
gnome-extensions disable win11-clipboard-history@gustavosett.dev
rm -rf ~/.local/share/gnome-shell/extensions/win11-clipboard-history@gustavosett.dev
```

Logout/login lại sau khi gỡ extension.

## 11. Lệnh Thường Dùng Khi Phát Triển

Build nhanh app release:

```bash
PATH="$HOME/.cargo/bin:$PATH" npm run tauri:build -- --bundles deb
```

Cài đè bản mới:

```bash
sudo dpkg -i ./src-tauri/target/release/bundle/deb/win11-clipboard-history_0.7.1_amd64.deb
```

Restart app nền:

```bash
pids=$(pgrep -f '^/usr/bin/win11-clipboard-history-bin' || true)
if [ -n "$pids" ]; then
  kill $pids
  sleep 2
fi
nohup /usr/bin/win11-clipboard-history --background >/tmp/win11-clipboard-history.log 2>&1 &
```

Reload extension:

```bash
scripts/install-gnome-extension.sh
gnome-extensions disable win11-clipboard-history@gustavosett.dev
sleep 1
gnome-extensions enable win11-clipboard-history@gustavosett.dev
```
