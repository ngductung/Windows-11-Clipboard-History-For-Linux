# Windows 11 Clipboard History For Linux

Clipboard manager phong cách Windows 11 cho Linux, hỗ trợ Wayland/X11, mở nhanh bằng `Super+V`, lưu lịch sử clipboard local và có popup theo vị trí con trỏ trên GNOME Wayland thông qua extension đi kèm.

## Dành Cho Người Dùng Cuối

Phần này dành cho người chỉ muốn cài app và sử dụng hằng ngày.

### Cài Từ File `.deb` Trên Release

Với Ubuntu/Debian/Mint/Pop!_OS, người dùng cuối nên cài bằng file `.deb` đã build sẵn trong GitHub Releases.

1. Mở trang Releases.
2. Tải file `.deb` mới nhất, ví dụ:

```text
win11-clipboard-history_0.7.1_amd64.deb
```

3. Cài file vừa tải:

```bash
cd ~/Downloads
sudo apt install ./win11-clipboard-history_*_amd64.deb
```

4. Mở app:

```bash
win11-clipboard-history
```

Sau khi cài, app sẽ hướng dẫn cấu hình shortcut và quyền cần thiết trong Setup Wizard.

### Cài GNOME Extension Cho Wayland

Trên GNOME Wayland, nếu muốn `Super+V` mở popup đúng vị trí con trỏ chuột, cần cài GNOME extension đi kèm.

Nếu release có đính kèm gói extension, tải và giải nén theo hướng dẫn trong release đó. Nếu bạn đang có source code của project, cài extension bằng:

```bash
scripts/install-gnome-extension.sh
gnome-extensions disable win11-clipboard-history@gustavosett.dev
sleep 1
gnome-extensions enable win11-clipboard-history@gustavosett.dev
```

Kiểm tra extension:

```bash
gnome-extensions info win11-clipboard-history@gustavosett.dev
```

Kết quả mong đợi:

```text
Enabled: Yes
State: ACTIVE
```

Nếu `Super+V` chưa hoạt động ngay sau khi cài extension, logout/login lại để GNOME Shell nạp lại extension.

Hướng dẫn chi tiết cho Ubuntu GNOME Wayland nằm tại [docs/ubuntu-gnome-wayland-install.md](docs/ubuntu-gnome-wayland-install.md).

### Gỡ Cài Đặt

Gỡ app đã cài từ `.deb`:

```bash
sudo apt remove win11-clipboard-history
```

Gỡ GNOME extension:

```bash
gnome-extensions disable win11-clipboard-history@gustavosett.dev
rm -rf ~/.local/share/gnome-shell/extensions/win11-clipboard-history@gustavosett.dev
```

### Tính Năng Chính

| Tính năng | Mô tả |
| --- | --- |
| Clipboard history | Lưu và tìm lại nội dung clipboard đã copy. |
| `Super+V` | Mở popup clipboard nhanh. |
| Wayland và X11 | Hỗ trợ cả hai session phổ biến trên Linux. |
| GNOME Wayland pointer launcher | Mở popup tại vị trí con trỏ chuột bằng extension đi kèm. |
| Auto paste | Chọn item và paste trực tiếp vào app đích. |
| Terminal paste | Cấu hình app cần `Ctrl+Shift+V` thay vì `Ctrl+V`. |
| Pin item | Ghim nội dung quan trọng lên đầu danh sách. |
| Emoji picker | Tìm và paste emoji nhanh. |
| Local-first | Dữ liệu clipboard được lưu trên máy, không gửi ra ngoài. |

### Phím Tắt

| Phím | Hành động |
| --- | --- |
| `Super+V` | Mở Clipboard History |
| `Ctrl+Alt+V` | Shortcut thay thế |
| `Enter` | Paste item đang chọn |
| `Esc` | Đóng cửa sổ |

### Lỗi Thường Gặp

`Super+V` không mở app:

```bash
pgrep -f win11-clipboard-history-bin
```

Nếu app đang chạy nhưng shortcut không hoạt động, mở lại Setup Wizard:

```bash
rm ~/.config/win11-clipboard-history/setup.json
win11-clipboard-history
```

Chọn item nhưng không tự paste:

```bash
sudo modprobe uinput
sudo setfacl -m u:$USER:rw /dev/uinput
```

Để quyền `/dev/uinput` ổn định hơn sau reboot:

```bash
sudo usermod -aG input "$USER"
```

Sau khi thêm user vào group `input`, logout/login lại.

Terminal không paste đúng:

Một số terminal cần `Ctrl+Shift+V`. Mở Settings trong app, vào:

```text
Paste Shortcuts -> Ctrl+Shift+V Targets
```

Thêm tên terminal, mỗi pattern một dòng, ví dụ:

```text
gnome-terminal
kgx
konsole
alacritty
kitty
wezterm
```

## Dành Cho Developer

Phần này dành cho người muốn chạy source, sửa code, build package local hoặc phát triển GNOME extension.

Tech stack:

- Rust
- Tauri v2
- React
- TypeScript
- Tailwind CSS
- Linux desktop APIs

### Chuẩn Bị Môi Trường

```bash
git clone https://github.com/gustavosett/Windows-11-Clipboard-History-For-Linux.git
cd Windows-11-Clipboard-History-For-Linux
make deps
make rust
make node
source ~/.cargo/env
npm ci
```

Kiểm tra phiên bản toolchain:

```bash
node --version
npm --version
rustc --version
cargo --version
```

### Chạy Dev Mode

```bash
make dev
```

Hoặc:

```bash
npm run tauri:dev
```

### Kiểm Tra Trước Khi Build

```bash
npm run lint
PATH="$HOME/.cargo/bin:$PATH" cargo fmt --manifest-path src-tauri/Cargo.toml --check
PATH="$HOME/.cargo/bin:$PATH" cargo check --manifest-path src-tauri/Cargo.toml --locked
```

### Build File `.deb`

```bash
PATH="$HOME/.cargo/bin:$PATH" npm run tauri:build -- --bundles deb
```

File build ra nằm trong:

```bash
src-tauri/target/release/bundle/deb/
```

Cài lần đầu:

```bash
sudo apt install ./src-tauri/target/release/bundle/deb/win11-clipboard-history_0.7.1_amd64.deb
```

Cài đè cùng version khi đang phát triển:

```bash
sudo dpkg -i ./src-tauri/target/release/bundle/deb/win11-clipboard-history_0.7.1_amd64.deb
sudo apt -f install
```

### Cài GNOME Extension Từ Source

```bash
scripts/install-gnome-extension.sh
gnome-extensions disable win11-clipboard-history@gustavosett.dev
sleep 1
gnome-extensions enable win11-clipboard-history@gustavosett.dev
```

Kiểm tra extension:

```bash
gnome-extensions info win11-clipboard-history@gustavosett.dev
```

Kết quả mong đợi:

```text
Enabled: Yes
State: ACTIVE
```

### Restart App Nền Sau Khi Cài Đè

```bash
pids=$(pgrep -f '^/usr/bin/win11-clipboard-history-bin' || true)
if [ -n "$pids" ]; then
  kill $pids
  sleep 2
fi
nohup /usr/bin/win11-clipboard-history --background >/tmp/win11-clipboard-history.log 2>&1 &
```

Kiểm tra log:

```bash
tail -n 120 /tmp/win11-clipboard-history.log
```

### Debug GNOME Extension

```bash
journalctl --user -f -o cat | grep --line-buffered Win11ClipboardHistoryPointer
```

Khi bấm `Super+V`, log đúng thường có dạng:

```text
[Win11ClipboardHistoryPointer] keybinding fired at X, Y using win11-clipboard-history
```

## License

MIT. Xem [LICENSE](LICENSE).
