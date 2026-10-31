# 00. Mẫu báo cáo task

Lưu ở `instructions/implementations/reports/<ID>.md`. Giữ đúng thứ tự mục. Không bỏ mục nào; không có gì thì ghi "Không có".

```markdown
# Báo cáo <ID>: <tên task>

- Ngày giờ: <lấy từ lệnh date, không đoán>
- Trạng thái cây làm việc: <đầu ra git status --porcelain, nguyên văn> (agent không commit; D-21)
- Trạng thái: DONE | DONE_WITH_GAPS | BLOCKED

## Tóm tắt cho người không rành kỹ thuật
<3–5 câu tiếng Việt, không thuật ngữ: đã làm được gì, dùng được chưa, còn thiếu gì>

## Đã làm
<danh sách file tạo/sửa, lấy từ `git status --porcelain` và `git diff --stat`; dán nguyên văn>

## Tiêu chí nghiệm thu
| # | Tiêu chí (chép từ file task) | Kết quả | Bằng chứng |
|---|---|---|---|
| 1 | ... | Qua / Không qua / Chưa chạy được | <lệnh và đầu ra nguyên văn, hoặc đường dẫn file> |

## Đầu ra pnpm run verify (máy Dev)
<dán nguyên văn>

## QC1 (đối chiếu spec, kiến trúc, Rust, Kantanda)
<bảng vi phạm tìm thấy và đã sửa; "Không có" nếu không>

## Đối chiếu giao diện (chỉ task có giao diện)
<ảnh chụp 1440×900 sáng và tối ở reports/<ID>-screens/; bảng khác biệt so với ui-reference/*.png (bố cục, nhãn, trạng thái, màu) kèm lý do từng khác biệt; danh sách ô cắm (slot) đã làm>

## QC2 (bản sao sạch)
<lệnh đã chạy, đầu ra nguyên văn của pnpm install --frozen-lockfile và pnpm run verify>

## Kiểm đường dẫn được bảo vệ
<đầu ra lệnh ở 00_QUY_TAC_CHUNG mục 5; phải rỗng>

## Kiểm khóa cấu hình (từ sau mốc khóa)
<đầu ra git diff <mốc-khóa> -- <các tệp>; chỉ được có dòng thêm>

## Chưa kiểm chứng / chờ người dùng kiểm
<mọi thứ cần CLI, tài khoản hay dịch vụ thật; mã TSU... → dùng mã DAN-VERIFY-nnn>

## Quyết định tự đưa ra
<mục, lựa chọn, lý do, hệ quả; "Không có" nếu không>

## Đề xuất commit (cho chủ dự án)
```text
git add <đường dẫn cụ thể 1> <đường dẫn cụ thể 2> ...
git commit -m "<type>(<scope>): <short English description>" -m "Task: <ID>"
```
<nếu task nên tách thành nhiều commit thì ghi từng cặp git add / git commit>
<thông điệp commit: TIẾNG ANH, ngắn gọn, động từ nguyên mẫu, theo 03 mục 6>

## Đề xuất (ngoài phạm vi, KHÔNG làm)
<ý hay nhưng ngoài task>
```
