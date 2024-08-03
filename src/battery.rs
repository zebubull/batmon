use crate::device::Device;
use crate::poll::PolledValue;
use crate::status::ChargingStatus;
use std::str::FromStr;

#[derive(Debug)]
pub struct Battery {
    pub name: String,
    level: PolledValue<u8>,
    capacity: PolledValue<u64>,
    charge: PolledValue<u64>,
    current: PolledValue<u64>,
    cycles: PolledValue<u64>,
    status: PolledValue<ChargingStatus>,
}

#[derive(Debug, Clone)]
pub struct BatteryState {
    pub level: u8,
    pub capacity: u64,
    pub charge: u64,
    pub current: u64,
    pub cycles: u64,
    pub status: ChargingStatus,
}

impl Battery {
    pub fn find() -> Option<Self> {
        if std::fs::metadata("/tmp/batmon-battery").is_ok() {
            debug!("Using cached battery");
            if let Ok(bat) = Battery::load_cached_battery() {
                return Some(bat);
            }
            debug!("Failed to create battery from cache, falling back to autodetect")
        }

        let devices = std::fs::read_dir("/sys/class/power_supply").ok()?;

        let mut devices: Vec<_> = devices
            .filter_map(|d| d.ok().map(|d| Device::from(d.path())))
            .filter_map(|d| {
                if !d.is_system_battery() {
                    debug!(
                        "Device '{}' is (probably) not a battery",
                        d.path.file_name().unwrap_or_default().to_string_lossy()
                    );
                    None
                } else {
                    let rating = d.rating();
                    Some((d, rating))
                }
            })
            .collect();

        devices.sort_by_key(|d| d.1);

        for (d, r) in devices {
            match Battery::try_from(&d) {
                Ok(bat) => {
                    debug!("found battery at device '{}' (rating {r})", bat.name);
                    if r < 5 {
                        warn!(
                            "device '{}' may be missing some features (expected 5, got {r})",
                            bat.name
                        );
                    }
                    let _ = std::fs::write("/tmp/batmon-battery", &bat.name);
                    return Some(bat);
                }
                Err(e) => {
                    let name = d
                        .path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    debug!("device {name} (rating {r}) failed to init: {e}",);
                }
            };
        }

        None
    }

    pub fn new(name: &str) -> Result<Battery, Box<dyn std::error::Error>> {
        let mut path = std::path::PathBuf::from_str("/sys/class/power_supply")?;
        path.push(name.trim());

        let device = Device::from(path);
        let rating = device.rating();

        let b = Battery::try_from(&device)?;

        if rating < 5 {
            warn!(
                "Cached device '{}' may be missing features (expected 5, got {rating})",
                device
                    .path
                    .file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
            );
        }

        Ok(b)
    }

    fn load_cached_battery() -> Result<Battery, Box<dyn std::error::Error>> {
        let bat = std::fs::read_to_string("/tmp/batmon-battery")?;
        Battery::new(&bat)
    }

    pub fn state(&self) -> BatteryState {
        BatteryState {
            level: *self.level,
            capacity: *self.capacity,
            charge: *self.charge,
            current: *self.current,
            cycles: *self.cycles,
            status: *self.status,
        }
    }

    pub fn update(&mut self) {
        if let Err(e) = self.level.update() {
            debug!("Failed to update charge level: {e}");
        }

        if let Err(e) = self.capacity.update() {
            debug!("Failed to update capacity: {e}");
        }

        if let Err(e) = self.charge.update() {
            debug!("Failed to update charge: {e}");
        }

        if let Err(e) = self.current.update() {
            debug!("Failed to update current: {e}");
        }

        if let Err(e) = self.status.update() {
            debug!("Failed to update status: {e}");
        }

        if let Err(e) = self.cycles.update() {
            debug!("Failed to update cycles: {e}");
        }
    }

    pub fn remaining(&self) -> String {
        let charge = *self.charge;
        let capacity = *self.capacity;
        let current = *self.current;
        let total_seconds = match *self.status {
            ChargingStatus::Full => return String::from("Full"),
            ChargingStatus::Discharging => charge * 60 * 60 / current,
            ChargingStatus::Charging => (capacity - charge) * 60 * 60 / current,
        };

        let s = total_seconds % 60;
        let m = (total_seconds / 60) % 60;
        let h = total_seconds / 60 / 60;

        format!("{h:0>2}:{m:0>2}:{s:0>2}")
    }

    pub fn remaining_labelled(&self) -> String {
        let label = match *self.status {
            ChargingStatus::Full => return String::from("Full"),
            ChargingStatus::Charging => "until full",
            ChargingStatus::Discharging => "remaining",
        };
        format!("{} {}", self.remaining(), label)
    }
}

impl std::fmt::Display for Battery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} ({}) @ {}%, {}, {}",
            self.name,
            *self.cycles,
            *self.level,
            *self.status,
            self.remaining_labelled()
        )
    }
}

impl TryFrom<&Device> for Battery {
    type Error = Box<dyn std::error::Error>;
    fn try_from(device: &Device) -> Result<Self, Self::Error> {
        if std::fs::metadata(&device.path).is_err() {
            return Err("Device does not exist".into());
        }

        if !device.is_system_battery() {
            return Err("Device is not a system battery".into());
        }

        let name = device
            .path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let mut bat = Battery {
            name,
            level: PolledValue::new(100, device.path.join("capacity")),
            capacity: PolledValue::new(0, device.path.join("charge_full")),
            charge: PolledValue::new(0, device.path.join("charge_now")),
            current: PolledValue::new(0, device.path.join("current_now")),
            cycles: PolledValue::new(0, device.path.join("cycle_count")),
            status: PolledValue::new(ChargingStatus::Full, device.path.join("status")),
        };

        bat.update();
        Ok(bat)
    }
}
