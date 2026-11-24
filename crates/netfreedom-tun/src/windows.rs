use anyhow::{anyhow, Result};
use std::ffi::{c_void, OsStr};
use std::os::windows::ffi::OsStrExt;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

pub type WintunAdapterHandle = *mut c_void;
pub type WintunSessionHandle = *mut c_void;

pub const WINTUN_DEFAULT_RING_CAPACITY: u32 = 0x400000; // 4 MiB

type WintunCreateAdapterFn = unsafe extern "system" fn(
    name: *const u16,
    tunnel_type: *const u16,
    requested_guid: *const c_void,
) -> WintunAdapterHandle;

type WintunOpenAdapterFn = unsafe extern "system" fn(
    name: *const u16,
) -> WintunAdapterHandle;

type WintunCloseAdapterFn = unsafe extern "system" fn(
    adapter: WintunAdapterHandle,
);

type WintunStartSessionFn = unsafe extern "system" fn(
    adapter: WintunAdapterHandle,
    capacity: u32,
) -> WintunSessionHandle;

type WintunEndSessionFn = unsafe extern "system" fn(
    session: WintunSessionHandle,
);

type WintunReceivePacketFn = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet_size: *mut u32,
) -> *mut u8;

type WintunReleaseReceivePacketFn = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet: *const u8,
);

type WintunAllocateSendPacketFn = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet_size: u32,
) -> *mut u8;

type WintunSendPacketFn = unsafe extern "system" fn(
    session: WintunSessionHandle,
    packet: *const u8,
);

extern "system" {
    fn LoadLibraryW(lpLibFileName: *const u16) -> *mut c_void;
    fn GetProcAddress(hModule: *mut c_void, lpProcName: *const i8) -> *mut c_void;
}

fn to_u16_vec(s: &str) -> Vec<u16> {
    OsStr::new(s).encode_wide().chain(std::iter::once(0)).collect()
}

pub struct WintunApi {
    _h_module: *mut c_void,
    create_adapter: WintunCreateAdapterFn,
    open_adapter: WintunOpenAdapterFn,
    close_adapter: WintunCloseAdapterFn,
    start_session: WintunStartSessionFn,
    end_session: WintunEndSessionFn,
    receive_packet: WintunReceivePacketFn,
    release_receive_packet: WintunReleaseReceivePacketFn,
    allocate_send_packet: WintunAllocateSendPacketFn,
    send_packet: WintunSendPacketFn,
}

unsafe impl Send for WintunApi {}
unsafe impl Sync for WintunApi {}

impl WintunApi {
    pub fn load() -> Result<Self> {
        let candidates = [
            "wintun.dll",
            "target/release/wintun.dll",
            "C:\\Users\\adi\\AppData\\Local\\Programs\\NetFreedom\\wintun.dll",
        ];

        let mut h_module: *mut c_void = std::ptr::null_mut();
        for candidate in candidates {
            let wide = to_u16_vec(candidate);
            unsafe {
                h_module = LoadLibraryW(wide.as_ptr());
                if !h_module.is_null() {
                    info!("Successfully loaded wintun.dll from '{}'", candidate);
                    break;
                }
            }
        }

        if h_module.is_null() {
            return Err(anyhow!("Could not locate or load wintun.dll in application directory or PATH"));
        }

        unsafe {
            macro_rules! get_fn {
                ($name:expr, $type:ty) => {{
                    let proc = GetProcAddress(h_module, concat!($name, "\0").as_ptr() as *const i8);
                    if proc.is_null() {
                        return Err(anyhow!("Missing Wintun entry point: {}", $name));
                    }
                    std::mem::transmute::<*mut c_void, $type>(proc)
                }};
            }

            Ok(Self {
                _h_module: h_module,
                create_adapter: get_fn!("WintunCreateAdapter", WintunCreateAdapterFn),
                open_adapter: get_fn!("WintunOpenAdapter", WintunOpenAdapterFn),
                close_adapter: get_fn!("WintunCloseAdapter", WintunCloseAdapterFn),
                start_session: get_fn!("WintunStartSession", WintunStartSessionFn),
                end_session: get_fn!("WintunEndSession", WintunEndSessionFn),
                receive_packet: get_fn!("WintunReceivePacket", WintunReceivePacketFn),
                release_receive_packet: get_fn!("WintunReleaseReceivePacket", WintunReleaseReceivePacketFn),
                allocate_send_packet: get_fn!("WintunAllocateSendPacket", WintunAllocateSendPacketFn),
                send_packet: get_fn!("WintunSendPacket", WintunSendPacketFn),
            })
        }
    }
}

pub struct WintunAdapter {
    api: Arc<WintunApi>,
    adapter: WintunAdapterHandle,
    session: Option<WintunSessionHandle>,
    is_running: Arc<AtomicBool>,
}

unsafe impl Send for WintunAdapter {}
unsafe impl Sync for WintunAdapter {}

impl WintunAdapter {
    pub fn create_or_open(name: &str, tunnel_type: &str) -> Result<Self> {
        let api = Arc::new(WintunApi::load()?);
        let name_wide = to_u16_vec(name);
        let type_wide = to_u16_vec(tunnel_type);

        info!("Creating or opening Wintun adapter '{}' ({})", name, tunnel_type);

        let mut adapter = unsafe {
            (api.open_adapter)(name_wide.as_ptr())
        };

        if adapter.is_null() {
            adapter = unsafe {
                (api.create_adapter)(name_wide.as_ptr(), type_wide.as_ptr(), std::ptr::null())
            };
        }

        if adapter.is_null() {
            return Err(anyhow!("Failed to create or open Wintun adapter. Check administrator privileges."));
        }

        let session = unsafe {
            (api.start_session)(adapter, WINTUN_DEFAULT_RING_CAPACITY)
        };

        if session.is_null() {
            unsafe { (api.close_adapter)(adapter); }
            return Err(anyhow!("Failed to start Wintun session"));
        }

        info!("Wintun adapter session established with {} byte ring buffer", WINTUN_DEFAULT_RING_CAPACITY);

        Ok(Self {
            api,
            adapter,
            session: Some(session),
            is_running: Arc::new(AtomicBool::new(true)),
        })
    }

    /// Launches async packet bridge channels (tun_tx to send to TUN, tun_rx to receive from TUN)
    pub fn start_io_bridge(self) -> (mpsc::Sender<Vec<u8>>, mpsc::Receiver<Vec<u8>>, Arc<AtomicBool>) {
        let (out_tx, mut out_rx) = mpsc::channel::<Vec<u8>>(1024); // to write to TUN
        let (in_tx, in_rx) = mpsc::channel::<Vec<u8>>(1024);      // read from TUN

        let is_running = self.is_running.clone();
        let api_read = self.api.clone();
        let session_read = self.session.unwrap();
        let is_running_read = is_running.clone();

        // 1. Reader thread (blocking read from Wintun ring -> send to in_tx)
        std::thread::spawn(move || {
            while is_running_read.load(Ordering::Relaxed) {
                let mut packet_size: u32 = 0;
                let packet_ptr = unsafe {
                    (api_read.receive_packet)(session_read, &mut packet_size)
                };

                if !packet_ptr.is_null() && packet_size > 0 {
                    let slice = unsafe { std::slice::from_raw_parts(packet_ptr, packet_size as usize) };
                    let vec = slice.to_vec();
                    unsafe { (api_read.release_receive_packet)(session_read, packet_ptr); }
                    let _ = in_tx.blocking_send(vec);
                } else {
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            }
            debug!("Wintun reader thread exited");
        });

        // 2. Writer thread (receive from out_rx -> allocate and send to Wintun ring)
        let api_write = self.api.clone();
        let session_write = self.session.unwrap();
        let is_running_write = is_running.clone();

        std::thread::spawn(move || {
            while is_running_write.load(Ordering::Relaxed) {
                if let Some(packet) = out_rx.blocking_recv() {
                    let size = packet.len() as u32;
                    let send_ptr = unsafe {
                        (api_write.allocate_send_packet)(session_write, size)
                    };
                    if !send_ptr.is_null() {
                        unsafe {
                            std::ptr::copy_nonoverlapping(packet.as_ptr(), send_ptr, size as usize);
                            (api_write.send_packet)(session_write, send_ptr);
                        }
                    }
                }
            }
            debug!("Wintun writer thread exited");
        });

        (out_tx, in_rx, is_running)
    }

    pub fn close(&mut self) {
        self.is_running.store(false, Ordering::Relaxed);
        if let Some(session) = self.session.take() {
            unsafe { (self.api.end_session)(session); }
        }
        if !self.adapter.is_null() {
            unsafe { (self.api.close_adapter)(self.adapter); }
            self.adapter = std::ptr::null_mut();
        }
    }
}

impl Drop for WintunAdapter {
    fn drop(&mut self) {
        self.close();
    }
}
