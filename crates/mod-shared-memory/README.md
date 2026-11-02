# mod-shared-memory

Module bộ nhớ dùng chung giữa các agent trong Dandori (P3-10).
- Lưu trữ các mục sự thật (`fact`), quyết định (`decision`), quy ước (`convention`), và bài học (`lesson`).
- Chống nhiễm độc bộ nhớ (Memory Poisoning): Mọi mục do agent đề xuất luôn ở trạng thái `pending` và chỉ vào prompt sau khi con người phê duyệt.
- Lọc khóa riêng tư đa dòng (B-10) và bí mật hệ thống.
- Đóng góp vào prompt agent (`PromptContributor`) với ranh giới ngữ cảnh cách ly rõ ràng.
