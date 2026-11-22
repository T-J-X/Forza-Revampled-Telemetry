//! Finds the Forza Horizon 6 install: `GameInstallDir` from the config first, then every Steam
//! library listed in `steamapps\libraryfolders.vdf` (parsed here in pure Rust), under the Steam
//! root from the registry (`HKCU\Software\Valve\Steam\SteamPath`, then
//! `HKLM\SOFTWARE\WOW6432Node\Valve\Steam\InstallPath`) or the default install folder.
use std::fs;
use std::path::{Path, PathBuf};

/// The game's folder under `steamapps\common`.
pub const GAME_FOLDER: &str = "ForzaHorizon6";
/// The camera files' folder under the install.
pub const CAMERA_SUBDIR: [&str; 2] = ["mediapc", "Camera"];
/// The default Steam folder when the registry has nothing.
pub const DEFAULT_STEAM_ROOT: &str = r"C:\Program Files (x86)\Steam";

/// A parsed Valve KeyValues (VDF) node: either a string value or a block of children, in file
/// order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Vdf {
    /// `"key" "value"`.
    Value(String),
    /// `"key" { ... }`.
    Block(Vec<(String, Vdf)>),
}

impl Vdf {
    /// The first child named `key` (case-insensitive), for a block.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&Vdf> {
        match self {
            Vdf::Block(children) => children
                .iter()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
                .map(|(_, v)| v),
            Vdf::Value(_) => None,
        }
    }

    /// The string value, for a value node.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Vdf::Value(s) => Some(s),
            Vdf::Block(_) => None,
        }
    }
}

/// Why a VDF text could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum VdfError {
    /// A quoted string was not closed.
    #[error("unterminated string")]
    UnterminatedString,
    /// A `}` without a block, or a block that was never closed.
    #[error("unbalanced braces")]
    Unbalanced,
    /// A key without a value or block.
    #[error("key `{0}` has no value")]
    MissingValue(String),
}

#[derive(Debug, PartialEq, Eq)]
enum Tok {
    Str(String),
    Open,
    Close,
}

fn tokenize(text: &str) -> Result<Vec<Tok>, VdfError> {
    let mut out = Vec::new();
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' => {
                let mut s = String::new();
                loop {
                    match chars.next() {
                        None => return Err(VdfError::UnterminatedString),
                        Some('"') => break,
                        Some('\\') => match chars.next() {
                            Some('n') => s.push('\n'),
                            Some('t') => s.push('\t'),
                            Some(other) => s.push(other),
                            None => return Err(VdfError::UnterminatedString),
                        },
                        Some(other) => s.push(other),
                    }
                }
                out.push(Tok::Str(s));
            }
            '{' => out.push(Tok::Open),
            '}' => out.push(Tok::Close),
            '/' if chars.peek() == Some(&'/') => {
                for n in chars.by_ref() {
                    if n == '\n' {
                        break;
                    }
                }
            }
            c if c.is_whitespace() => {}
            // Unquoted tokens (rare in Steam files): read up to whitespace or a brace.
            c => {
                let mut s = String::from(c);
                while let Some(&n) = chars.peek() {
                    if n.is_whitespace() || n == '{' || n == '}' || n == '"' {
                        break;
                    }
                    s.push(n);
                    chars.next();
                }
                out.push(Tok::Str(s));
            }
        }
    }
    Ok(out)
}

/// Parses VDF text into a root block.
///
/// # Errors
/// [`VdfError`] for unterminated strings, unbalanced braces or a dangling key.
pub fn parse_vdf(text: &str) -> Result<Vdf, VdfError> {
    let toks = tokenize(text.strip_prefix('\u{feff}').unwrap_or(text))?;
    let mut stack: Vec<Vec<(String, Vdf)>> = vec![Vec::new()];
    let mut keys: Vec<String> = Vec::new();
    let mut it = toks.into_iter().peekable();
    while let Some(t) = it.next() {
        match t {
            Tok::Str(key) => match it.next() {
                Some(Tok::Str(value)) => stack
                    .last_mut()
                    .ok_or(VdfError::Unbalanced)?
                    .push((key, Vdf::Value(value))),
                Some(Tok::Open) => {
                    keys.push(key);
                    stack.push(Vec::new());
                }
                Some(Tok::Close) | None => return Err(VdfError::MissingValue(key)),
            },
            Tok::Close => {
                let block = stack.pop().ok_or(VdfError::Unbalanced)?;
                let key = keys.pop().ok_or(VdfError::Unbalanced)?;
                stack
                    .last_mut()
                    .ok_or(VdfError::Unbalanced)?
                    .push((key, Vdf::Block(block)));
            }
            Tok::Open => return Err(VdfError::Unbalanced),
        }
    }
    if stack.len() != 1 {
        return Err(VdfError::Unbalanced);
    }
    Ok(Vdf::Block(stack.pop().unwrap_or_default()))
}

/// The library folders listed in a `libraryfolders.vdf` text (new `"path"` format and the old
/// numbered-value format), in file order.
#[must_use]
pub fn library_paths(vdf_text: &str) -> Vec<PathBuf> {
    let Ok(root) = parse_vdf(vdf_text) else {
        return Vec::new();
    };
    let Some(Vdf::Block(libs)) = root.get("libraryfolders").or_else(|| root.get("LibraryFolders")) else {
        return Vec::new();
    };
    libs.iter()
        .filter(|(k, _)| k.chars().all(|c| c.is_ascii_digit()))
        .filter_map(|(_, v)| match v {
            Vdf::Block(_) => v.get("path").and_then(Vdf::as_str).map(PathBuf::from),
            Vdf::Value(p) => Some(PathBuf::from(p)),
        })
        .collect()
}

/// Whether `dir` looks like the game install (has the camera settings file).
#[must_use]
pub fn is_game_install(dir: &Path) -> bool {
    camera_dir(dir).join("CameraSettings.ini").is_file()
}

/// The camera folder of an install.
#[must_use]
pub fn camera_dir(install: &Path) -> PathBuf {
    let mut p = install.to_path_buf();
    for part in CAMERA_SUBDIR {
        p.push(part);
    }
    p
}

/// The first library (Steam root first) that holds the game.
#[must_use]
pub fn find_in_libraries(steam_root: &Path, libraries: &[PathBuf]) -> Option<PathBuf> {
    std::iter::once(steam_root.to_path_buf())
        .chain(libraries.iter().cloned())
        .map(|lib| lib.join("steamapps").join("common").join(GAME_FOLDER))
        .find(|dir| is_game_install(dir))
}

/// The game install: `override_dir` when it is an install, else the Steam libraries.
#[must_use]
pub fn find_game_install(override_dir: &str) -> Option<PathBuf> {
    let o = override_dir.trim();
    if !o.is_empty() {
        let p = PathBuf::from(o);
        if is_game_install(&p) {
            return Some(p);
        }
        tracing::warn!(
            dir = o,
            "GameInstallDir has no mediapc\\Camera\\CameraSettings.ini; searching Steam"
        );
    }
    let root = steam_root().unwrap_or_else(|| PathBuf::from(DEFAULT_STEAM_ROOT));
    let vdf = fs::read_to_string(root.join("steamapps").join("libraryfolders.vdf")).unwrap_or_default();
    find_in_libraries(&root, &library_paths(&vdf))
}

/// The Steam root from the registry.
#[cfg(windows)]
#[must_use]
pub fn steam_root() -> Option<PathBuf> {
    use windows::core::w;
    use windows::Win32::System::Registry::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
    reg_string(HKEY_CURRENT_USER, w!(r"Software\Valve\Steam"), w!("SteamPath"))
        .or_else(|| {
            reg_string(
                HKEY_LOCAL_MACHINE,
                w!(r"SOFTWARE\WOW6432Node\Valve\Steam"),
                w!("InstallPath"),
            )
        })
        .map(|s| PathBuf::from(s.replace('/', "\\")))
        .filter(|p| p.is_dir())
}

/// The Steam root (non-Windows builds have none).
#[cfg(not(windows))]
#[must_use]
pub fn steam_root() -> Option<PathBuf> {
    None
}

#[cfg(windows)]
fn reg_string(
    key: windows::Win32::System::Registry::HKEY,
    subkey: windows::core::PCWSTR,
    value: windows::core::PCWSTR,
) -> Option<String> {
    use windows::Win32::Foundation::ERROR_SUCCESS;
    use windows::Win32::System::Registry::{RegGetValueW, RRF_RT_REG_SZ};
    let mut buf = [0u16; 1024];
    let mut bytes = u32::try_from(std::mem::size_of_val(&buf)).unwrap_or(0);
    // SAFETY: `subkey` and `value` are static NUL-terminated wide strings; `buf` is a live
    // buffer of `bytes` bytes that RegGetValueW fills with a NUL-terminated REG_SZ, and it
    // writes the used size back into `bytes`.
    let status = unsafe {
        RegGetValueW(
            key,
            subkey,
            value,
            RRF_RT_REG_SZ,
            None,
            Some(buf.as_mut_ptr().cast()),
            Some(&mut bytes),
        )
    };
    if status != ERROR_SUCCESS {
        return None;
    }
    let chars = (bytes as usize / 2).min(buf.len());
    let s = String::from_utf16_lossy(buf.get(..chars)?);
    let s = s.trim_end_matches('\0').to_string();
    (!s.is_empty()).then_some(s)
}

#[cfg(test)]
mod tests {
    use super::*;

    const NEW_FORMAT: &str = r#"
"libraryfolders"
{
	"0"
	{
		"path"		"C:\\Program Files (x86)\\Steam"
		"label"		""
		"contentid"		"123"
		"apps"
		{
			"228980"		"123"
		}
	}
	"1"
	{
		"path"		"E:\\SteamLibrary"
		"label"		""
		"apps"
		{
		}
	}
}
"#;

    const OLD_FORMAT: &str = r#"
"LibraryFolders"
{
	"TimeNextStatsReport"		"1234"
	"ContentStatsID"		"-1"
	"1"		"D:\\Games\\Steam" // a comment
}
"#;

    #[test]
    fn parses_both_libraryfolders_formats() {
        assert_eq!(
            library_paths(NEW_FORMAT),
            vec![
                PathBuf::from(r"C:\Program Files (x86)\Steam"),
                PathBuf::from(r"E:\SteamLibrary")
            ]
        );
        assert_eq!(library_paths(OLD_FORMAT), vec![PathBuf::from(r"D:\Games\Steam")]);
    }

    #[test]
    fn malformed_vdf_is_an_error_not_a_panic() {
        assert_eq!(parse_vdf("\"a\" { \"b\" \"c\""), Err(VdfError::Unbalanced));
        assert_eq!(parse_vdf("\"a"), Err(VdfError::UnterminatedString));
        assert_eq!(parse_vdf("\"a\""), Err(VdfError::MissingValue("a".into())));
        assert_eq!(parse_vdf("}"), Err(VdfError::Unbalanced));
        assert!(library_paths("garbage {").is_empty());
    }

    #[test]
    fn finds_the_game_in_a_library() {
        let root = std::env::temp_dir().join(format!("fh6_steam_{}", std::process::id()));
        let lib = root.join("lib2");
        let cam = camera_dir(&lib.join("steamapps").join("common").join(GAME_FOLDER));
        fs::create_dir_all(&cam).unwrap();
        fs::write(cam.join("CameraSettings.ini"), "CockpitCam\\FOV 80\r\n").unwrap();
        let found = find_in_libraries(&root.join("steam"), &[root.join("lib1"), lib.clone()]);
        assert_eq!(found, Some(lib.join("steamapps").join("common").join(GAME_FOLDER)));
        assert!(find_in_libraries(&root.join("steam"), &[root.join("lib1")]).is_none());
        let found = find_game_install(found.as_deref().and_then(Path::to_str).unwrap_or(""));
        assert!(found.is_some(), "override honoured");
        fs::remove_dir_all(&root).ok();
    }
}
