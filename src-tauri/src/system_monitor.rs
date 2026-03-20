//! 系统状态感知：前台窗口/应用、时段、用户空闲时长。
//! Windows / macOS 有具体实现；其他平台返回 `"Unknown"` / `0`，不 panic。
//! CPU / 内存占用通过 `sysinfo` 获取。

use chrono::{Local, Timelike};
use sysinfo::{CpuRefreshKind, System};

// ═══════════════════════════════════════════════════════════════════════════
// 公共 API
// ═══════════════════════════════════════════════════════════════════════════

/// 当前前台窗口标题（尽可能获取；失败或无标题时返回合理占位）。
pub fn get_active_window() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        return windows_impl::active_window();
    }
    #[cfg(target_os = "macos")]
    {
        return macos_impl::active_window();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok("Unknown".to_string())
    }
}

/// 当前前台应用名称（如 Chrome、Code）。
pub fn get_active_app() -> Result<String, String> {
    #[cfg(target_os = "windows")]
    {
        return windows_impl::active_app();
    }
    #[cfg(target_os = "macos")]
    {
        return macos_impl::active_app();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok("Unknown".to_string())
    }
}

/// 本地时段：`morning` / `afternoon` / `evening` / `night`
pub fn get_system_time() -> Result<String, String> {
    let hour = Local::now().hour();
    let label = match hour {
        5..=11 => "morning",
        12..=16 => "afternoon",
        17..=20 => "evening",
        _ => "night", // 21–23, 0–4
    };
    Ok(label.to_string())
}

/// 用户空闲时长（秒）。无法获取时返回 `0` 并打日志，不当作硬错误。
pub fn get_idle_duration() -> Result<u64, String> {
    #[cfg(target_os = "windows")]
    {
        return windows_impl::idle_seconds();
    }
    #[cfg(target_os = "macos")]
    {
        return macos_impl::idle_seconds();
    }
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    {
        Ok(0)
    }
}

/// 全局 CPU 使用率（0.0–100.0，百分比）。
/// 需短暂休眠以满足 `sysinfo` 的最小采样间隔，请在 `spawn_blocking` 中调用。
pub fn get_cpu_usage() -> Result<f32, String> {
    let mut sys = System::new();
    sys.refresh_cpu_list(CpuRefreshKind::everything());
    std::thread::sleep(sysinfo::MINIMUM_CPU_UPDATE_INTERVAL);
    sys.refresh_cpu_usage();
    let pct = sys.global_cpu_usage();
    if !pct.is_finite() {
        return Err("CPU 读数无效".to_string());
    }
    Ok(pct.clamp(0.0, 100.0))
}

/// 已用物理内存（字节）。
pub fn get_memory_usage() -> Result<u64, String> {
    let mut sys = System::new();
    sys.refresh_memory();
    Ok(sys.used_memory())
}

// ═══════════════════════════════════════════════════════════════════════════
// Windows
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "windows")]
mod windows_impl {
    use std::path::Path;
    use windows::Win32::Foundation::CloseHandle;
    use windows::Win32::System::ProcessStatus::GetModuleFileNameExW;
    use windows::Win32::System::SystemInformation::GetTickCount64;
    use windows::Win32::System::Threading::{
        OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_VM_READ,
    };
    use windows::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows::Win32::UI::WindowsAndMessaging::{GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId};

    pub fn active_window() -> Result<String, String> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_invalid() {
            return Ok("Unknown".to_string());
        }
        let mut buf = [0u16; 512];
        let len = unsafe { GetWindowTextW(hwnd, &mut buf) } as usize;
        if len == 0 {
            return Ok("Unknown".to_string());
        }
        Ok(String::from_utf16_lossy(&buf[..len]).to_string())
    }

    pub fn active_app() -> Result<String, String> {
        let hwnd = unsafe { GetForegroundWindow() };
        if hwnd.is_invalid() {
            return Ok("Unknown".to_string());
        }

        let mut pid: u32 = 0;
        unsafe { GetWindowThreadProcessId(hwnd, Some(&mut pid)) };
        if pid == 0 {
            return Ok("Unknown".to_string());
        }

        let handle = unsafe {
            OpenProcess(
                PROCESS_QUERY_LIMITED_INFORMATION | PROCESS_VM_READ,
                false,
                pid,
            )
        };

        let handle = match handle {
            Ok(h) => h,
            Err(_) => {
                // 部分环境无 VM_READ，再试 LIMITED
                match unsafe { OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid) } {
                    Ok(h) => h,
                    Err(_) => return Ok("Unknown".to_string()),
                }
            }
        };

        let mut buf = [0u16; 520];
        let n = unsafe { GetModuleFileNameExW(handle, None, &mut buf) } as usize;

        let _ = unsafe { CloseHandle(handle) };

        if n == 0 {
            return Ok("Unknown".to_string());
        }

        let path = String::from_utf16_lossy(&buf[..n]);
        let name = Path::new(path.as_str())
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown")
            .to_string();

        if name.is_empty() {
            Ok("Unknown".to_string())
        } else {
            Ok(name)
        }
    }

    pub fn idle_seconds() -> Result<u64, String> {
        let mut lii = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        let ok = unsafe { GetLastInputInfo(&mut lii) };
        if !ok.as_bool() {
            tracing::warn!("GetLastInputInfo 失败");
            return Ok(0);
        }
        let now = unsafe { GetTickCount64() };
        let idle_ms = now.saturating_sub(lii.dwTime as u64);
        Ok(idle_ms / 1000)
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// macOS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(target_os = "macos")]
mod macos_impl {
    use std::process::Command;

    /// `kCGEventSourceStateHIDSystemState`
    const KCG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE: u32 = 1;
    /// `kCGAnyInputEventType` == (uint32_t)(-1)
    const KCG_ANY_INPUT_EVENT_TYPE: u32 = u32::MAX;

    #[link(name = "CoreGraphics", kind = "framework")]
    unsafe extern "C" {
        fn CGEventSourceSecondsSinceLastEventType(state: u32, event_type: u32) -> f64;
    }

    fn run_osascript(script: &str) -> Result<String, String> {
        let out = Command::new("osascript")
            .arg("-e")
            .arg(script)
            .output()
            .map_err(|e| format!("osascript 启动失败: {}", e))?;

        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            tracing::debug!("osascript 失败: {}", err);
            return Err(err.to_string());
        }

        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    }

    pub fn active_app() -> Result<String, String> {
        let script = r#"tell application "System Events" to return name of first application process whose frontmost is true"#;
        match run_osascript(script) {
            Ok(s) if !s.is_empty() => Ok(s),
            Ok(_) => Ok("Unknown".to_string()),
            Err(_) => Ok("Unknown".to_string()),
        }
    }

    pub fn active_window() -> Result<String, String> {
        let script = r#"tell application "System Events"
  tell (first process whose frontmost is true)
    try
      return name of first window
    on error
      return ""
    end try
  end tell
end tell"#;
        match run_osascript(script) {
            Ok(s) => Ok(if s.is_empty() { "Unknown".to_string() } else { s }),
            Err(_) => Ok("Unknown".to_string()),
        }
    }

    pub fn idle_seconds() -> Result<u64, String> {
        let secs = unsafe {
            CGEventSourceSecondsSinceLastEventType(
                KCG_EVENT_SOURCE_STATE_HID_SYSTEM_STATE,
                KCG_ANY_INPUT_EVENT_TYPE,
            )
        };
        if !secs.is_finite() || secs < 0.0 {
            tracing::warn!("CGEventSourceSecondsSinceLastEventType 异常: {}", secs);
            return Ok(0);
        }
        Ok(secs.floor() as u64)
    }
}
