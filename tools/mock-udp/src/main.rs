use std::net::UdpSocket;
use std::time::{Duration, Instant};

const PACKET_SIZE: usize = 324;

#[allow(clippy::too_many_arguments)]
fn build_packet(
    is_race_on: i32,
    timestamp_ms: u32,
    engine_max_rpm: f32,
    engine_idle_rpm: f32,
    current_engine_rpm: f32,
    accel_x: f32,
    accel_y: f32,
    accel_z: f32,
    vel_x: f32,
    vel_y: f32,
    vel_z: f32,
    car_ordinal: i32,
    drivetrain_type: i32,
    speed_ms: f32,
    power_watts: f32,
    boost_psi: f32,
    accel: u8,
    brake: u8,
    clutch: u8,
    handbrake: u8,
    gear: u8,
    steer: i8,
) -> [u8; PACKET_SIZE] {
    let mut buf = [0u8; PACKET_SIZE];
    buf[0..4].copy_from_slice(&is_race_on.to_le_bytes());
    buf[4..8].copy_from_slice(&timestamp_ms.to_le_bytes());
    buf[8..12].copy_from_slice(&engine_max_rpm.to_le_bytes());
    buf[12..16].copy_from_slice(&engine_idle_rpm.to_le_bytes());
    buf[16..20].copy_from_slice(&current_engine_rpm.to_le_bytes());
    buf[20..24].copy_from_slice(&accel_x.to_le_bytes());
    buf[24..28].copy_from_slice(&accel_y.to_le_bytes());
    buf[28..32].copy_from_slice(&accel_z.to_le_bytes());
    buf[32..36].copy_from_slice(&vel_x.to_le_bytes());
    buf[36..40].copy_from_slice(&vel_y.to_le_bytes());
    buf[40..44].copy_from_slice(&vel_z.to_le_bytes());
    buf[212..216].copy_from_slice(&car_ordinal.to_le_bytes());
    buf[224..228].copy_from_slice(&drivetrain_type.to_le_bytes());
    buf[256..260].copy_from_slice(&speed_ms.to_le_bytes());
    buf[260..264].copy_from_slice(&power_watts.to_le_bytes());
    buf[284..288].copy_from_slice(&boost_psi.to_le_bytes());
    buf[315] = accel;
    buf[316] = brake;
    buf[317] = clutch;
    buf[318] = handbrake;
    buf[319] = gear;
    buf[320] = steer as u8;
    buf
}

fn send_packet(sock: &UdpSocket, pkt: &[u8], ip: &str, port: u16) -> std::io::Result<()> {
    let addr = format!("{}:{}", ip, port);
    sock.send_to(pkt, addr)?;
    Ok(())
}

fn scenario_bmw_m4_cs(t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    let cycle = (t % 8.0) / 8.0;
    let gear = (1.0 + cycle * 6.0).floor() as u8;
    let rpm_ratio = ((t * 3.0).sin() + 1.0) / 2.0;
    let current_rpm = 800.0 + rpm_ratio * (7600.0 - 800.0);
    let speed = 10.0 + cycle * 65.0;
    let accel = (150.0 + 105.0 * rpm_ratio).clamp(0.0, 255.0) as u8;
    let steer = (30.0 * (t * 1.5).sin()) as i8;
    build_packet(
        1,
        0,
        7600.0,
        800.0,
        current_rpm,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        ordinal,
        1,
        speed,
        120000.0 + rpm_ratio * 380000.0,
        5.0 + rpm_ratio * 18.0,
        accel,
        0,
        0,
        0,
        gear,
        steer,
    )
}

fn scenario_ferrari_sf90(t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    let speed = 15.0 + (t * 10.0) % 80.0;
    let gear = ((1.0 + speed / 15.0).floor()).min(8.0) as u8;
    let current_rpm = 1500.0 + (t * 2000.0) % 6500.0;
    build_packet(
        1,
        0,
        8000.0,
        1000.0,
        current_rpm,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        ordinal,
        2,
        speed,
        735000.0,
        20.0,
        255,
        0,
        0,
        0,
        gear,
        0,
    )
}

fn scenario_rimac_nevera(t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    let speed = (5.0 + t * 12.0).min(115.0);
    let power = (100000.0 + t * 150000.0).min(1400000.0);
    build_packet(
        1, 0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, ordinal, 2, speed, power, 0.0, 255, 0, 0, 0, 1, 0,
    )
}

fn scenario_high_rpm_swap(t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    let phase = (t % 4.0) / 4.0;
    let current_rpm = 1200.0 + phase * (11500.0 - 1200.0);
    let gear = (((t / 4.0).floor() as i32 % 6) + 1) as u8;
    build_packet(
        1,
        0,
        11500.0,
        1200.0,
        current_rpm,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        0.0,
        ordinal,
        1,
        45.0,
        0.0,
        0.0,
        0,
        0,
        0,
        0,
        gear,
        0,
    )
}

fn scenario_camera_sync(t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    let steer = (127.0 * (t * 4.0).sin()) as i8;
    let lateral_g = 25.0 * (t * 4.0).sin();
    build_packet(
        1, 0, 7600.0, 800.0, 5500.0, lateral_g, 2.0, 0.0, 0.0, 0.0, 0.0, ordinal, 1, 0.0, 0.0, 0.0, 0, 0, 0, 0, 0,
        steer,
    )
}

fn scenario_watchdog_drop(_t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    build_packet(
        1, 0, 0.0, 0.0, 4000.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, ordinal, 1, 30.0, 0.0, 0.0, 0, 0, 0, 0, 3, 0,
    )
}

fn scenario_night_tunnel(t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    let rpm = 4000.0 + 3000.0 * (t * 2.0).sin();
    build_packet(
        1, 0, 0.0, 0.0, rpm, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, ordinal, 1, 35.0, 0.0, 0.0, 0, 0, 0, 0, 3, 0,
    )
}

/// Racing for 4 s, then paused in the menu (`IsRaceOn = 0`) for 4 s, repeating.
/// Exercises the overlay's hide-on-pause rule.
fn scenario_menu_pause(t: f32, ordinal: i32) -> [u8; PACKET_SIZE] {
    let mut pkt = scenario_bmw_m4_cs(t, ordinal);
    let racing = (t % 8.0) < 4.0;
    pkt[0..4].copy_from_slice(&i32::from(racing).to_le_bytes());
    pkt
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut scenario = "bmw_m4_cs";
    let mut port: u16 = 8000;
    let mut fallback_port: u16 = 5500;
    let mut forward_port: u16 = 0;
    let mut ip = "127.0.0.1";
    let mut duration: f32 = 10.0;
    let mut rate_hz: f32 = 60.0;
    let mut once = false;
    let mut race_on: Option<i32> = None;
    let mut ordinal: i32 = 4122;
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--scenario" if i + 1 < args.len() => {
                scenario = &args[i + 1];
                i += 1;
            }
            "--port" if i + 1 < args.len() => {
                port = args[i + 1].parse().unwrap_or(port);
                i += 1;
            }
            "--fallback-port" if i + 1 < args.len() => {
                fallback_port = args[i + 1].parse().unwrap_or(fallback_port);
                i += 1;
            }
            "--forward-port" if i + 1 < args.len() => {
                forward_port = args[i + 1].parse().unwrap_or(forward_port);
                i += 1;
            }
            "--ip" if i + 1 < args.len() => {
                ip = &args[i + 1];
                i += 1;
            }
            "--duration" if i + 1 < args.len() => {
                duration = args[i + 1].parse().unwrap_or(duration);
                i += 1;
            }
            "--rate-hz" if i + 1 < args.len() => {
                rate_hz = args[i + 1].parse().unwrap_or(rate_hz);
                i += 1;
            }
            "--ordinal" if i + 1 < args.len() => {
                ordinal = args[i + 1].parse().unwrap_or(ordinal);
                i += 1;
            }
            "--race-on" if i + 1 < args.len() => {
                race_on = args[i + 1].parse().ok();
                i += 1;
            }
            "--once" => {
                once = true;
            }
            _ => {}
        }
        i += 1;
    }

    let sock = UdpSocket::bind("0.0.0.0:0").expect("bind udp");
    let interval = Duration::from_secs_f32(1.0 / rate_hz.max(1.0));
    let start = Instant::now();
    let mut sent = 0usize;

    println!(
        "[FH6 Mock UDP] scenario={} -> {}:{} / {} @ {}Hz for {}s",
        scenario, ip, port, fallback_port, rate_hz, duration
    );

    loop {
        let t = start.elapsed().as_secs_f32();
        if t > duration && !once {
            break;
        }
        let ts_ms = start.elapsed().as_millis() as u32;
        let pkt = match scenario {
            "bmw_m4_cs" => scenario_bmw_m4_cs(t, ordinal),
            "ferrari_sf90" => scenario_ferrari_sf90(t, ordinal),
            "rimac_nevera" => scenario_rimac_nevera(t, ordinal),
            "high_rpm_swap" => scenario_high_rpm_swap(t, ordinal),
            "camera_sync" => scenario_camera_sync(t, ordinal),
            "watchdog_drop" => scenario_watchdog_drop(t, ordinal),
            "night_tunnel" => scenario_night_tunnel(t, ordinal),
            "menu_pause" => scenario_menu_pause(t, ordinal),
            _ => scenario_bmw_m4_cs(t, ordinal),
        };
        // build with timestamp
        let mut pkt_mut = pkt;
        pkt_mut[4..8].copy_from_slice(&ts_ms.to_le_bytes());
        if let Some(flag) = race_on {
            pkt_mut[0..4].copy_from_slice(&flag.to_le_bytes());
        }
        let _ = send_packet(&sock, &pkt_mut, ip, port);
        if fallback_port != port {
            let _ = send_packet(&sock, &pkt_mut, ip, fallback_port);
        }
        if forward_port > 0 {
            let _ = send_packet(&sock, &pkt_mut, ip, forward_port);
        }
        sent += 1;
        if once {
            break;
        }
        std::thread::sleep(interval);
    }
    println!("[FH6 Mock UDP] Completed. Sent {} packets.", sent);
}
