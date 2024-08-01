use std::path::PathBuf;

pub struct Device {
    pub path: PathBuf,
}

impl Device {
    pub fn is_system_battery(&self) -> bool {
        let meta = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        if !meta.is_dir() {
            return false;
        }

        // Type should contain "Battery" if the device is a battery
        let type_file = self.path.join("type");

        let ty = match std::fs::read_to_string(type_file) {
            Ok(res) => res,
            Err(_) => return false,
        };

        if ty.trim() != "Battery" {
            debug!(
                "Device '{}' ('{}') rejected",
                self.path.file_name().unwrap_or_default().to_string_lossy(),
                ty.trim(),
            );
            return false;
        }

        // Scope may or may not exist.
        // It can be ignored if not present, but it should contain "System" if it exists.
        let scope_file = self.path.join("scope");

        match std::fs::read_to_string(scope_file) {
            Ok(s) => {
                debug!(
                    "Match '{}' ({})",
                    self.path.file_name().unwrap_or_default().to_string_lossy(),
                    s.trim()
                );
                s.trim() == "System"
            }
            Err(_) => true,
        }
    }

    fn has_file_available(&self, file: &str) -> bool {
        std::fs::metadata(self.path.join(file)).is_ok()
    }

    pub fn rating(&self) -> u8 {
        [
            self.has_file_available("current_now"),
            self.has_file_available("capacity"),
            self.has_file_available("charge_full"),
            self.has_file_available("charge_now"),
            self.has_file_available("cycle_count"),
            self.has_file_available("status"),
        ]
        .into_iter()
        .filter(|b| *b)
        .count() as u8
    }
}

impl From<PathBuf> for Device {
    fn from(value: PathBuf) -> Self {
        Device { path: value }
    }
}
