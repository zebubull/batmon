use crate::device::Device;
use crate::poll::PolledValue;
use crate::status::ChargingStatus;

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
        let devices = std::fs::read_dir("/sys/class/power_supply").ok()?;

        let mut devices: Vec<_> = devices
            .filter_map(|d| {
                if d.is_err() {
                    None
                } else {
                    Some(Device::from(d.unwrap().path()))
                }
            })
            .filter_map(|d| {
                if !d.is_battery() {
                    None
                } else {
                    let rating = d.rating();
                    Some((d, rating))
                }
            }).collect();

        devices.sort_by_key(|d| d.1);

        for (d, r) in devices {
            match Battery::try_from(&d) {
                Ok(bat) => {
                    debug!("found battery at device '{}' (rating {r})", bat.name);
                    return Some(bat);
                }
                Err(e) => {
                    let name = d.path.file_name().unwrap_or_default().to_string_lossy().to_string();
                    debug!(
                        "device {name} (rating {r}) is (probably) not a battery: {e}",
                    );
                }
            };
        }

        None
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
            warn!("Failed to update charge level: {e}");
        }

        if let Err(e) = self.capacity.update() {
            warn!("Failed to update capacity: {e}");
        }

        if let Err(e) = self.charge.update() {
            warn!("Failed to update charge: {e}");
        }

        if let Err(e) = self.current.update() {
            warn!("Failed to update current: {e}");
        }

        if let Err(e) = self.status.update() {
            warn!("Failed to update status: {e}");
        }

        if let Err(e) = self.cycles.update() {
            warn!("Failed to update cycles: {e}");
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
        if !device.is_battery() {
            return Err("Invalid type".into());
        }
    
        let name = device.path.file_name().unwrap_or_default().to_string_lossy().to_string();
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
