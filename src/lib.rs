use std::{error::Error, fs::DirEntry, path::PathBuf};

use strum::Display;

#[macro_use] extern crate log;

trait DirEntryUtil {
    fn is_battery_device(&self) -> bool;
}

impl DirEntryUtil for DirEntry {
    fn is_battery_device(&self) -> bool {
        let meta = match std::fs::metadata(self.path()) {
            Ok(m) => m,
            Err(_) => return false,
        };

        if !meta.is_dir() {
            return false;
        }

        let mut type_path = self.path();
        type_path.push("type");

        let ty = match std::fs::read_to_string(type_path) {
            Ok(res) => res,
            Err(_) => return false,
        };
        ty.trim() == "Battery"
    }
}

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
pub enum BatteryStatus {
    Charging,
    Discharging,
    Full,
}

#[derive(Debug)]
pub struct Battery {
    path: PathBuf,
    pub name: String,
    pub level: u8,
    pub capacity: u64,
    pub charge: u64,
    pub current: u64,
    pub status: BatteryStatus,
}

impl Battery {
    pub fn find() -> Option<Self> {
        let devices = std::fs::read_dir("/sys/class/power_supply").ok()?;

        for d in devices {
            let d = match d {
                Ok(dir) => dir,
                Err(_) => continue,
            };

            match Battery::try_from(&d) {
                Ok(bat) => {
                    info!("found battery at device {}", bat.name);
                    return Some(bat);
                }
                Err(e) => {
                    debug!("device {} is (probably) not a battery: {e}", d.file_name().to_string_lossy());
                },
            };
        }

        None
    }

    pub fn update(&mut self) {
        if let Ok(level) = self.read_level() {
            self.level = level;
        }
        if let Ok(cap) = self.read_capacity() {
            self.capacity = cap;
        }
        if let Ok(charge) = self.read_charge() {
            self.charge = charge;
        }
        if let Ok(status) = self.read_status() {
            self.status = status;
        }
        if let Ok(current) = self.read_current() {
            self.current = current;
        }
    }

    pub fn remaining(&self) -> String {
        let remaining_seconds = match self.status {
            BatteryStatus::Full => return String::from("Full"),
            BatteryStatus::Discharging => self.charge * 60 * 60 / self.current,
            BatteryStatus::Charging => (self.capacity - self.charge) * 60 * 60 / self.current
        };

        let s = remaining_seconds % 60;
        let m = (remaining_seconds / 60) % 60;
        let h = remaining_seconds / 60 / 60;

        format!("{h:0>2}:{m:0>2}:{s:0>2}")
    }

    pub fn remaining_labelled(&self) -> String {
        let label = match self.status {
            BatteryStatus::Full => return String::from("Full"),
            BatteryStatus::Charging => "until full",
            BatteryStatus::Discharging => "remaining",
        };
        format!("{} {}", self.remaining(), label)
    }

    fn read_capacity(&self) -> Result<u64, Box<dyn Error>> {
        let mut path = self.path.clone();
        path.push("charge_full");
        let data = std::fs::read_to_string(path)?;

        match data.trim().parse() {
            Ok(d) => Ok(d),
            Err(e) => {
                warn!("Failed to parse capacity '{}': {}", data.trim(), e.to_string());
                Err("Failed to parse".into())
            }
        }
    }

    fn read_charge(&self) -> Result<u64, Box<dyn Error>> {
        let mut path = self.path.clone();
        path.push("charge_now");
        let data = std::fs::read_to_string(path)?;

        match data.trim().parse() {
            Ok(d) => Ok(d),
            Err(e) => {
                warn!("Failed to parse charge '{}': {}", data.trim(), e.to_string());
                Err("Failed to parse".into())
            }
        }
    }

    fn read_level(&self) -> Result<u8, Box<dyn Error>> {
        let mut path = self.path.clone();
        path.push("capacity");
        let data = std::fs::read_to_string(path)?;

        match data.trim().parse() {
            Ok(d) => Ok(d),
            Err(e) => {
                warn!("Failed to parse level '{}': {}", data.trim(), e.to_string());
                Err("Failed to parse".into())
            }
        }
    }

    fn read_status(&self) -> Result<BatteryStatus, Box<dyn Error>> {
        use BatteryStatus::*;

        if self.level == 100 {
            return Ok(Full);
        }

        let mut path = self.path.clone();
        path.push("status");
        let data = std::fs::read_to_string(path)?;

        match data.trim() {
            "Discharging" => Ok(Discharging),
            "Charging" => Ok(Charging),
            "Full" => Ok(Full),
            _ => {
                warn!("Unrecognized battery state '{}'", data.trim());
                Err("Unknown battery state".into())
            }
        }
    }

    fn read_current(&self) -> Result<u64, Box<dyn Error>> {
        let mut path = self.path.clone();
        path.push("current_now");
        let data = std::fs::read_to_string(path)?;

        match data.trim().parse() {
            Ok(d) => Ok(d),
            Err(e) => {
                warn!("Failed to parse current '{}': {}", data.trim(), e.to_string());
                Err("Failed to parse".into())
            }
        }
    }
}

impl std::fmt::Display for Battery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} @ {}%, {}, {}", self.name, self.level, self.status, self.remaining_labelled())
    }
}

impl TryFrom<&DirEntry> for Battery {
    type Error = Box<dyn Error>;
    fn try_from(value: &DirEntry) -> Result<Self, Self::Error> {
        if !value.is_battery_device() {
            return Err("not a battery".into());
        }

        let mut bat = Battery {
            name: value.file_name().to_string_lossy().to_string(),
            path: value.path(),
            level: 90,
            capacity: 0,
            charge: 0,
            current: 0,
            status: BatteryStatus::Full
        };

        bat.update();
        Ok(bat)
    }
}
