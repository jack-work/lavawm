param(
  [int]$Attempts = 4,
  [int]$TimeoutSeconds = 10,
  [string]$Log = (Join-Path $PSScriptRoot 'restart-zebar.log')
)

function Write-Log([string]$Message) {
  if ((Test-Path $Log) -and (Get-Item $Log).Length -gt 100KB) {
    Get-Content $Log -Tail 200 | Set-Content "$Log.tmp"
    Move-Item "$Log.tmp" $Log -Force
  }
  "$(Get-Date -Format o) $Message" | Add-Content $Log
}

$mutex = New-Object System.Threading.Mutex($false, 'Local\lavawm-restart-zebar')
if (-not $mutex.WaitOne(0)) {
  Write-Log 'skipped: a restart is already running'
  exit 0
}

function Get-ZebarWebViews {
  $zebarIds = @(Get-Process zebar -ErrorAction SilentlyContinue | ForEach-Object Id)
  if (-not $zebarIds) { return @() }
  @(Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe'" |
    Where-Object { $zebarIds -contains $_.ParentProcessId })
}

try {
  for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
    Stop-Process -Name zebar -Force -ErrorAction SilentlyContinue
    Wait-Process -Name zebar -Timeout 5 -ErrorAction SilentlyContinue
    Start-Process zebar

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
      Start-Sleep -Milliseconds 500
      $webViews = Get-ZebarWebViews
    } while (-not $webViews -and (Get-Date) -lt $deadline)

    if ($webViews) {
      Write-Log "attempt $attempt ok"
      exit 0
    }

    Write-Log "attempt $attempt stalled: no WebView2 process after ${TimeoutSeconds}s"
  }

  Write-Log "gave up after $Attempts attempts"
  exit 1
} finally {
  $mutex.ReleaseMutex()
}
