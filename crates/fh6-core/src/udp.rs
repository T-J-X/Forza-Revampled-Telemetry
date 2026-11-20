use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;

use crate::telemetry::{decode, FH6TelemetryData};

const PRIMARY_PORT: u16 = 8000;
const FALLBACK_PORT: u16 = 5500;
const FORWARD_PORT: u16 = 5607;

struct EngineState {
    last_packet_time: Option<Instant>,
    last_driving_resume: Option<Instant>,
    was_driving_active: bool,
}

pub struct TelemetryEngine {
    latest: Arc<ArcSwap<FH6TelemetryData>>,
    state: Arc<Mutex<EngineState>>,
    running: Arc<AtomicBool>,
    handles: Mutex<Vec<thread::JoinHandle<()>>>,
}

impl TelemetryEngine {
    pub fn new() -> Self {
        Self {
            latest: Arc::new(ArcSwap::from_pointee(FH6TelemetryData::default())),
            state: Arc::new(Mutex::new(EngineState {
                last_packet_time: None,
                last_driving_resume: None,
                was_driving_active: false,
            })),
            running: Arc::new(AtomicBool::new(false)),
            handles: Mutex::new(Vec::new()),
        }
    }

    pub fn start(&self) -> Result<(), String> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let primary = Self::bind_socket(PRIMARY_PORT);
        let secondary = if FALLBACK_PORT != PRIMARY_PORT {
            Self::bind_socket(FALLBACK_PORT)
        } else {
            None
        };
        let forwarder = if FORWARD_PORT > 0 {
            UdpSocket::bind("0.0.0.0:0").ok().map(Arc::new)
        } else {
            None
        };

        let state = self.state.clone();
        let latest = self.latest.clone();
        let running = self.running.clone();
        let mut handles = self.handles.lock().unwrap();

        if let Some(sock) = primary {
            let sock_arc = Arc::new(sock);
            let fwd = forwarder.clone();
            let state_clone = state.clone();
            let latest_clone = latest.clone();
            let running_clone = running.clone();
            let h = thread::spawn(move || Self::receive_loop(sock_arc, state_clone, latest_clone, fwd, running_clone));
            handles.push(h);
        }
        if let Some(sock) = secondary {
            let sock_arc = Arc::new(sock);
            let fwd = forwarder.clone();
            let state_clone = state.clone();
            let latest_clone = latest.clone();
            let running_clone = running.clone();
            let h = thread::spawn(move || Self::receive_loop(sock_arc, state_clone, latest_clone, fwd, running_clone));
            handles.push(h);
        }

        Ok(())
    }

    fn bind_socket(port: u16) -> Option<UdpSocket> {
        let addr = format!("0.0.0.0:{}", port);
        let sock = UdpSocket::bind(&addr).ok()?;
        sock.set_nonblocking(true).ok()?;
        Some(sock)
    }

    fn receive_loop(
        socket: Arc<UdpSocket>,
        state: Arc<Mutex<EngineState>>,
        latest: Arc<ArcSwap<FH6TelemetryData>>,
        forwarder: Option<Arc<UdpSocket>>,
        running: Arc<AtomicBool>,
    ) {
        let mut buf = [0u8; 2048];
        while running.load(Ordering::SeqCst) {
            match socket.recv_from(&mut buf) {
                Ok((len, _src)) => {
                    if len == 324 {
                        Self::process_packet(&buf[..len], &latest, &state, &forwarder);
                    }
                }
                Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(10));
                }
                Err(_) => {
                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    }

    fn process_packet(
        bytes: &[u8],
        latest: &Arc<ArcSwap<FH6TelemetryData>>,
        state: &Arc<Mutex<EngineState>>,
        forwarder: &Option<Arc<UdpSocket>>,
    ) {
        if let Ok(data) = decode(bytes) {
            let now = Instant::now();
            let mut st = state.lock().unwrap();
            let prev_active = st
                .last_packet_time
                .map(|t| now.duration_since(t) < Duration::from_millis(500))
                .unwrap_or(false);
            st.last_packet_time = Some(now);
            if data.is_race_on == 1 && (!prev_active || !st.was_driving_active) {
                st.was_driving_active = true;
                st.last_driving_resume = Some(now);
            } else if data.is_race_on == 0 {
                st.was_driving_active = false;
            }
            latest.store(Arc::new(data));
            if let Some(fwd) = forwarder {
                let _ = fwd.send_to(bytes, format!("127.0.0.1:{}", FORWARD_PORT));
            }
        }
    }

    pub fn get_latest(&self) -> Arc<FH6TelemetryData> {
        Arc::clone(&self.latest.load())
    }

    pub fn is_driving_active(&self) -> bool {
        let st = self.state.lock().unwrap();
        let latest = self.latest.load();
        if latest.is_race_on != 1 {
            return false;
        }
        if let Some(t) = st.last_packet_time {
            Instant::now().duration_since(t) < Duration::from_millis(250)
        } else {
            false
        }
    }

    pub fn time_since_driving_resumed_ms(&self) -> f64 {
        if !self.is_driving_active() {
            return 0.0;
        }
        let st = self.state.lock().unwrap();
        if let Some(t) = st.last_driving_resume {
            Instant::now().duration_since(t).as_secs_f64() * 1000.0
        } else {
            0.0
        }
    }

    pub fn shutdown(&self) {
        self.running.store(false, Ordering::SeqCst);
        let mut handles = self.handles.lock().unwrap();
        for h in handles.drain(..) {
            let _ = h.join();
        }
    }
}

impl Default for TelemetryEngine {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn engine_initial_state() {
        let eng = TelemetryEngine::new();
        assert!(!eng.is_driving_active());
        assert_eq!(eng.time_since_driving_resumed_ms(), 0.0);
    }
}
