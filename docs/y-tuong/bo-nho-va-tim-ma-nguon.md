# Ý tưởng: tối ưu token, bộ nhớ "bản đồ não" và định vị mã nguồn

Ghi lại cuộc thảo luận ngày 2026-11-01 giữa chủ dự án và Claude. **Chưa triển khai.** Mục đích: không thất lạc, dùng làm đầu vào cho P2/P3/P4 (xem bảng "Gắn vào kế hoạch").

## 1. Vấn đề

| Vấn đề | Số liệu thật từ lần chạy Claude CLI đầu tiên (2026-11-01) |
| :--- | :--- |
| Mỗi lần chạy CLI phải trả "phí mở cửa" (nạp ngữ cảnh hệ thống của Claude Code) | khoảng 29.500 token ghi cache + 24.200 token đọc cache; `total_cost_usd` = 0,245 cho việc tạo một tệp |
| Cuộc trò chuyện dài phình dần, tới ngưỡng thì phải nén (từng làm công cụ sập) | cuộc trò chuyện lập kế hoạch đạt khoảng 371.000 token |
| Agent đọc cả tệp lớn để tìm chỗ sửa | chưa đo trong Dandori |

## 2. Ý tưởng của chủ dự án: "bộ nhớ theo bản đồ não"

Giống con người: nói chuyện về nhiều chủ đề (caching, DB, logic...), tới mức nào đó thì **nén lại, cất ở một nơi, cần thì gọi nhớ lại**. Claude đã làm một phần: nén hội thoại, tệp bộ nhớ có mục lục, skills nạp theo nhu cầu, agent con có ngữ cảnh riêng.

Thiết kế gợi ý (5 tầng): trí nhớ làm việc (lời nhắc lần chạy) · trí nhớ theo sự kiện (tóm tắt sau mỗi lần chạy) · trí nhớ khái niệm (mục lục + liên kết) · gọi nhớ chọn 3 đến 5 nút liên quan trong ngân sách token · củng cố/quên và **đối chiếu với mã thật** trước khi tin.

Rủi ro đã nêu: nhớ sai lan truyền (ví dụ bộ nhớ cũ về chính dự án này từng ghi "chưa cài Rust" khi đã xong hai giai đoạn); chèn lệnh qua bộ nhớ (ai ghi được là điều khiển được agent); va chạm với cache nếu phần nạp đổi theo từng task; bản thân việc tóm tắt cũng tốn token. **Hiệu quả thật chưa được chứng minh: phải đo.**

## 3. Nguồn đã kiểm chứng (đọc trang gốc ngày 2026-11-01)

| Chủ đề | Nguồn | Điều rút ra | Ghi chú |
| :--- | :--- | :--- | :--- |
| Bản đồ repo | [Aider](https://aider.chat/docs/repomap.html), 49,9k sao, Apache-2.0 | tree-sitter lấy ký hiệu và chữ ký, xếp hạng bằng đồ thị tệp, ngân sách mặc định 1k token (`--map-tokens`) | Trang gốc **không** nêu tên PageRank |
| Định vị theo tầng | [Agentless](https://arxiv.org/abs/2407.01489) | tệp, rồi lớp/hàm (chỉ đưa "bộ xương" khai báo), rồi dòng sửa; 32,00% SWE-bench Lite, $0,70/issue (tác giả ghi) | Số liệu 2024 |
| Giao diện agent | [SWE-agent](https://arxiv.org/abs/2405.15793) | cửa sổ xem 100 dòng 18,0% so với cả tệp 12,7%; tìm tối đa 50 kết quả; sửa có kiểm lỗi 18,0% so với 15,0%; giữ 5 quan sát gần nhất | Số liệu 2024 |
| Điều hướng ký hiệu | [Serena](https://github.com/oraios/serena), 31,0k sao | LSP + MCP, hơn 40 ngôn ngữ | **GPL-3.0**; README không có số đo |
| Ngữ cảnh | [Anthropic](https://www.anthropic.com/engineering/effective-context-engineering-for-ai-agents) | nạp đúng lúc (định danh nhẹ), nén, ghi chú ngoài ngữ cảnh, agent con trả 1.000 đến 2.000 token; "context rot" | 29/9/2025 |
| Cache | [Tài liệu](https://platform.claude.com/docs/en/build-with-claude/prompt-caching) | ghi cache 1,25 lần (5 phút) hoặc 2 lần (1 giờ); đọc 0,1 lần, **0,05 lần với Opus 5.5 và Sonnet 5.5**; hạn 5 phút, tự gia hạn khi dùng; khớp theo phần đầu giống hệt | |
| Bộ nhớ dài hạn | [Mem0](https://arxiv.org/abs/2504.19413) 68,2k sao; [Letta](https://github.com/letta-ai/letta) 25,3k; [Graphiti](https://github.com/getzep/graphiti) 32,4k | Mem0: trên 90% chi phí token và 91% độ trễ p95 so với nhồi cả hội thoại | Đo trên hội thoại (LOCOMO), do chính tác giả công bố; **không suy ra được cho việc sửa mã** |
| CLAUDE.md | [Tài liệu](https://code.claude.com/docs/en/memory) | nên dưới 200 dòng; tệp thư mục con chỉ nạp khi đụng tới | |

## 4. Gắn vào kế hoạch (backlog, chưa giao việc)

| Mã | Việc | Gắn vào | Căn cứ |
| :-: | :--- | :--- | :--- |
| B-01 | Module `code-map`: bản đồ repo bằng tree-sitter (khoảng 1k token) + gợi ý `tệp:dòng` lấy từ test hỏng/dấu vết lỗi, đưa vào lời nhắc | đầu P3 (sau P2) | Aider, Agentless |
| B-02 | `context-budget`: lời nhắc xếp phần ổn định lên đầu; mô hình và `--effort` theo vai trò; `--max-budget-usd`; gom lần chạy trong 5 phút | P2-12 trở đi | Tài liệu cache; cờ CLI đã kiểm: `--model`, `--effort`, `--max-budget-usd`, `--tools`, `--disable-slash-commands`, `--strict-mcp-config` |
| B-03 | "Bản đồ não": tóm tắt sau mỗi lần chạy, mục lục chủ đề, chọn nút liên quan, đối chiếu với mã | P3-10, P3-11, P4-07 | Anthropic, MemGPT/Letta, Graphiti |
| B-04 | Đo thật: thêm `-ExtraArgs` vào `docs/agents/chay-thu-cli-that.ps1`, so `total_cost_usd` giữa các tổ hợp (có/không bản đồ, mô hình, cờ giảm nạp) | trước B-01/B-02 | Mọi con số tiết kiệm ở trên là ước đoán |
| B-05 | Chạy thật `agy` và sửa bộ đọc đầu ra theo mẫu thật | trước khi dùng `agy` hằng ngày | Mới có `claude` được kiểm |
| B-06 | Giới hạn chạy đồng thời có thể bị vượt vài lần khi yêu cầu sát nhau | khi rảnh | Phát hiện lúc kiểm P1 |
| B-07 | Dọn cảnh báo `ts-rs failed to parse serde attribute` | khi rảnh | Chỉ là nhiễu lúc build |
| B-08 | Biên dịch và kiểm Linux/macOS (`rustup target add x86_64-unknown-linux-gnu`) | trước P5 | Mã `#[cfg(unix)]` chưa từng biên dịch |
| B-09 | Mở giao diện trên trình duyệt thật nối máy chủ thật | trước P5 | Mới kiểm bằng vitest |
| B-10 | Che khóa riêng nhiều dòng (`-----BEGIN ... PRIVATE KEY-----`) trong đầu ra agent | P3-10 | Bộ che hiện chỉ bắt tiền tố một dòng |

## 5. Nguyên tắc khi làm

1. **Đo trước, tin sau.** Mỗi mục B-01 đến B-03 phải có kịch bản so sánh (chi phí, tỉ lệ thành công) với chạy không có tính năng đó; không có lợi rõ ràng thì bỏ.
2. Không nhúng Mem0/Letta/Graphiti cho việc sửa mã: bằng chứng của chúng là về hội thoại.
3. Serena (GPL-3.0): chỉ dùng như công cụ ngoài qua MCP, không nhúng vào mã Rust của Dandori khi chưa quyết định giấy phép.
4. Bộ nhớ là bề mặt tấn công (chèn lệnh); thiết kế theo ADR 0011 (P3-10) và luôn kiểm quyền workspace.
