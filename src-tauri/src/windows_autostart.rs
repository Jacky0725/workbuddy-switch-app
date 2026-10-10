//! Current-user startup with a quoted executable path and authoritative readback.
use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, KEY_READ, KEY_SET_VALUE, REG_BINARY};
use winreg::{RegKey, RegValue};

const ENTRY: &str = "WorkBuddy Manager";
const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const APPROVED: &str = r"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

fn startup_command(path: &Path) -> Result<String, String> {
    let text = path.to_str().ok_or("自启动程序路径无法编码")?;
    if !path.is_absolute() || text.contains('"') || text.chars().any(char::is_control) {
        return Err("自启动程序路径无效".into());
    }
    Ok(format!("\"{text}\" --hidden"))
}

fn read_enabled(run: &RegKey, approved: &RegKey, command: &str) -> Result<bool, String> {
    let actual: String = match run.get_value(ENTRY) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("读取自启动注册失败：{error}")),
    };
    if actual != command {
        return Ok(false);
    }
    match approved.get_raw_value(ENTRY) {
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(true),
        Err(error) => Err(format!("读取自启动系统状态失败：{error}")),
        Ok(value) if value.vtype == REG_BINARY && value.bytes.len() == 12 => {
            Ok(value.bytes[0] == 2 && value.bytes[1..].iter().all(|byte| *byte == 0))
        }
        Ok(_) => Err("自启动系统状态格式无法识别".into()),
    }
}

fn write_enabled(
    run: &RegKey,
    approved: &RegKey,
    command: &str,
    enabled: bool,
) -> Result<bool, String> {
    let previous_run = read_raw(run)?;
    let previous_approved = read_raw(approved)?;
    let result = apply_enabled(run, approved, command, enabled);
    if let Err(error) = result {
        let mut failures = Vec::new();
        for (key, previous) in [(run, previous_run), (approved, previous_approved)] {
            // Avoid writing an unchanged value, including read-only system state.
            match read_raw(key) {
                Ok(current) if current == previous => continue,
                _ => {}
            }
            if let Err(failure) = restore_raw(key, previous.as_ref()) {
                failures.push(failure);
            }
        }
        return if failures.is_empty() {
            Err(error)
        } else {
            Err(format!(
                "{error}；恢复原自启动设置失败：{}",
                failures.join("；")
            ))
        };
    }
    result
}

fn read_raw(key: &RegKey) -> Result<Option<RegValue>, String> {
    match key.get_raw_value(ENTRY) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(format!("读取自启动原设置失败：{error}")),
    }
}

fn restore_raw(key: &RegKey, value: Option<&RegValue>) -> Result<(), String> {
    let result = match value {
        Some(value) => key.set_raw_value(ENTRY, value),
        None => key.delete_value(ENTRY),
    };
    match result {
        Ok(()) => Ok(()),
        Err(error) if value.is_none() && error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn apply_enabled(
    run: &RegKey,
    approved: &RegKey,
    command: &str,
    enabled: bool,
) -> Result<bool, String> {
    if enabled {
        run.set_value(ENTRY, &command)
            .map_err(|error| format!("注册自启动失败：{error}"))?;
        // This is changed only by the user's explicit enable action.
        approved
            .set_raw_value(
                ENTRY,
                &RegValue {
                    vtype: REG_BINARY,
                    bytes: vec![2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                },
            )
            .map_err(|error| format!("更新自启动系统状态失败：{error}"))?;
    } else {
        for key in [run, approved] {
            match key.delete_value(ENTRY) {
                Ok(()) => {}
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => return Err(format!("关闭自启动失败：{error}")),
            }
        }
    }
    let actual = read_enabled(run, approved, command)?;
    if actual != enabled {
        return Err("自启动设置后回读不一致".into());
    }
    Ok(actual)
}

pub fn is_enabled() -> Result<bool, String> {
    let command = startup_command(&std::env::current_exe().map_err(|error| error.to_string())?)?;
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let run = match root.open_subkey_with_flags(RUN, KEY_READ) {
        Ok(key) => key,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(format!("读取自启动注册失败：{error}")),
    };
    let approved = match root.open_subkey_with_flags(APPROVED, KEY_READ) {
        Ok(key) => key,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return run
                .get_value::<String, _>(ENTRY)
                .map(|value| value == command)
                .or_else(|error| {
                    if error.kind() == std::io::ErrorKind::NotFound {
                        Ok(false)
                    } else {
                        Err(error.to_string())
                    }
                });
        }
        Err(error) => return Err(format!("读取自启动系统状态失败：{error}")),
    };
    read_enabled(&run, &approved, &command)
}

pub fn set_enabled(enabled: bool) -> Result<bool, String> {
    let command = startup_command(&std::env::current_exe().map_err(|error| error.to_string())?)?;
    let root = RegKey::predef(HKEY_CURRENT_USER);
    let flags = KEY_READ | KEY_SET_VALUE;
    let (run, _) = root
        .create_subkey_with_flags(RUN, flags)
        .map_err(|error| error.to_string())?;
    let (approved, _) = root
        .create_subkey_with_flags(APPROVED, flags)
        .map_err(|error| error.to_string())?;
    write_enabled(&run, &approved, &command, enabled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_command_quotes_spaces_and_rejects_relative_or_injected_paths() {
        assert_eq!(
            startup_command(Path::new(r"D:\WorkBuddy Manager\workbuddy-manager.exe")).unwrap(),
            "\"D:\\WorkBuddy Manager\\workbuddy-manager.exe\" --hidden"
        );
        for path in ["relative.exe", "D:\\bad\"path.exe", "D:\\bad\npath.exe"] {
            assert!(startup_command(Path::new(path)).is_err());
        }
    }

    #[test]
    fn startup_registration_cycle_preserves_other_values_and_obeys_system_disable() {
        let root = RegKey::predef(HKEY_CURRENT_USER);
        let path = format!(
            r"Software\WorkBuddyManager\Tests\Autostart-{}",
            uuid::Uuid::new_v4()
        );
        let (fixture, _) = root.create_subkey(&path).unwrap();
        let (run, _) = fixture.create_subkey("Run").unwrap();
        let (approved, _) = fixture.create_subkey("Approved").unwrap();
        run.set_value("fiction-other-app", &"keep").unwrap();
        let command =
            startup_command(Path::new(r"D:\fiction manager\workbuddy-manager.exe")).unwrap();
        assert!(!read_enabled(&run, &approved, &command).unwrap());
        assert!(write_enabled(&run, &approved, &command, true).unwrap());
        assert_eq!(run.get_value::<String, _>(ENTRY).unwrap(), command);
        let other_command =
            startup_command(Path::new(r"D:\fiction other\workbuddy-manager.exe")).unwrap();
        assert!(!read_enabled(&run, &approved, &other_command).unwrap());
        approved
            .set_raw_value(
                ENTRY,
                &RegValue {
                    vtype: REG_BINARY,
                    bytes: vec![3, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0],
                },
            )
            .unwrap();
        assert!(!read_enabled(&run, &approved, &command).unwrap());
        approved
            .set_raw_value(
                ENTRY,
                &RegValue {
                    vtype: REG_BINARY,
                    bytes: vec![],
                },
            )
            .unwrap();
        assert!(read_enabled(&run, &approved, &command).is_err());
        assert!(!write_enabled(&run, &approved, &command, false).unwrap());
        assert!(!write_enabled(&run, &approved, &command, false).unwrap());
        assert_eq!(
            run.get_value::<String, _>("fiction-other-app").unwrap(),
            "keep"
        );
        let readonly_approved = fixture
            .open_subkey_with_flags("Approved", KEY_READ)
            .unwrap();
        assert!(write_enabled(&run, &readonly_approved, &command, true).is_err());
        assert!(read_raw(&run).unwrap().is_none());
        assert!(read_raw(&approved).unwrap().is_none());
        drop(readonly_approved);
        drop(run);
        drop(approved);
        drop(fixture);
        assert!(path.starts_with(r"Software\WorkBuddyManager\Tests\Autostart-"));
        root.delete_subkey_all(&path).unwrap();
    }
}
