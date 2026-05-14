use crate::error::JError;
use std::cell::RefCell;

pub trait EnvWriter {
    /// Returns the current user PATH. An `Err` means the registry/state
    /// could not be read reliably; callers must NOT assume an empty PATH
    /// on failure (doing so would erase an existing PATH on `write_path`).
    /// An `Ok("")` specifically means "the PATH value is unset" (e.g., fresh user).
    fn read_path(&self) -> Result<String, JError>;
    fn write_path(&self, new_value: &str) -> Result<(), JError>;

    fn ensure_path_contains(&self, entry: &str) -> Result<(), JError> {
        let cur = self.read_path()?;
        let mut parts: Vec<&str> = cur.split(';').filter(|s| !s.is_empty()).collect();
        if parts.iter().any(|p| p.eq_ignore_ascii_case(entry)) {
            return Ok(());
        }
        parts.push(entry);
        self.write_path(&parts.join(";"))
    }

    fn ensure_path_not_contains(&self, entry: &str) -> Result<(), JError> {
        let cur = self.read_path()?;
        let parts: Vec<&str> = cur
            .split(';')
            .filter(|s| !s.is_empty() && !s.eq_ignore_ascii_case(entry))
            .collect();
        self.write_path(&parts.join(";"))
    }
}

pub struct MockEnvWriter {
    inner: RefCell<String>,
}

impl MockEnvWriter {
    pub fn new(initial: &str) -> Self {
        Self {
            inner: RefCell::new(initial.to_string()),
        }
    }
}

impl EnvWriter for MockEnvWriter {
    fn read_path(&self) -> Result<String, JError> {
        Ok(self.inner.borrow().clone())
    }
    fn write_path(&self, v: &str) -> Result<(), JError> {
        *self.inner.borrow_mut() = v.to_string();
        Ok(())
    }
}

#[cfg(windows)]
pub struct WinRegEnvWriter;

#[cfg(windows)]
impl EnvWriter for WinRegEnvWriter {
    fn read_path(&self) -> Result<String, JError> {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        // HKCU\Environment is per-user and auto-created by Windows; this
        // should never fail in a healthy profile, but we surface the error
        // so ensure_path_contains doesn't clobber an unreadable PATH.
        let env = hkcu
            .open_subkey("Environment")
            .map_err(|e| JError::InstallError {
                msg: format!("open HKCU\\Environment: {}", e),
            })?;
        match env.get_value::<String, _>("Path") {
            Ok(v) => Ok(v),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(String::new()),
            Err(e) => Err(JError::InstallError {
                msg: format!("read HKCU\\Environment\\Path: {}", e),
            }),
        }
    }

    fn write_path(&self, v: &str) -> Result<(), JError> {
        use winreg::enums::*;
        use winreg::RegKey;
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        let (env, _) = hkcu
            .create_subkey("Environment")
            .map_err(|e| JError::InstallError {
                msg: format!("open HKCU\\Environment: {}", e),
            })?;
        env.set_value("Path", &v.to_string())
            .map_err(|e| JError::InstallError {
                msg: format!("set Path: {}", e),
            })?;
        broadcast_settingchange();
        Ok(())
    }
}

#[cfg(windows)]
fn broadcast_settingchange() {
    use windows::Win32::Foundation::{LPARAM, WPARAM};
    use windows::Win32::UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE,
    };
    // "Environment" as wide string literal via windows::core::w! macro
    let env_wide = windows::core::w!("Environment");
    unsafe {
        let _ = SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(env_wide.as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            5000,
            None,
        );
    }
}

pub mod cmd;
pub mod posix;
pub mod powershell;
pub mod region;
