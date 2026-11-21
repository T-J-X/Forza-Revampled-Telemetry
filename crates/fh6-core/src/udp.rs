//! UDP telemetry engine: receives Forza "Data Out" packets and publishes the latest one.
//!
//! There is no queue. Each receive thread blocks in `recv_from`, decodes the 324-byte packet,
//! stamps it with the receive [`Instant`], swaps it into an `ArcSwap` and calls the notifier
//! (the render thread's `unpark`). Readers always see the newest packet; older ones are dropped.
use std::io;
use std::net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};

use arc_swap::ArcSwap;

use crate::config::TelemetryPorts;
use crate::telemetry::{decode, FH6TelemetryData, PACKET_LEN};

/// How long a stream may be silent before it counts as stale (menus, pause, alt-tab).
pub const STALE_AFTER: Duration = Duration::from_millis(250);

/// How often a blocked receive wakes to check for shutdown.
const RECV_TIMEOUT: Duration = Duration::from_millis(100);

/// A packet that arrives on the second port within this window with the same game timestamp
/// as the latest one is a duplicate (senders such as `mock-udp` send to both ports).
const DUPLICATE_WINDOW: Duration = Duration::from_millis(50);

/// Called after every published packet.
pub type PacketNotify = Arc<dyn Fn() + Send + Sync>;

/// One decoded packet plus when it arrived.
#[derive(Debug, Clone, PartialEq)]
pub struct TelemetryFrame {
    /// The decoded packet.
    pub data: FH6TelemetryData,
    /// When `recv_from` returned this packet. `None` for the empty frame before any packet.
    pub received_at: Option<Instant>,
    /// 1 for the first packet, then +1 per published packet; 0 means "no packet yet".
    pub seq: u64,
    /// When the current `IsRaceOn == 1` stretch began, if racing.
    pub race_resumed_at: Option<Instant>,
}

impl TelemetryFrame {
    /// The frame readers see before any packet has arrived.
    pub fn empty() -> Self {
        Self {
            data: FH6TelemetryData::default(),
            received_at: None,
            seq: 0,
            race_resumed_at: None,
        }
    }

    /// Time since this packet arrived, or `None` if there has been no packet.
    pub fn age(&self, now: Instant) -> Option<Duration> {
        self.received_at.map(|t| now.saturating_duration_since(t))
    }

    /// True when no packet has arrived for [`STALE_AFTER`] (or ever).
    pub fn is_stale(&self, now: Instant) -> bool {
        self.age(now).is_none_or(|age| age > STALE_AFTER)
    }

    /// Racing (`IsRaceOn == 1`) with a fresh stream.
    pub fn is_driving_active(&self, now: Instant) -> bool {
        self.data.is_race_on == 1 && !self.is_stale(now)
    }

    /// Milliseconds since racing resumed, or 0 when not driving.
    pub fn time_since_driving_resumed_ms(&self, now: Instant) -> f64 {
        match (self.is_driving_active(now), self.race_resumed_at) {
            (true, Some(t)) => now.saturating_duration_since(t).as_secs_f64() * 1000.0,
            _ => 0.0,
        }
    }
}

/// Why the engine could not start.
#[derive(Debug, thiserror::Error)]
pub enum UdpError {
    /// No listening port could be bound.
    #[error("could not bind telemetry port {port}")]
    Bind {
        /// The primary port that failed.
        port: u16,
        /// The underlying error.
        #[source]
        source: io::Error,
    },
    /// A receive thread could not be spawned.
    #[error("spawning telemetry thread")]
    Spawn(#[source] io::Error),
}

/// Cheap, cloneable, lock-free read handle to the latest packet.
#[derive(Clone)]
pub struct TelemetryReader {
    latest: Arc<ArcSwap<TelemetryFrame>>,
}

impl std::fmt::Debug for TelemetryReader {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelemetryReader")
            .field("seq", &self.latest.load().seq)
            .finish()
    }
}

impl Default for TelemetryReader {
    fn default() -> Self {
        Self::new()
    }
}

impl TelemetryReader {
    /// A reader holding the empty frame. Hand it to [`TelemetryEngine::with_reader`] so a
    /// restarted engine (new ports) keeps publishing where existing readers look.
    pub fn new() -> Self {
        Self {
            latest: Arc::new(ArcSwap::from_pointee(TelemetryFrame::empty())),
        }
    }

    /// The newest frame. One atomic increment; no lock, no allocation.
    pub fn latest(&self) -> Arc<TelemetryFrame> {
        self.latest.load_full()
    }

    /// Publishes a frame directly. Used by tests and by synthetic sources such as the dyno.
    pub fn publish(&self, frame: TelemetryFrame) {
        self.latest.store(Arc::new(frame));
    }
}

/// Shared state of the receive threads. Only the UDP threads lock it; readers never do.
#[derive(Debug, Default)]
struct ReceiveState {
    /// The last published packet, to recognise the same datagram arriving on the second port.
    last_data: Option<FH6TelemetryData>,
    last_received: Option<Instant>,
    seq: u64,
    race_resumed_at: Option<Instant>,
    was_racing: bool,
}

impl ReceiveState {
    /// Builds the next frame, or `None` if `data` duplicates the previous packet.
    fn next_frame(&mut self, data: FH6TelemetryData, now: Instant) -> Option<TelemetryFrame> {
        // Same payload (not just the same game timestamp, so a pause flip is never dropped).
        let duplicate = self.last_data.as_ref() == Some(&data)
            && self
                .last_received
                .is_some_and(|t| now.saturating_duration_since(t) < DUPLICATE_WINDOW);
        if duplicate {
            return None;
        }
        let stream_was_live = self
            .last_received
            .is_some_and(|t| now.saturating_duration_since(t) <= STALE_AFTER);
        let racing = data.is_race_on == 1;
        if racing && !(self.was_racing && stream_was_live) {
            self.race_resumed_at = Some(now);
        } else if !racing {
            self.race_resumed_at = None;
        }
        self.was_racing = racing;
        self.last_data = Some(data.clone());
        self.last_received = Some(now);
        self.seq += 1;
        Some(TelemetryFrame {
            data,
            received_at: Some(now),
            seq: self.seq,
            race_resumed_at: self.race_resumed_at,
        })
    }
}

/// Owns the receive threads. Dropping it stops them.
pub struct TelemetryEngine {
    ports: TelemetryPorts,
    reader: TelemetryReader,
    notify: Option<PacketNotify>,
    running: Arc<AtomicBool>,
    threads: Vec<thread::JoinHandle<()>>,
    bound: Vec<u16>,
}

impl std::fmt::Debug for TelemetryEngine {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TelemetryEngine")
            .field("ports", &self.ports)
            .field("bound", &self.bound)
            .finish_non_exhaustive()
    }
}

impl TelemetryEngine {
    /// An engine for the given ports. Nothing is bound until [`TelemetryEngine::start`].
    pub fn new(ports: TelemetryPorts) -> Self {
        Self::with_reader(ports, TelemetryReader::new())
    }

    /// An engine that publishes into an existing reader.
    pub fn with_reader(ports: TelemetryPorts, reader: TelemetryReader) -> Self {
        Self {
            ports,
            reader,
            notify: None,
            running: Arc::new(AtomicBool::new(false)),
            threads: Vec::new(),
            bound: Vec::new(),
        }
    }

    /// Sets the callback run after each published packet (typically `Thread::unpark`).
    #[must_use]
    pub fn with_notify(mut self, notify: PacketNotify) -> Self {
        self.notify = Some(notify);
        self
    }

    /// Binds the primary (required) and fallback (optional) ports and starts receiving.
    ///
    /// # Errors
    /// [`UdpError::Bind`] if the primary port cannot be bound, [`UdpError::Spawn`] if a thread
    /// cannot be started. Calling `start` on a running engine is a no-op.
    pub fn start(&mut self) -> Result<(), UdpError> {
        if self.running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }
        let primary = bind(self.ports.primary, self.ports.all_interfaces).map_err(|source| {
            self.running.store(false, Ordering::SeqCst);
            UdpError::Bind {
                port: self.ports.primary,
                source,
            }
        })?;
        let mut sockets = vec![(self.ports.primary, primary)];
        if let Some(port) = self.ports.fallback {
            match bind(port, self.ports.all_interfaces) {
                Ok(sock) => sockets.push((port, sock)),
                Err(err) => tracing::warn!(port, error = %err, "fallback telemetry port not bound"),
            }
        }
        let forward = match self.ports.forward {
            Some(port) => UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))
                .map(|sock| Some((sock, SocketAddr::from((Ipv4Addr::LOCALHOST, port)))))
                .unwrap_or_else(|err| {
                    tracing::warn!(port, error = %err, "telemetry forwarding disabled");
                    None
                }),
            None => None,
        };
        let forward = Arc::new(forward);
        let state = Arc::new(Mutex::new(ReceiveState::default()));

        for (port, socket) in sockets {
            let ctx = ReceiveContext {
                socket,
                state: Arc::clone(&state),
                latest: Arc::clone(&self.reader.latest),
                forward: Arc::clone(&forward),
                notify: self.notify.clone(),
                running: Arc::clone(&self.running),
            };
            let handle = thread::Builder::new()
                .name(format!("fh6-udp-{port}"))
                .spawn(move || ctx.run())
                .map_err(UdpError::Spawn)?;
            self.threads.push(handle);
            self.bound.push(port);
        }
        tracing::info!(ports = ?self.bound, forward = ?self.ports.forward, "telemetry engine started");
        Ok(())
    }

    /// A lock-free read handle for other threads.
    pub fn reader(&self) -> TelemetryReader {
        self.reader.clone()
    }

    /// The newest frame.
    pub fn latest(&self) -> Arc<TelemetryFrame> {
        self.reader.latest()
    }

    /// The ports that were actually bound.
    pub fn bound_ports(&self) -> &[u16] {
        &self.bound
    }

    /// The ports this engine was configured with.
    pub fn ports(&self) -> TelemetryPorts {
        self.ports
    }

    /// Stops and joins the receive threads (each wakes within [`RECV_TIMEOUT`]).
    pub fn shutdown(&mut self) {
        self.running.store(false, Ordering::SeqCst);
        for handle in self.threads.drain(..) {
            let _ = handle.join();
        }
    }
}

impl Drop for TelemetryEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}

fn bind(port: u16, all_interfaces: bool) -> io::Result<UdpSocket> {
    // Loopback by default: Forza's Data Out targets 127.0.0.1 and loopback needs no firewall rule.
    let ip = if all_interfaces {
        Ipv4Addr::UNSPECIFIED
    } else {
        Ipv4Addr::LOCALHOST
    };
    let sock = UdpSocket::bind(SocketAddrV4::new(ip, port))?;
    sock.set_read_timeout(Some(RECV_TIMEOUT))?;
    Ok(sock)
}

struct ReceiveContext {
    socket: UdpSocket,
    state: Arc<Mutex<ReceiveState>>,
    latest: Arc<ArcSwap<TelemetryFrame>>,
    forward: Arc<Option<(UdpSocket, SocketAddr)>>,
    notify: Option<PacketNotify>,
    running: Arc<AtomicBool>,
}

impl ReceiveContext {
    fn run(self) {
        // Larger than a packet so oversized datagrams are seen (and rejected) whole.
        let mut buf = [0u8; 2048];
        while self.running.load(Ordering::Relaxed) {
            match self.socket.recv_from(&mut buf) {
                Ok((len, _src)) if len == PACKET_LEN => {
                    let now = Instant::now();
                    self.handle_packet(&buf[..len], now);
                }
                Ok(_) => {}
                Err(err) if matches!(err.kind(), io::ErrorKind::WouldBlock | io::ErrorKind::TimedOut) => {}
                // ConnectionReset is reported on Windows after an ICMP port-unreachable from a
                // previous forward; the socket stays usable.
                Err(err) if err.kind() == io::ErrorKind::ConnectionReset => {}
                Err(err) => {
                    tracing::warn!(error = %err, "telemetry receive failed");
                    thread::sleep(RECV_TIMEOUT);
                }
            }
        }
    }

    fn handle_packet(&self, bytes: &[u8], now: Instant) {
        let Ok(data) = decode(bytes) else { return };
        let frame = {
            // ReceiveState is consistent after every call, so a poisoned lock is still usable.
            let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            state.next_frame(data, now)
        };
        let Some(frame) = frame else { return };
        self.latest.store(Arc::new(frame));
        if let Some(notify) = &self.notify {
            notify();
        }
        if let Some((sock, addr)) = self.forward.as_ref() {
            let _ = sock.send_to(bytes, addr);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn packet(race_on: i32, ts: u32) -> FH6TelemetryData {
        FH6TelemetryData {
            is_race_on: race_on,
            timestamp_ms: ts,
            ..FH6TelemetryData::default()
        }
    }

    #[test]
    fn empty_frame_is_stale_and_not_driving() {
        let frame = TelemetryFrame::empty();
        let now = Instant::now();
        assert!(frame.is_stale(now));
        assert!(!frame.is_driving_active(now));
        assert_eq!(frame.time_since_driving_resumed_ms(now), 0.0);
    }

    #[test]
    fn duplicate_from_second_port_is_dropped() {
        let mut state = ReceiveState::default();
        let t0 = Instant::now();
        assert!(state.next_frame(packet(1, 100), t0).is_some());
        assert!(state
            .next_frame(packet(1, 100), t0 + Duration::from_millis(1))
            .is_none());
        let next = state
            .next_frame(packet(1, 116), t0 + Duration::from_millis(16))
            .expect("new timestamp is published");
        assert_eq!(next.seq, 2);
    }

    #[test]
    fn pause_flip_with_same_timestamp_is_not_dropped() {
        let mut state = ReceiveState::default();
        let t0 = Instant::now();
        assert!(state.next_frame(packet(1, 500), t0).is_some());
        let paused = state.next_frame(packet(0, 500), t0 + Duration::from_millis(2));
        assert!(paused.is_some(), "IsRaceOn changed, so it is a new packet");
    }

    #[test]
    fn race_resume_is_tracked() {
        let mut state = ReceiveState::default();
        let t0 = Instant::now();
        let f1 = state.next_frame(packet(1, 1), t0).expect("frame");
        assert_eq!(f1.race_resumed_at, Some(t0));
        let f2 = state
            .next_frame(packet(1, 2), t0 + Duration::from_millis(16))
            .expect("frame");
        assert_eq!(f2.race_resumed_at, Some(t0), "continuous racing keeps the resume time");
        let f3 = state
            .next_frame(packet(0, 3), t0 + Duration::from_millis(32))
            .expect("frame");
        assert_eq!(f3.race_resumed_at, None);
        let t4 = t0 + Duration::from_millis(48);
        let f4 = state.next_frame(packet(1, 4), t4).expect("frame");
        assert_eq!(f4.race_resumed_at, Some(t4));
        assert!(f4.is_driving_active(t4 + Duration::from_millis(10)));
        assert!(
            !f4.is_driving_active(t4 + Duration::from_millis(300)),
            "stale after 250 ms"
        );
    }

    #[test]
    fn engine_receives_and_notifies() {
        use std::sync::atomic::AtomicUsize;
        // Bind to an OS-chosen free port to keep the test independent of a running game.
        let probe = UdpSocket::bind("127.0.0.1:0").expect("probe bind");
        let port = probe.local_addr().expect("probe addr").port();
        drop(probe);

        let hits = Arc::new(AtomicUsize::new(0));
        let notify: PacketNotify = {
            let hits = Arc::clone(&hits);
            Arc::new(move || {
                hits.fetch_add(1, Ordering::SeqCst);
            })
        };
        let mut engine = TelemetryEngine::new(TelemetryPorts {
            primary: port,
            fallback: None,
            forward: None,
            all_interfaces: false,
        })
        .with_notify(notify);
        engine.start().expect("engine starts");

        let mut pkt = [0u8; PACKET_LEN];
        pkt[0..4].copy_from_slice(&1i32.to_le_bytes());
        pkt[4..8].copy_from_slice(&42u32.to_le_bytes());
        pkt[256..260].copy_from_slice(&30.0f32.to_le_bytes());
        let sender = UdpSocket::bind("127.0.0.1:0").expect("sender bind");
        sender.send_to(&pkt, ("127.0.0.1", port)).expect("send");

        let deadline = Instant::now() + Duration::from_secs(2);
        // The frame is stored before the notifier runs, so wait for both.
        while (engine.latest().seq == 0 || hits.load(Ordering::SeqCst) == 0) && Instant::now() < deadline {
            thread::sleep(Duration::from_millis(2));
        }
        let frame = engine.latest();
        assert_eq!(frame.seq, 1);
        assert_eq!(frame.data.timestamp_ms, 42);
        assert_eq!(frame.data.speed_ms, 30.0);
        assert_eq!(hits.load(Ordering::SeqCst), 1);
        engine.shutdown();
    }
}
