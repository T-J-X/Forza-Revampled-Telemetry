use fh6_core::telemetry::decode;
use std::fs;
use std::path::Path;

#[test]
fn test_all_20_golden_packets_match_json() {
    let packet_dir = Path::new("../../tests/golden/packets");
    let fallback_dir = Path::new("tests/golden/packets");
    let dir = if packet_dir.exists() { packet_dir } else { fallback_dir };
    
    // Check if dir exists relative to workspace root
    let dir = if dir.exists() {
        dir.to_path_buf()
    } else {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/golden/packets")
    };
    
    assert!(dir.exists(), "Golden packets directory not found at {:?}", dir);
    
    let mut count = 0;
    for i in 1..=20 {
        let bin_name = format!("{:02}.bin", i);
        let json_name = format!("{:02}.json", i);
        let bin_path = dir.join(&bin_name);
        let json_path = dir.join(&json_name);
        
        assert!(bin_path.exists(), "Missing golden packet {:?}", bin_path);
        assert!(json_path.exists(), "Missing golden json {:?}", json_path);
        
        let bytes = fs::read(&bin_path).expect("read bin");
        assert_eq!(bytes.len(), 324, "Packet must be exactly 324 bytes");
        
        let data = decode(&bytes).expect("decode packet");
        let json_str = fs::read_to_string(&json_path).expect("read json");
        let expected: serde_json::Value = serde_json::from_str(&json_str).expect("parse json");
        
        assert_eq!(data.is_race_on, expected["IsRaceOn"].as_i64().unwrap() as i32);
        assert_eq!(data.timestamp_ms, expected["TimestampMS"].as_u64().unwrap() as u32);
        assert_eq!(data.gear, expected["Gear"].as_u64().unwrap() as u8);
        assert_eq!(data.accel, expected["Accel"].as_u64().unwrap() as u8);
        assert_eq!(data.brake, expected["Brake"].as_u64().unwrap() as u8);
        assert_eq!(data.car_ordinal, expected["CarOrdinal"].as_i64().unwrap() as i32);
        count += 1;
    }
    assert_eq!(count, 20, "Must test all 20 golden packets");
}
