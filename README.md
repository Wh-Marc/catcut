# 🐱 CatCut

> **Ultra-lightweight, Cross-platform AI Video Editor with Native MCP (Model Context Protocol).**  
> Edit videos through natural language chat (Claude Desktop, Cursor, omp) and interactive GUI.

---

## ⚡ Highlights

- **Client siêu nhẹ (Ultra-lightweight)**:
  - Binary duy nhất chỉ **~5.3 MB** (so với CapCut gốc ~500MB+ và các ứng dụng Electron ~150MB+).
  - Không có CEF (Chromium Embedded Framework) bloat, không telemetry, không background tracking.
  - RAM khi chạy cực thấp (**< 80 MB**).
- **Native MCP 2024-11-05 Server**:
  - Tích hợp sẵn 17 công cụ biên tập video trực tiếp qua Model Context Protocol.
  - Hỗ trợ cả hai chế độ truyền tải: **Stdio** (cho Claude Desktop / Cursor / omp) và **HTTP/SSE**.
  - Tự động đồng bộ thời gian thực (Real-time bidirectional sync) giữa AI chat và giao diện người dùng.
  - Hỗ trợ công cụ `inspect_frame` trích xuất khung hình trực tiếp dưới dạng hình ảnh để LLM Multimodal "nhìn" thấy video khi đang dựng!
- **Tương thích CapCut 1:1**:
  - Nhập (`import_capcut_draft`) và xuất (`export_capcut_draft`) trực tiếp định dạng `draft_content.json` của CapCut.
  - Dựng thô bằng AI qua chat, sau đó mở lại trên CapCut máy tính mượt mà.
- **Đa nền tảng (Cross-platform)**:
  - Windows, macOS, Linux (kể cả WSL2).
  - Chạy dưới dạng Desktop App hoặc Web GUI cục bộ.

---

## 🚀 Cài đặt & Khởi chạy

### 1. Khởi chạy giao diện CatCut (Web & Desktop GUI)
```bash
# Chạy trực tiếp (tự động mở trình duyệt tại http://localhost:8765)
./target/release/catcut

# Hoặc chỉ định cổng và file dự án:
./target/release/catcut --port 8765 --project ./my_project.json
```

### 2. Khởi chạy dưới dạng MCP Server cho AI Chat (Stdio Mode)
```bash
./target/release/catcut --mode stdio --project ./my_project.json
```

---

## 🤖 Kết nối với AI qua MCP (Claude Desktop, Cursor, omp)

### A. Cấu hình cho Claude Desktop
Thêm vào file cấu hình Claude Desktop (`claude_desktop_config.json`):
```json
{
  "mcpServers": {
    "catcut": {
      "command": "/absolute/path/to/catcut/target/release/catcut",
      "args": ["--mode", "stdio", "--project", "/absolute/path/to/catcut/catcut_project.json"]
    }
  }
}
```

### B. Cấu hình cho Cursor / Remote Clients (SSE Mode)
Khi chạy CatCut ở chế độ UI (`./target/release/catcut`), thêm vào cấu hình MCP:
```json
{
  "mcpServers": {
    "catcut": {
      "url": "http://localhost:8765/sse"
    }
  }
}
```

---

## 🛠️ Danh sách 17 công cụ MCP (AI Editing Tools)

| Tên Tool | Mô tả |
| :--- | :--- |
| `get_timeline` | Lấy báo cáo chi tiết về timeline (Resolution, FPS, thời lượng, danh sách clip, track). |
| `import_media` | Probe file video/audio/ảnh bằng ffprobe và thêm vào thư viện dự án. |
| `add_clip` | Đặt một đoạn clip từ thư viện lên track ở vị trí thời gian chỉ định. |
| `split_clip` | Cắt / chia đôi một clip tại timestamp bất kỳ. |
| `trim_clip` | Cắt xén đoạn đầu hoặc thay đổi thời lượng của clip. |
| `remove_clip` | Xoá clip khỏi timeline (hỗ trợ `ripple: true` để tự dồn khoảng trống). |
| `move_clip` | Di chuyển clip sang thời điểm khác hoặc chuyển track. |
| `set_speed` | Thay đổi tốc độ phát (0.5x, 1.5x, 2x, 4x...). |
| `adjust_audio` | Điều chỉnh âm lượng (volume), fade-in, fade-out, mute. |
| `set_transform` | Chỉnh scale, position (X, Y), rotation và opacity. |
| `add_text` | Thêm tiêu đề / phụ đề chữ với tuỳ chỉnh font size và màu sắc. |
| `inspect_frame` | **Trích xuất ảnh snapshot tại một timestamp** gửi cho Multimodal AI quan sát. |
| `export_project` | Render video cuối cùng ra MP4 bằng FFmpeg. |
| `import_capcut_draft` | Nhập file `draft_content.json` của CapCut vào CatCut. |
| `export_capcut_draft` | Xuất timeline hiện tại ra file `draft_content.json` của CapCut. |
| `undo` | Hoàn tác thao tác chỉnh sửa gần nhất. |
| `redo` | Thực hiện lại thao tác vừa hoàn tác. |

---

## 💡 Ví dụ câu lệnh Chat để điều khiển CatCut

- *"Hãy import file `/path/to/video.mp4` và đặt lên timeline từ giây thứ 0"*
- *"Cắt bỏ đoạn từ 00:05 đến 00:10 và ripple delete khoảng trống"*
- *"Tăng tốc đoạn clip thứ 2 lên 2x và giảm âm lượng xuống 50%"*
- *"Thêm phụ đề 'Giới thiệu sản phẩm' từ giây thứ 2 đến giây thứ 5, màu vàng"*
- *"Chụp cho tôi khung hình ở giây thứ 3 để tôi xem thử"*
- *"Xuất video này ra file `final_video.mp4` ở độ phân giải 1080p"*
