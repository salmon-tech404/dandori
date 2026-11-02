# Chạy thử Dandori với CLI agent thật (`DAN-VERIFY-003`, `DAN-VERIFY-004`)

Mục tiêu: xem Dandori giao một việc nhỏ cho **Claude Code** (hoặc **Agy**) thật, agent làm việc trong một nhánh git riêng, và kho git chính không bị đụng tới.

**Việc này dùng hạn mức tài khoản của bạn** (một lời nhắc rất ngắn, tốn rất ít). Script chỉ làm việc trên một repo git **tạm** do chính nó tạo, không đụng repo của bạn.

## Chuẩn bị (một lần)

- Đã cài và chạy được `claude --version` (hoặc `agy --version`) trong PowerShell.
- Đã commit mã P1 (không bắt buộc, nhưng nên làm để khỏi mất).
- Hai cửa sổ PowerShell: **A** (chạy máy chủ) và **B** (chạy script). Có thể thêm cửa sổ **C** chỉ để đăng nhập CLI.

## Bước 1. Cửa sổ A: chạy máy chủ Dandori

```powershell
cd D:\19-dandori
$env:DANDORI_DATA_DIR = "D:\dandori-real-test-data"
cargo run -p dandori-server
```

Để nguyên cửa sổ này, thấy dòng "Máy chủ Dandori đang lắng nghe" là được. Lần đầu build có thể mất vài phút.

> Không đặt `DANDORI_ENABLE_FAKE_PROVIDER`. Cờ đó chỉ để kiểm thử với agent giả.

## Bước 2. Cửa sổ B: chạy script

```powershell
cd D:\19-dandori
.\docs\agents\chay-thu-cli-that.ps1 -Provider claude
```

Muốn thử Agy: `-Provider agy`. Nếu PowerShell chặn script, chạy trước: `Set-ExecutionPolicy -Scope Process Bypass`.

Script sẽ tự làm: tạo chủ hệ thống, bật module, tạo workspace và dự án trên repo tạm, tạo **tài khoản agent**.

## Bước 3. Đăng nhập CLI vào thư mục riêng (bước thủ công duy nhất)

Script in ra một **lệnh dài** bắt đầu bằng `$env:USERPROFILE=...`. Lệnh này đặt thư mục HOME riêng cho tài khoản rồi chạy `claude auth login`. Làm như sau:

1. Mở **cửa sổ PowerShell mới (C)**.
2. **Dán nguyên lệnh** đó, nhấn Enter.
3. Trình duyệt mở ra, đăng nhập tài khoản Claude của bạn.
4. Quay lại cửa sổ B, nhấn **Enter**.

Vì sao phải dùng HOME riêng: đó là cách Dandori giữ phiên đăng nhập của từng tài khoản tách biệt (`DAN-VERIFY-004`). Phiên đăng nhập này **không** ảnh hưởng đến Claude Code bạn đang dùng hằng ngày.

Với `agy`: sau khi đăng nhập xong thì thoát phiên `agy`. Dandori chưa có lệnh tự kiểm tra cho agy nên script xác nhận thay bạn.

Script kiểm tra lại: nếu thấy `ready` là đăng nhập thành công; nếu thấy `not_logged_in`, lệnh đăng nhập chưa thành công trong thư mục HOME riêng, hãy chạy lại bước 3.

## Bước 4. Chờ kết quả

Script tạo một task ("tạo tệp HELLO.txt") rồi giao cho agent. Đợi tối đa 5 phút. Cuối cùng nó in một **bảng tổng kết**. Các dòng cần thấy `True`:

| Dòng | Ý nghĩa |
|---|---|
| Agent chạy xong | Claude/Agy chạy và kết thúc bình thường |
| Agent tạo được HELLO.txt trong worktree | Agent thật sự làm việc trong thư mục riêng của task |
| Có nhánh git riêng cho task | Mỗi task một nhánh |
| Repo chính không bị sửa | Kho git gốc nguyên vẹn |

## Bước 5. Gửi kết quả cho Claude

- **Nếu tất cả đạt:** gửi cho tôi bảng tổng kết (copy từ cửa sổ B).
- **Nếu có dòng lỗi (kể cả chỉ một):** gửi cho tôi **toàn bộ nội dung cửa sổ B**, và vài dòng cuối của cửa sổ A. Đừng cố sửa. Lỗi ở bước này rất quý vì nó cho biết chỗ đoán sai về CLI thật.

## Dọn dẹp

```powershell
Remove-Item -Recurse -Force D:\dandori-real-test-data
Remove-Item -Recurse -Force "$env:TEMP\dandori-real-repo-*"
```

Tắt máy chủ ở cửa sổ A bằng Ctrl+C.

## Những điều CHƯA được kiểm chứng (nói trước để bạn không bất ngờ)

- Định dạng đầu ra thật của `claude -p --output-format stream-json`: Dandori đang đọc đoán. Nếu cột "đầu ra agent" hiện dòng thô thay vì chữ của Claude thì đó là lỗi cần sửa, không phải lỗi của bạn.
- `--dangerously-skip-permissions`: agent chạy mọi lệnh **không hỏi**. Vì vậy script chỉ làm trên repo tạm.
- Nút **Hủy** đã được kiểm với agent giả (agent bị diệt thật), chưa kiểm với Claude/Agy thật.
- Với `agy`, cờ `--print --output-format stream-json` có trong `agy --help` nhưng chưa từng chạy thử một lần đầy đủ.

---

## Phụ lục: giải thích từng lệnh (làm gì, để làm gì, vì sao)

**Biến môi trường (`$env:TÊN = "giá trị"`)** là "ghi chú dán lên cửa sổ PowerShell" mà chương trình chạy trong cửa sổ đó đọc được. Chỉ có hiệu lực trong **đúng cửa sổ đó**; đóng cửa sổ là mất, không đổi cài đặt máy.

| Lệnh | Làm gì | Vì sao cần |
|---|---|---|
| `cd D:9-dandori` | Vào thư mục dự án | Các lệnh sau tìm tệp theo thư mục hiện tại |
| `$env:DANDORI_DATA_DIR = "D:\dandori-real-test-data"` | Bảo Dandori lưu dữ liệu (cơ sở dữ liệu, thư mục HOME riêng của từng tài khoản, thư mục làm việc của task) vào đây | Để dữ liệu thử nằm riêng, dọn một lần là sạch, không lẫn dữ liệu thật sau này. Nếu không đặt, Dandori dùng thư mục mặc định trong `%APPDATA%` |
| `cargo run -p dandori-server` | `cargo` là công cụ build của Rust; `run` biên dịch rồi chạy; `-p dandori-server` chọn chương trình máy chủ (dự án có nhiều phần) | Máy chủ chỉ lắng nghe `127.0.0.1:3000`, tức chỉ máy bạn truy cập được |
| `Set-ExecutionPolicy -Scope Process Bypass` | Cho phép chạy tệp `.ps1` | Windows mặc định chặn script. `-Scope Process` nghĩa là chỉ trong cửa sổ này |
| `.\docsgents\chay-thu-cli-that.ps1 -Provider claude` | Chạy script; `.\` nghĩa là "tệp ở thư mục hiện tại"; `-Provider` chọn CLI cần thử | Script gọi API của Dandori thay bạn |

**Lệnh dài ở bước 3** (script in ra):

| Phần | Ý nghĩa |
|---|---|
| `$env:USERPROFILE` và `$env:HOME` | "Thư mục nhà" mà CLI dùng để lưu phiên đăng nhập và cấu hình. Đổi sang thư mục riêng của tài khoản |
| `$env:APPDATA` và `$env:LOCALAPPDATA` | Thư mục dữ liệu ứng dụng của Windows; cũng trỏ vào thư mục riêng |
| `claude auth login` | Lệnh đăng nhập chính thức của Claude. Bạn đăng nhập trực tiếp với Anthropic trên trình duyệt; Dandori không biết và không lưu mật khẩu của bạn |

Vì sao cần cửa sổ mới (C): bốn biến trên làm cho Claude tưởng "nhà của nó" nằm ở thư mục riêng. Đặt trong cửa sổ C thì chỉ cửa sổ C bị đổi; Claude Code bạn dùng hằng ngày (đăng nhập ở thư mục nhà thật) **không bị ảnh hưởng**. Nhờ vậy mỗi tài khoản có một phiên đăng nhập tách biệt.

**Khi giao việc**, Dandori chạy `claude -p --output-format stream-json --verbose --dangerously-skip-permissions -- "<lời nhắc>"` với cùng các biến HOME riêng:

| Cờ | Ý nghĩa |
|---|---|
| `-p` | Chạy một lần, không mở giao diện tương tác |
| `--output-format stream-json` | In kết quả theo từng dòng JSON để Dandori đọc và hiển thị trực tiếp |
| `--verbose` | CLI bắt buộc phải có khi dùng `stream-json` (đã thử: thiếu thì báo lỗi) |
| `--dangerously-skip-permissions` | Agent chạy lệnh mà **không hỏi quyền**. Cần vì không có người ngồi canh; rủi ro là agent làm gì cũng được, nên chỉ chạy trên repo tạm |
| `--` | Hết phần tùy chọn, phía sau là lời nhắc (để lời nhắc bắt đầu bằng "-" không bị hiểu nhầm) |

**Dọn dẹp:** `Remove-Item -Recurse -Force <đường dẫn>` xóa thư mục cùng toàn bộ nội dung bên trong và **không hỏi lại**. Hãy đọc kỹ đường dẫn trước khi nhấn Enter.
