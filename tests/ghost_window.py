"""
Ghost window test harness for LavaWM's wm-cleanup-windows command.

BACKGROUND:
  A "ghost window" is one that LavaWM tracks internally but whose HWND has
  become invalid (IsWindow() returns FALSE).  In production this happens when
  the WM misses an EVENT_OBJECT_DESTROY notification — a rare edge case
  triggered by system load, event queue overflow, or Win32 subsystem races.

  Windows' Win32k kernel always sends EVENT_OBJECT_DESTROY to registered
  WinEvent hooks when a process dies, even via TerminateProcess.  That means
  ghost windows are HARD to reproduce artificially.

MODES:
  single (default)
      Create one window and wait for a manual kill.  LavaWM will almost
      certainly catch the destroy event, so this is mainly a smoke test.

  flood [COUNT]
      Spawn COUNT child processes (default 50), each with a visible window.
      After LavaWM manages them all, terminate every child simultaneously
      with TerminateProcess.  The burst of destroy events may overflow the
      WinEvent delivery queue, leaving some windows as ghosts.

USAGE:
  # Smoke test (WM will likely catch the destroy event):
  python tests/ghost_window.py
  taskkill /F /PID <pid>
  target\\debug\\lavawm.exe command wm-cleanup-windows

  # Flood test (best chance of creating ghosts):
  python tests/ghost_window.py flood
  target\\debug\\lavawm.exe command wm-cleanup-windows

  # Flood with custom count:
  python tests/ghost_window.py flood 100
  target\\debug\\lavawm.exe command wm-cleanup-windows
"""

import ctypes
import ctypes.wintypes
import multiprocessing
import os
import sys
import time

user32 = ctypes.windll.user32
kernel32 = ctypes.windll.kernel32

# Win32 constants
WNDPROC = ctypes.WINFUNCTYPE(
    ctypes.c_long,
    ctypes.wintypes.HWND,
    ctypes.c_uint,
    ctypes.wintypes.WPARAM,
    ctypes.wintypes.LPARAM,
)
WS_OVERLAPPEDWINDOW = 0x00CF0000
WS_VISIBLE = 0x10000000
CW_USEDEFAULT = 0x80000000
WM_DESTROY = 0x0002
CS_HREDRAW = 0x0002
CS_VREDRAW = 0x0001
IDI_APPLICATION = 32512
IDC_ARROW = 32512
COLOR_WINDOW = 5
PROCESS_TERMINATE = 0x0001
PM_REMOVE = 0x0001


class WNDCLASS(ctypes.Structure):
    _fields_ = [
        ("style", ctypes.c_uint),
        ("lpfnWndProc", WNDPROC),
        ("cbClsExtra", ctypes.c_int),
        ("cbWndExtra", ctypes.c_int),
        ("hInstance", ctypes.wintypes.HINSTANCE),
        ("hIcon", ctypes.wintypes.HICON),
        ("hCursor", ctypes.wintypes.HANDLE),
        ("hbrBackground", ctypes.wintypes.HBRUSH),
        ("lpszMenuName", ctypes.wintypes.LPCWSTR),
        ("lpszClassName", ctypes.wintypes.LPCWSTR),
    ]


def wnd_proc(hwnd, msg, wparam, lparam):
    if msg == WM_DESTROY:
        user32.PostQuitMessage(0)
        return 0
    return user32.DefWindowProcW(hwnd, msg, wparam, lparam)


def create_window(title="Ghost Test Window"):
    """Create a visible Win32 window.  Returns HWND or None."""
    hinstance = kernel32.GetModuleHandleW(None)
    class_name = f"GhostTest_{os.getpid()}"

    wnd_class = WNDCLASS()
    wnd_class.style = CS_HREDRAW | CS_VREDRAW
    wnd_class.lpfnWndProc = WNDPROC(wnd_proc)
    wnd_class.hInstance = hinstance
    wnd_class.hIcon = user32.LoadIconW(None, IDI_APPLICATION)
    wnd_class.hCursor = user32.LoadCursorW(None, IDC_ARROW)
    wnd_class.hbrBackground = COLOR_WINDOW + 1
    wnd_class.lpszClassName = class_name

    if not user32.RegisterClassW(ctypes.byref(wnd_class)):
        return None

    hwnd = user32.CreateWindowExW(
        0,
        class_name,
        title,
        WS_OVERLAPPEDWINDOW | WS_VISIBLE,
        CW_USEDEFAULT,
        CW_USEDEFAULT,
        400,
        300,
        None,
        None,
        hinstance,
        None,
    )
    return hwnd if hwnd else None


# ── child process entry point (flood mode) ──────────────────────────

def child_worker(ready_event, kill_event):
    """Create a window, signal ready, pump messages until told to stop."""
    hwnd = create_window(f"Ghost #{os.getpid()}")
    if not hwnd:
        return

    ready_event.set()

    msg = ctypes.wintypes.MSG()
    while not kill_event.is_set():
        # Non-blocking message pump so we can check the kill event.
        if user32.PeekMessageW(ctypes.byref(msg), None, 0, 0, PM_REMOVE):
            user32.TranslateMessage(ctypes.byref(msg))
            user32.DispatchMessageW(ctypes.byref(msg))
        else:
            time.sleep(0.01)

    # Spin forever — the parent will TerminateProcess us.
    while True:
        time.sleep(10)


# ── modes ────────────────────────────────────────────────────────────

def mode_single():
    """Create one window and wait for manual kill."""
    hwnd = create_window("Ghost Test - kill me with taskkill /F")
    if not hwnd:
        print("Failed to create window", file=sys.stderr)
        return 1

    pid = os.getpid()
    print(f"Window created.  HWND={hwnd}  PID={pid}")
    print(f"Force-kill:  taskkill /F /PID {pid}")
    print()
    print("NOTE: Windows typically still delivers EVENT_OBJECT_DESTROY even")
    print("after taskkill /F, so LavaWM will likely catch the destroy event.")
    print("Use 'flood' mode for a better chance at creating real ghosts.")
    print()
    print("Waiting for message loop (or force-kill)...")

    msg = ctypes.wintypes.MSG()
    while user32.GetMessageW(ctypes.byref(msg), None, 0, 0) > 0:
        user32.TranslateMessage(ctypes.byref(msg))
        user32.DispatchMessageW(ctypes.byref(msg))

    return 0


def mode_flood(count=50):
    """Spawn many child processes then terminate them all at once."""
    print(f"Spawning {count} child processes with windows...")

    children = []
    for i in range(count):
        ready = multiprocessing.Event()
        kill = multiprocessing.Event()
        p = multiprocessing.Process(target=child_worker, args=(ready, kill))
        p.start()
        children.append((p, ready, kill))

    # Wait for every child to create its window.
    print("Waiting for all windows to be created...")
    for p, ready, _kill in children:
        if not ready.wait(timeout=10):
            print(f"  Child PID={p.pid} timed out", file=sys.stderr)

    managed = sum(1 for _, r, _ in children if r.is_set())
    print(f"{managed}/{count} windows created and (presumably) managed.")

    print("Pausing 3 s so LavaWM can finish processing manage events...")
    time.sleep(3)

    # Tell children to stop pumping messages, then terminate them all
    # as close together as possible.
    print(f"Terminating all {managed} processes simultaneously...")
    for _, _, kill in children:
        kill.set()
    time.sleep(0.05)  # let children enter their spin loop

    # Collect handles first so we can call TerminateProcess in a tight loop.
    handles = []
    for p, ready, _ in children:
        if p.is_alive() and ready.is_set():
            h = kernel32.OpenProcess(PROCESS_TERMINATE, False, p.pid)
            if h:
                handles.append(h)

    # Fire all terminations as fast as possible.
    for h in handles:
        kernel32.TerminateProcess(h, 1)
    for h in handles:
        kernel32.CloseHandle(h)

    for p, _, _ in children:
        p.join(timeout=5)

    print()
    print("All child processes terminated.")
    print(f"If any of the {managed} windows became ghosts, run:")
    print("  target\\debug\\lavawm.exe command wm-cleanup-windows")
    print()
    print("Check LavaWM logs for 'Removing invalid window' messages.")
    return 0


# ── entry point ──────────────────────────────────────────────────────

def main():
    if len(sys.argv) < 2 or sys.argv[1] == "single":
        return mode_single()
    elif sys.argv[1] == "flood":
        n = int(sys.argv[2]) if len(sys.argv) > 2 else 50
        return mode_flood(n)
    else:
        print(f"Unknown mode: {sys.argv[1]}")
        print("Usage: python ghost_window.py [single | flood [COUNT]]")
        return 1


if __name__ == "__main__":
    multiprocessing.freeze_support()
    sys.exit(main())
