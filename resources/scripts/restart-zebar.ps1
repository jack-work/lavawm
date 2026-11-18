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

Add-Type -TypeDefinition @'
using System;
using System.Collections.Generic;
using System.Runtime.InteropServices;
using System.Text;
public static class ZebarWindows {
  delegate bool EnumProc(IntPtr hwnd, IntPtr lParam);
  [DllImport("user32.dll")] static extern bool EnumWindows(EnumProc callback, IntPtr lParam);
  [DllImport("user32.dll")] static extern uint GetWindowThreadProcessId(IntPtr hwnd, out uint pid);
  [DllImport("user32.dll", CharSet = CharSet.Unicode)] static extern int GetWindowText(IntPtr hwnd, StringBuilder text, int max);
  [DllImport("user32.dll")] static extern bool IsWindowVisible(IntPtr hwnd);
  [DllImport("user32.dll")] static extern bool GetWindowRect(IntPtr hwnd, out Rect rect);
  struct Rect { public int Left, Top, Right, Bottom; }
  public static List<int[]> WidgetSizes(uint pid) {
    var sizes = new List<int[]>();
    EnumWindows((hwnd, _) => {
      uint owner;
      GetWindowThreadProcessId(hwnd, out owner);
      var title = new StringBuilder(256);
      GetWindowText(hwnd, title, title.Capacity);
      Rect rect;
      if (owner == pid && IsWindowVisible(hwnd) && title.ToString().StartsWith("Zebar - ") && GetWindowRect(hwnd, out rect)) {
        sizes.Add(new[] { rect.Right - rect.Left, rect.Bottom - rect.Top });
      }
      return true;
    }, IntPtr.Zero);
    return sizes;
  }
}
'@

function Test-ZebarDrawn([System.Diagnostics.Process]$Zebar) {
  $webViews = Get-CimInstance Win32_Process -Filter "Name = 'msedgewebview2.exe' AND ParentProcessId = $($Zebar.Id)"
  $sizes = [ZebarWindows]::WidgetSizes($Zebar.Id)
  $unplaced = @($sizes | Where-Object { $_[0] * 3 -eq $_[1] * 4 })
  [bool]$webViews -and $sizes.Count -gt 0 -and $unplaced.Count -eq 0
}

try {
  for ($attempt = 1; $attempt -le $Attempts; $attempt++) {
    Stop-Process -Name zebar -Force -ErrorAction SilentlyContinue
    Wait-Process -Name zebar -Timeout 5 -ErrorAction SilentlyContinue
    $zebar = Start-Process zebar -PassThru

    $deadline = (Get-Date).AddSeconds($TimeoutSeconds)
    do {
      Start-Sleep -Milliseconds 500
      $drawn = Test-ZebarDrawn $zebar
    } while (-not $drawn -and -not $zebar.HasExited -and (Get-Date) -lt $deadline)

    if ($drawn) {
      Write-Log "attempt $attempt ok: zebar $($zebar.Id)"
      exit 0
    }

    if ($zebar.HasExited) {
      Write-Log "attempt $attempt failed: zebar $($zebar.Id) exited with $($zebar.ExitCode)"
    } else {
      Write-Log "attempt $attempt stalled: zebar $($zebar.Id) not drawn after ${TimeoutSeconds}s"
    }
  }

  Write-Log "gave up after $Attempts attempts"
  exit 1
} finally {
  $mutex.ReleaseMutex()
}
