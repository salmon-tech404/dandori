# Chạy thử Dandori với CLI agent thật (DAN-VERIFY-003 / DAN-VERIFY-004).
#
# Cách dùng (xem docs/agents/chay-thu-cli-that.md):
#   1. Terminal A: chạy máy chủ Dandori (xem hướng dẫn).
#   2. Terminal B: .\docs\agents\chay-thu-cli-that.ps1 -Provider claude
#
# Script chỉ gọi API của Dandori trên 127.0.0.1; không đọc thư mục cấu hình thật của CLI.
# Nó tạo một repo git TẠM (không đụng repo của bạn) và chạy một lời nhắc rất ngắn.

param(
    [ValidateSet("claude", "agy")]
    [string]$Provider = "claude",
    [int]$Port = 3000,
    [string]$Username = "owner",
    [string]$Password = "OwnerPassword123!",
    [string]$Prompt = "Create a file named HELLO.txt in the current directory containing exactly one line: hello from dandori. Do nothing else, then stop.",
    [int]$TimeoutSeconds = 300,
    # Chỉ để tự kiểm script: chạy với nhà cung cấp giả (máy chủ phải bật DANDORI_ENABLE_FAKE_PROVIDER=1).
    [switch]$UseFake
)

$ErrorActionPreference = "Stop"
$base = "http://127.0.0.1:$Port"
$session = New-Object Microsoft.PowerShell.Commands.WebRequestSession
$results = New-Object System.Collections.ArrayList

function Add-Result([string]$name, [bool]$ok, [string]$detail = "") {
    [void]$results.Add([pscustomobject]@{ Buoc = $name; Dat = $ok; ChiTiet = $detail })
    $mark = if ($ok) { "[ĐẠT]" } else { "[LỖI]" }
    Write-Host "$mark $name  $detail"
}

function Invoke-Api([string]$method, [string]$path, $body = $null) {
    $args2 = @{ Uri = "$base$path"; Method = $method; WebSession = $session; ContentType = "application/json"; UseBasicParsing = $true }
    # Gửi UTF-8 (PowerShell 5.1 mặc định mã hóa chuỗi sai nên tiếng Việt có dấu làm JSON hỏng, máy chủ trả 400).
    if ($null -ne $body) { $args2.Body = [System.Text.Encoding]::UTF8.GetBytes(($body | ConvertTo-Json -Depth 10 -Compress)) }
    try {
        $resp = Invoke-WebRequest @args2
        $json = $null
        if ($resp.Content) { $json = $resp.Content | ConvertFrom-Json }
        return [pscustomobject]@{ Status = [int]$resp.StatusCode; Body = $json }
    }
    catch {
        $status = 0
        $json = $null
        if ($_.Exception.Response) {
            $status = [int]$_.Exception.Response.StatusCode
            try {
                $reader = New-Object System.IO.StreamReader($_.Exception.Response.GetResponseStream())
                $text = $reader.ReadToEnd()
                if ($text) { $json = $text | ConvertFrom-Json }
            } catch { }
        }
        return [pscustomobject]@{ Status = $status; Body = $json }
    }
}

$providerName = if ($UseFake) { "fake" } else { $Provider }

# 1. Máy chủ
$health = Invoke-Api "GET" "/api/v1/health"
Add-Result "Máy chủ đang chạy ($base)" ($health.Status -eq 200) "mã $($health.Status)"
if ($health.Status -ne 200) { Write-Host "Hãy chạy máy chủ trước (xem hướng dẫn)."; exit 1 }

# 2. Chủ hệ thống
$setup = Invoke-Api "POST" "/api/v1/auth/setup-owner" @{ username = $Username; password = $Password }
if ($setup.Status -eq 409) {
    $setup = Invoke-Api "POST" "/api/v1/auth/login" @{ username = $Username; password = $Password }
    Add-Result "Đăng nhập chủ hệ thống (đã tồn tại)" ($setup.Status -eq 200) "mã $($setup.Status)"
} else {
    Add-Result "Tạo chủ hệ thống" ($setup.Status -eq 200) "mã $($setup.Status)"
}
if ($setup.Status -ne 200) { exit 1 }

# 3. Bật module của gói "Solo nhẹ"
$packs = Invoke-Api "GET" "/api/v1/features/packs"
$solo = $packs.Body.packs | Where-Object { $_.id -eq "solo-light" }
$allApplied = $true
foreach ($m in $solo.modules) {
    $r = Invoke-Api "POST" "/api/v1/features/apply" @{ module_id = $m; scope = "system"; target_id = "system"; enable = $true; risk_acknowledged = $true }
    if ($r.Status -ne 200) { $allApplied = $false; Write-Host "  không bật được $m (mã $($r.Status))" }
}
Add-Result "Bật các module của gói Solo nhẹ" $allApplied ($solo.modules -join ", ")

# 4. Repo git tạm, workspace, dự án
$repo = Join-Path $env:TEMP ("dandori-real-repo-" + [guid]::NewGuid().ToString("N").Substring(0, 8))
New-Item -ItemType Directory -Path $repo | Out-Null
git -C $repo init -b main | Out-Null
git -C $repo config user.email "test@dandori.local"
git -C $repo config user.name "Dandori Test"
"# Repo thử Dandori" | Set-Content -Path (Join-Path $repo "README.md") -Encoding utf8
git -C $repo add -A | Out-Null
git -C $repo commit -m "init" | Out-Null

$ws = Invoke-Api "POST" "/api/v1/workspaces" @{ name = "Workspace thử CLI thật" }
$wsId = $ws.Body.workspace.id
$proj = Invoke-Api "POST" "/api/v1/workspaces/$wsId/projects" @{ name = "Dự án thử"; repo_path = $repo; default_branch = "main" }
Add-Result "Tạo workspace và dự án (repo tạm $repo)" ($proj.Status -eq 200) "mã $($proj.Status)"
if ($proj.Status -ne 200) { exit 1 }
$projId = $proj.Body.project.id

# 5. Tài khoản agent + đăng nhập trong HOME cô lập
$acc = Invoke-Api "POST" "/api/v1/m/subscription-connect/accounts" @{ provider = $providerName; label = "Tài khoản thử $providerName" }
Add-Result "Tạo tài khoản agent ($providerName)" ($acc.Status -eq 201) "trạng thái đầu: $($acc.Body.status)"
if ($acc.Status -ne 201) { exit 1 }
$accId = $acc.Body.id

if ($acc.Body.status -eq "not_installed") {
    Write-Host "CLI '$providerName' chưa được cài (không tìm thấy trên PATH)."; exit 1
}

if (-not $UseFake) {
    $li = Invoke-Api "GET" "/api/v1/m/subscription-connect/accounts/$accId/login-instructions"
    Write-Host ""
    Write-Host "=== BƯỚC THỦ CÔNG: ĐĂNG NHẬP CLI VÀO THƯ MỤC HOME CÔ LẬP ==="
    Write-Host "Mở một cửa sổ PowerShell MỚI, dán nguyên lệnh dưới đây, hoàn tất đăng nhập trên trình duyệt:"
    Write-Host ""
    Write-Host $li.Body.command
    Write-Host ""
    if ($Provider -eq "agy") {
        Write-Host "(agy: sau khi đăng nhập xong, thoát khỏi phiên agy. Dandori chưa có lệnh tự kiểm tra cho agy nên bạn xác nhận thủ công.)"
    }
    Read-Host "Đăng nhập xong thì nhấn Enter ở đây"

    $confirm = ($Provider -eq "agy")
    $body = @{}
    if ($confirm) { $body = @{ confirm_logged_in = $true } }
    $chk = Invoke-Api "POST" "/api/v1/m/subscription-connect/accounts/$accId/check" $body
    Add-Result "Kiểm tra trạng thái đăng nhập" ($chk.Body.status -eq "ready") "trạng thái: $($chk.Body.status)"
    if ($chk.Body.status -ne "ready") {
        Write-Host "Tài khoản chưa ở trạng thái 'ready' (đang là '$($chk.Body.status)'). Dừng ở đây."
        Write-Host "Nếu là not_logged_in: lệnh đăng nhập chưa thành công trong thư mục HOME cô lập."
        exit 1
    }
}

# 6. Task + giao việc
if ($UseFake) {
    $desc = (@{ write_file = @{ path = "HELLO.txt"; content = "hello from dandori" }; print_lines = @("working"); exit_code = 0 } | ConvertTo-Json -Compress -Depth 5)
} else {
    $desc = $Prompt
}
$task = Invoke-Api "POST" "/api/v1/projects/$projId/tasks" @{ title = "Tạo HELLO.txt"; description = $desc }
$taskId = $task.Body.task.id
Add-Result "Tạo task" ($task.Status -eq 200) "mã $($task.Status)"
$disp = Invoke-Api "POST" "/api/v1/tasks/$taskId/dispatch"
Add-Result "Giao việc cho agent" ($disp.Status -eq 201) "mã $($disp.Status) $($disp.Body.error.message)"
if ($disp.Status -ne 201) { exit 1 }
$runId = $disp.Body.run_id

# 7. Chờ chạy xong
$deadline = (Get-Date).AddSeconds($TimeoutSeconds)
$final = $null
Write-Host "Đang chờ agent chạy (tối đa $TimeoutSeconds giây)..."
while ((Get-Date) -lt $deadline) {
    Start-Sleep -Seconds 3
    $run = Invoke-Api "GET" "/api/v1/runs/$runId"
    $status = if ($run.Body.run) { $run.Body.run.status } else { $run.Body.status }
    if ($status -and $status -ne "running") { $final = $status; break }
}
if (-not $final) {
    Add-Result "Agent chạy xong trong thời hạn" $false "quá $TimeoutSeconds giây, thử hủy"
    Invoke-Api "POST" "/api/v1/runs/$runId/cancel" | Out-Null
} else {
    Add-Result "Agent chạy xong" ($final -in @("success", "succeeded", "done", "completed", "finished")) "trạng thái: $final"
}

# 8. Kết quả
$events = Invoke-Api "GET" "/api/v1/runs/$runId/events?limit=200"
Write-Host ""
Write-Host "=== ĐẦU RA AGENT (tối đa 30 dòng đầu) ==="
$items = @($events.Body.events)
if (-not $items.Count -and $events.Body.items) { $items = @($events.Body.items) }
$items | Select-Object -First 30 | ForEach-Object { Write-Host ("  [" + $_.kind + "] " + $_.data) }

$wt = Invoke-Api "GET" "/api/v1/m/worktree/tasks/$taskId/worktree"
$wtPath = $wt.Body.workdir_path
Write-Host ""
Write-Host "Thư mục làm việc của task: $wtPath"
$hello = if ($wtPath) { Join-Path $wtPath "HELLO.txt" } else { "" }
$created = ($hello -ne "") -and (Test-Path $hello)
Add-Result "Agent tạo được HELLO.txt trong worktree" $created $(if ($created) { (Get-Content $hello -Raw).Trim() } else { "không thấy tệp" })
$branches = (git -C $repo branch --list "task/*") -join " "
Add-Result "Có nhánh git riêng cho task" ($branches -match "task/") $branches
$mainDirty = (git -C $repo status --porcelain)
Add-Result "Repo chính không bị sửa" (-not $mainDirty) $(if ($mainDirty) { "$mainDirty" } else { "sạch" })

Write-Host ""
Write-Host "================ TỔNG KẾT ================"
$results | Format-Table -AutoSize
$failed = @($results | Where-Object { -not $_.Dat })
if ($failed.Count -eq 0) { Write-Host "TẤT CẢ ĐẠT. Gửi bảng trên cho Claude." } else { Write-Host "CÓ $($failed.Count) BƯỚC LỖI. Gửi toàn bộ nội dung cửa sổ này cho Claude." }
Write-Host "Repo tạm: $repo   (xóa khi xong)"
