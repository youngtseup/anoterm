# 生成更新清单 latest.json（本地应急发布用，仅 Windows）
# 用法: .\scripts\make-latest-json.ps1 -Version 0.2.0
# 说明: createUpdaterArtifacts = true 模式下，NSIS 安装器 .exe 即更新包
param(
  [Parameter(Mandatory = $true)]
  [string]$Version,
  [string]$Repo = "youngtseup/anoterm",
  [string]$BundleDir = "src-tauri/target/release/bundle/nsis",
  [string]$Notes = ""
)

$ErrorActionPreference = "Stop"

$exe = Get-ChildItem -LiteralPath $BundleDir -Filter "Anoterm_${Version}_x64-setup.exe" | Select-Object -First 1

if (-not $exe) { throw "未找到安装包: Anoterm_${Version}_x64-setup.exe（请先执行 pnpm tauri build）" }

$exeSig = "$($exe.FullName).sig"
if (-not (Test-Path -LiteralPath $exeSig)) { throw "缺少签名文件: $exeSig" }

if ([string]::IsNullOrWhiteSpace($Notes)) { $Notes = "Anoterm $Version 更新" }

$entry = @{
  signature = (Get-Content -LiteralPath $exeSig -Raw).Trim()
  url       = "https://github.com/$Repo/releases/download/v$Version/$($exe.Name)"
}

$json = @{
  version   = $Version
  notes     = $Notes
  pub_date  = (Get-Date).ToUniversalTime().ToString("yyyy-MM-ddTHH:mm:ss.fffZ")
  platforms = @{
    "windows-x86_64"      = $entry
    "windows-x86_64-nsis" = $entry
  }
} | ConvertTo-Json -Depth 5

$out = Join-Path $BundleDir "latest.json"
$utf8NoBom = New-Object System.Text.UTF8Encoding($false)
[System.IO.File]::WriteAllText($out, $json, $utf8NoBom)
Write-Host "已生成: $out"
Write-Host "上传到 Release v$Version 的资产: $($exe.Name), $($exe.Name).sig, latest.json"
Write-Host "注意: macOS/Linux 平台需通过 CI 发布（.github/workflows/release.yml）"
