use std::time::Duration;

#[cfg(windows)]
use windows::Win32::System::Diagnostics::ToolHelp::{CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W, TH32CS_SNAPPROCESS};
#[cfg(windows)]
use windows::Win32::Foundation::CloseHandle;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    Stopped,
    Started,
}

#[derive(Debug, Clone)]
pub struct ProcessInfo {
    pub name: String,
    pub pid: u32,
}

pub struct ProcessWatcher {
    target_name: String,
    poll_interval_ms: u64,
    last_state: GameState,
}

pub fn enumerate_processes() -> Vec<ProcessInfo> {
    #[cfg(windows)]
    {
        let snapshot_res = unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0) };
        let handle = match snapshot_res {
            Ok(h) if !h.0.is_null() => h,
            _ => return Vec::new(),
        };
        let mut entry = PROCESSENTRY32W {
            dwSize: std::mem::size_of::<PROCESSENTRY32W>() as u32,
            ..Default::default()
        };
        let mut processes = Vec::new();
        unsafe {
            if Process32FirstW(handle, &mut entry).is_ok() {
                loop {
                    let len = entry.szExeFile.iter().position(|&c| c == 0).unwrap_or(260);
                    let name = String::from_utf16_lossy(&entry.szExeFile[..len]);
                    if !name.is_empty() {
                        processes.push(ProcessInfo { name, pid: entry.th32ProcessID });
                    }
                    if Process32NextW(handle, &mut entry).is_err() {
                        break;
                    }
                }
            }
            let _ = CloseHandle(handle);
        }
        processes
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

impl ProcessWatcher {
    pub fn new(target_name: &str, poll_interval_ms: u64) -> Self {
        Self {
            target_name: target_name.to_lowercase(),
            poll_interval_ms,
            last_state: GameState::Stopped,
        }
    }

    pub fn check(&mut self, processes: &[ProcessInfo]) -> Option<GameState> {
        let found = processes.iter().any(|p| p.name.to_lowercase() == self.target_name);
        let current = if found { GameState::Started } else { GameState::Stopped };
        let changed = current != self.last_state;
        self.last_state = current;
        if changed { Some(current) } else { None }
    }

    pub fn poll_interval(&self) -> Duration {
        Duration::from_millis(self.poll_interval_ms)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn process_detected_transitions() {
        let mut watcher = ProcessWatcher::new("ForzaHorizon6.exe", 2000);
        let empty: Vec<ProcessInfo> = vec![];
        assert_eq!(watcher.check(&empty), None);

        let procs = vec![ProcessInfo { name: "ForzaHorizon6.exe".to_string(), pid: 1234 }];
        assert_eq!(watcher.check(&procs), Some(GameState::Started));

        assert_eq!(watcher.check(&empty), Some(GameState::Stopped));
    }

    #[test]
    fn case_insensitive_match() {
        let mut watcher = ProcessWatcher::new("forzahorizon6.exe", 1000);
        let procs = vec![ProcessInfo { name: "ForzaHorizon6.EXE".to_string(), pid: 1 }];
        assert_eq!(watcher.check(&procs), Some(GameState::Started));
    }
}
