use std::path::PathBuf;

pub struct Device {
    pub path: PathBuf,
}

impl Device {
    pub fn is_battery(&self) -> bool {
        let meta = match std::fs::metadata(&self.path) {
            Ok(m) => m,
            Err(_) => return false,
        };

        if !meta.is_dir() {
            return false;
        }

        let type_file = self.path.join("type");

        let ty = match std::fs::read_to_string(type_file) {
            Ok(res) => res,
            Err(_) => return false,
        };

        debug!(
            "Found device '{}' of type '{}'",
            self.path.file_name().unwrap_or_default().to_string_lossy(),
            ty.trim(),
        );

        ty.trim() == "Battery"
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
        ].into_iter().filter(|b| *b).count() as u8
    }
}

impl From<PathBuf> for Device
{
    fn from(value: PathBuf) -> Self {
        Device {
            path: value,
        }
    }
}
