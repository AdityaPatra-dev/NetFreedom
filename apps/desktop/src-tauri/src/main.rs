// Prevents additional console window on Windows in release
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use netfreedom_common::{ConnectionStatus, HealthMetrics};
use std::process::Command;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

#[cfg(target_os = "windows")]
fn win_cmd(program: impl AsRef<std::ffi::OsStr>) -> Command {
    let mut cmd = Command::new(program);
    cmd.creation_flags(CREATE_NO_WINDOW);
    cmd
}

#[cfg(target_os = "windows")]
fn notify_wininet_proxy_change() {
    unsafe {
        type InternetSetOptionWFn = unsafe extern "system" fn(*mut std::ffi::c_void, u32, *mut std::ffi::c_void, u32) -> i32;
        extern "system" {
            fn LoadLibraryA(lpLibFileName: *const i8) -> *mut std::ffi::c_void;
            fn GetProcAddress(hModule: *mut std::ffi::c_void, lpProcName: *const i8) -> *mut std::ffi::c_void;
        }

        let h_wininet = LoadLibraryA(b"wininet.dll\0".as_ptr() as *const i8);
        if !h_wininet.is_null() {
            let proc = GetProcAddress(h_wininet, b"InternetSetOptionW\0".as_ptr() as *const i8);
            if !proc.is_null() {
                let func: InternetSetOptionWFn = std::mem::transmute(proc);
                // INTERNET_OPTION_SETTINGS_CHANGED = 39
                func(std::ptr::null_mut(), 39, std::ptr::null_mut(), 0);
                // INTERNET_OPTION_REFRESH = 37
                func(std::ptr::null_mut(), 37, std::ptr::null_mut(), 0);
            }
        }
    }
}

#[tauri::command]
fn get_connection_status() -> ConnectionStatus {
    #[cfg(target_os = "linux")]
    {
        let mode_out = Command::new("gsettings")
            .args(&["get", "org.gnome.system.proxy", "mode"])
            .output();
        let is_manual = if let Ok(out) = mode_out {
            String::from_utf8_lossy(&out.stdout).contains("manual")
        } else {
            false
        };

        let pgrep_out = Command::new("pgrep").arg("-f").arg("ciadpi").status();
        let is_running = pgrep_out.map(|s| s.success()).unwrap_or(false);

        if is_running && is_manual {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }

    #[cfg(target_os = "windows")]
    {
        let tasklist_out = win_cmd("tasklist").output();
        let is_running = if let Ok(out) = tasklist_out {
            let s = String::from_utf8_lossy(&out.stdout).to_lowercase();
            s.contains("ciadpi") || s.contains("goodbyedpi")
        } else {
            false
        };

        if is_running {
            ConnectionStatus::Connected
        } else {
            ConnectionStatus::Disconnected
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "windows")))]
    {
        ConnectionStatus::Disconnected
    }
}

#[tauri::command]
fn get_health_metrics() -> HealthMetrics {
    HealthMetrics {
        tcp_latency_ms: Some(42.5),
        jitter_ms: Some(1.1),
        download_bytes_sec: 18_500_000,
        upload_bytes_sec: 4_200_000,
        total_rx_bytes: 512_000_000,
        total_tx_bytes: 128_000_000,
        public_ip: Some("123.63.59.157".to_string()),
        country: Some("India".to_string()),
        colo: Some("MAA".to_string()),
        captive_portal_detected: false,
        ipv6_working: false,
    }
}

#[tauri::command]
async fn connect_profile(mode: String) -> Result<String, String> {
    // 1. Kill any existing instance
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();

        // Start ciadpi in daemon mode with DPI desynchronization
        let ciadpi_status = Command::new("./third_party/byedpi/ciadpi")
            .args(&[
                "-i", "127.0.0.1",
                "-p", "1080",
                "-I", "0.0.0.0",
                "-s", "2+s",
                "-f", "-1",
                "-t", "3",
                "-D",
            ])
            .status();

        if let Err(e) = ciadpi_status {
            return Err(format!("Failed to start ciadpi: {}", e));
        }

        // Set GNOME system proxy
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.socks", "host", "127.0.0.1"])
            .status();
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy.socks", "port", "1080"])
            .status();
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "manual"])
            .status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = win_cmd("taskkill")
            .args(&["/F", "/IM", "ciadpi.exe", "/IM", "goodbyedpi.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        let exe_dir = std::env::current_exe().ok().and_then(|p| p.parent().map(|p| p.to_path_buf()));

        // 1. Check for and spawn GoodbyeDPI (WinDivert kernel engine)
        let mut gbdpi_candidates = Vec::new();
        if let Some(ref d) = exe_dir {
            gbdpi_candidates.push(d.join("goodbyedpi.exe"));
            gbdpi_candidates.push(d.join("third_party").join("goodbyedpi").join("goodbyedpi.exe"));
        }
        if let Ok(cur) = std::env::current_dir() {
            gbdpi_candidates.push(cur.join("goodbyedpi.exe"));
            gbdpi_candidates.push(cur.join("third_party").join("goodbyedpi").join("goodbyedpi.exe"));
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            gbdpi_candidates.push(std::path::PathBuf::from(local_app_data).join("Programs").join("NetFreedom").join("goodbyedpi.exe"));
        }
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            gbdpi_candidates.push(std::path::PathBuf::from(prog_files).join("NetFreedom").join("goodbyedpi.exe"));
        }
        gbdpi_candidates.push(std::path::PathBuf::from("goodbyedpi.exe"));

        if let Some(gbdpi_bin) = gbdpi_candidates.into_iter().find(|p| p.exists()) {
            let _ = win_cmd(&gbdpi_bin)
                .args(&["-9"])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }

        // 2. Check for and spawn ciadpi (ByeDPI SOCKS proxy with Fake SNI)
        let mut candidates = Vec::new();
        if let Some(ref d) = exe_dir {
            candidates.push(d.join("ciadpi.exe"));
        }
        if let Ok(cur) = std::env::current_dir() {
            candidates.push(cur.join("ciadpi.exe"));
        }
        if let Ok(local_app_data) = std::env::var("LOCALAPPDATA") {
            candidates.push(std::path::PathBuf::from(local_app_data).join("Programs").join("NetFreedom").join("ciadpi.exe"));
        }
        if let Ok(prog_files) = std::env::var("ProgramFiles") {
            candidates.push(std::path::PathBuf::from(prog_files).join("NetFreedom").join("ciadpi.exe"));
        }
        candidates.push(std::path::PathBuf::from("ciadpi.exe"));

        if let Some(ciadpi_bin) = candidates.into_iter().find(|p| p.exists()) {
            let _ = win_cmd(&ciadpi_bin)
                .args(&[
                    "-i", "127.0.0.1",
                    "-p", "1080",
                    "-I", "0.0.0.0",
                    "-s", "2+s",
                    "-f", "-1",
                    "-n", "www.google.com",
                ])
                .stdin(std::process::Stdio::null())
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .spawn();
        }

        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;

        // 3. Enable Windows internet proxy via Registry
        let _ = win_cmd("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "1",
                "/f",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = win_cmd("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyServer",
                "/t",
                "REG_SZ",
                "/d",
                "socks=127.0.0.1:1080",
                "/f",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        // 4. Force Windows WinINet cache to refresh immediately (Chrome / Edge / Firefox)
        notify_wininet_proxy_change();
    }

    Ok(format!("Connected via mode: {}", mode))
}

#[tauri::command]
async fn disconnect_profile() -> Result<String, String> {
    #[cfg(target_os = "linux")]
    {
        let _ = Command::new("gsettings")
            .args(&["set", "org.gnome.system.proxy", "mode", "none"])
            .status();
        let _ = Command::new("pkill").args(&["-9", "-f", "ciadpi"]).status();
    }

    #[cfg(target_os = "windows")]
    {
        let _ = win_cmd("reg")
            .args(&[
                "add",
                "HKCU\\Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
                "/v",
                "ProxyEnable",
                "/t",
                "REG_DWORD",
                "/d",
                "0",
                "/f",
            ])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();
        let _ = win_cmd("taskkill")
            .args(&["/F", "/IM", "ciadpi.exe", "/IM", "goodbyedpi.exe"])
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status();

        // Force Windows WinINet cache to refresh immediately
        notify_wininet_proxy_change();
    }

    Ok("Disconnected".to_string())
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            get_connection_status,
            get_health_metrics,
            connect_profile,
            disconnect_profile
        ])
        .run(tauri::generate_context!())
        .expect("error while running NetFreedom desktop application");
}
