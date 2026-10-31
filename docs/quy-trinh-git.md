# Quy trình Git của Dandori

Chủ dự án tự commit và tự đẩy mã (agent không đụng vào git). Tài liệu này là **luật chung** để lịch sử luôn gọn, dễ đọc, dễ tìm.

## 1. Năm nguyên tắc

1. **Mọi tên trong Git bằng tiếng Anh**: tên nhánh, thông điệp commit, tên tag, tiêu đề và nội dung Pull Request. Còn tài liệu, báo cáo và trò chuyện bằng tiếng Việt.
2. **Một nhánh cho mỗi giai đoạn** (xem mục 2). Không dồn nhiều giai đoạn vào một nhánh.
3. **Mỗi task duyệt xong là một commit** (hoặc vài commit nhỏ nếu task chạm hai mục đích khác nhau).
4. **`main` chỉ nhận nhánh đã qua cổng giai đoạn** và đã được Claude kiểm độc lập. Không commit thẳng vào `main`.
5. **Không bao giờ** đẩy ép (`--force`) lên `main`, không commit bí mật (khóa, token, `.env`).

## 2. Nhánh theo giai đoạn

| Giai đoạn | Tên nhánh | Task | Cổng |
| :--- | :--- | :--- | :--- |
| P0 Lõi và Option | `phase/p0-core` | T01, T01B, T02 ... T14 | T14 |
| P1 Chạy được một task | `phase/p1-run-one-task` | P1-01 ... P1-09 | P1-09 |
| P2 Vòng làm việc đầy đủ | `phase/p2-full-workflow` | P2-01 ... P2-13 | P2-13 |
| P3 Tự động và tri thức | `phase/p3-automation-knowledge` | P3-01 ... P3-15 | P3-15 |
| P4 Nặng và rủi ro | `phase/p4-heavy-risky` | P4-01 ... P4-10 | P4-10 |
| P5 Phát hành | `phase/p5-release` | P5-01, P5-02 | P5-02 |

Mỗi nhánh giai đoạn **tạo từ `main` mới nhất lúc bắt đầu giai đoạn đó** (không tạo trước, vì sẽ lỗi thời).

## 3. Quy tắc đặt tên nhánh

Dạng: `<kind>/<name>`

| Kind | Dùng khi | Ví dụ |
| :--- | :--- | :--- |
| `phase/` | Nhánh của một giai đoạn | `phase/p0-core` |
| `fix/` | Sửa lỗi **sau khi** giai đoạn đã gộp vào `main` | `fix/p0-registry-lock-order` |
| `spike/` | Thử nghiệm rủi ro, có thể bỏ | `spike/p4-sandbox-windows` |
| `docs/` | Chỉ sửa tài liệu | `docs/git-workflow` |

`<name>`: chữ thường, chỉ gồm `a-z`, `0-9` và dấu `-`; **tiếng Anh, không dấu, không khoảng trắng**; ngắn gọn (khoảng 40 ký tự trở xuống) và mô tả mục đích.

Không đặt tên: `test`, `new`, `tmp`, `my-branch`, hay tiếng Việt không dấu như `loi`, `chay`.

## 4. Quy tắc thông điệp commit

Dạng (Conventional Commits):

```text
<type>(<scope>): <short description>

<optional body: why, not what>

Task: <task id>
```

- **Dòng đầu** tối đa khoảng 72 ký tự, chữ thường, không dấu chấm cuối, dùng **động từ nguyên mẫu**: `add`, `fix`, `rename`, `remove`, `update`.
- **`<type>`:** `feat` (thêm tính năng), `fix` (sửa lỗi), `refactor` (đổi cấu trúc, không đổi hành vi), `test`, `docs`, `chore` (việc vặt, cấu hình), `build`, `perf`.
- **`<scope>`** (khu vực, chọn một): `workspace`, `types`, `db`, `events`, `vault`, `process`, `agent`, `git`, `module-api`, `registry`, `server`, `modules`, `e2e`, `frontend`, `scripts`, `tooling`, `docs`, `adr`, `plan`, `deps`, hoặc `<id>` của một module (ví dụ `kanban`).
- **Dòng cuối** `Task: T03` giúp lần theo từng task.

| Nên | Không nên |
| :--- | :--- |
| `feat(registry): compute effective feature state per scope` | `Cập nhật registry` (tiếng Việt) |
| `fix(db): correct typo in crate docs` | `fix bug` (mơ hồ) |
| `chore(workspace): rename web to frontend` | `update stuff` |
| `docs(adr): add source structure decision` | `Added a lot of changes to many files and some docs too.` (dài, không rõ) |

## 5. Vòng đời một giai đoạn

**Bắt đầu giai đoạn** (lấy `main` mới nhất rồi tạo nhánh):

```powershell
git switch main
git pull
git switch -c phase/p1-run-one-task
```

**Sau mỗi task đã duyệt:**

```powershell
git add <specific paths>
git commit -m "<type>(<scope>): <short description>" -m "Task: <id>"
git push            # lần đầu của nhánh: git push -u origin <branch>
```

**Tới cổng giai đoạn:**

1. Claude kiểm độc lập (`reports/REVIEW_P<N>.md`).
2. Mở **Pull Request** từ nhánh giai đoạn vào `main`. Tiêu đề tiếng Anh, ví dụ `feat: complete phase 0 core`.
3. Gộp bằng **"Create a merge commit"** (không dùng "Squash"), để mỗi commit task vẫn thấy được trong lịch sử.
4. Cập nhật `main` và tạo nhánh giai đoạn kế:

```powershell
git switch main
git pull
git switch -c phase/p1-run-one-task
```

**Tùy chọn:** đánh dấu cổng bằng tag (tiếng Anh), ví dụ `git tag -a p0-done -m "Phase 0 complete"` rồi `git push origin p0-done`.

## 6. Đổi tên nhánh đã có (làm một lần)

Nhánh cũ `build/p0-loi` đã đẩy lên GitHub. Đổi sang tên chuẩn:

```powershell
git branch -m build/p0-loi phase/p0-core      # đổi tên trên máy
git push -u origin phase/p0-core              # đẩy tên mới, gắn theo dõi
git push origin --delete build/p0-loi         # xóa tên cũ trên GitHub
```

Lệnh thứ ba **xóa nhánh trên GitHub**; chạy sau khi lệnh thứ hai báo thành công và bạn đã thấy nhánh mới trên GitHub.

## 7. Lịch sử cũ

Các commit đã có (`first commit`, và ba commit đầu bằng tiếng Việt) **giữ nguyên**, không viết lại lịch sử đã đẩy. Quy ước tiếng Anh áp dụng **từ nay về sau**.
