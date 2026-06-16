use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use network_interface::{NetworkInterface, NetworkInterfaceConfig};
use rand_core::OsRng;
use serde::Serialize;
use std::{fs, path::PathBuf};

#[derive(Serialize)]
struct BatteryInfo {
    percent: f32,
    charging: bool,
    state: String,
}

#[derive(Serialize)]
struct NetworkInfo {
    online: bool,
    kind: String,
    name: String,
}

#[derive(Serialize)]
struct PasswordStatus {
    has_password: bool,
}

#[derive(Serialize)]
struct LoginResult {
    ok: bool,
}

fn password_hash_path() -> Result<PathBuf, String> {
    let data_dir = dirs::data_dir().ok_or("Cannot find user data directory")?;
    Ok(data_dir.join("win12-desktop").join("password.hash"))
}

#[tauri::command]
fn get_login_password_status() -> Result<PasswordStatus, String> {
    Ok(PasswordStatus {
        has_password: password_hash_path()?.exists(),
    })
}

#[tauri::command]
fn verify_login_password(password: String) -> Result<LoginResult, String> {
    if password.is_empty() {
        return Err("Password cannot be empty".to_string());
    }

    let path = password_hash_path()?;

    if !path.exists() {
        return Ok(LoginResult { ok: true });
    }

    let hash = fs::read_to_string(&path).map_err(|e| e.to_string())?;
    let parsed_hash = PasswordHash::new(hash.trim()).map_err(|e| e.to_string())?;
    let ok = Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();

    Ok(LoginResult { ok })
}

#[tauri::command]
fn set_login_password(
    current_password: Option<String>,
    new_password: String,
) -> Result<(), String> {
    let path = password_hash_path()?;
    let has_password = path.exists();

    if has_password {
        let current_password =
            current_password.ok_or("Current password is required".to_string())?;
        if !verify_login_password(current_password)?.ok {
            return Err("Current password is incorrect".to_string());
        }
    }

    if new_password.is_empty() {
        if has_password {
            fs::remove_file(&path).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(new_password.as_bytes(), &salt)
        .map_err(|e| e.to_string())?
        .to_string();

    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }

    fs::write(&path, password_hash).map_err(|e| e.to_string())?;

    Ok(())
}

#[tauri::command]
fn get_battery_info() -> Result<BatteryInfo, String> {
    let manager = battery::Manager::new().map_err(|e| e.to_string())?;

    let mut batteries = manager.batteries().map_err(|e| e.to_string())?;

    let battery = batteries
        .next()
        .ok_or("No battery found")?
        .map_err(|e| e.to_string())?;

    let percent = battery
        .state_of_charge()
        .get::<battery::units::ratio::percent>();

    let state = format!("{:?}", battery.state());

    let charging = matches!(
        battery.state(),
        battery::State::Charging | battery::State::Full
    );

    Ok(BatteryInfo {
        percent,
        charging,
        state,
    })
}

#[tauri::command]
fn get_network_info() -> Result<NetworkInfo, String> {
    let interfaces = NetworkInterface::show().map_err(|e| e.to_string())?;

    for interface in interfaces {
        let name = interface.name.clone();

        // 跳过本机回环接口
        if name == "lo" || name.starts_with("lo") {
            continue;
        }

        // 没有 IP 地址的接口通常不是正在使用的网络
        if interface.addr.is_empty() {
            continue;
        }

        let kind = if name.starts_with("wl")
            || name.starts_with("wlan")
            || name.starts_with("wifi")
            || name.starts_with("wlp")
        {
            "wifi"
        } else if name.starts_with("en")
            || name.starts_with("eth")
            || name.starts_with("eno")
            || name.starts_with("ens")
            || name.starts_with("enp")
        {
            "ethernet"
        } else {
            "unknown"
        };

        return Ok(NetworkInfo {
            online: true,
            kind: kind.to_string(),
            name,
        });
    }

    Ok(NetworkInfo {
        online: false,
        kind: "offline".to_string(),
        name: String::new(),
    })
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            get_battery_info,
            get_network_info,
            get_login_password_status,
            verify_login_password,
            set_login_password
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
