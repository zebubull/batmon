use std::path::Path;

pub struct Device<'a> {
    pub path: &'a Path,
}

impl Device<'_> {
    pub fn is_battery(&self) -> bool {
        let meta = match std::fs::metadata(self.path) {
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
}

impl<'a, T> From<&'a T> for Device<'a>
where
    T: AsRef<Path>,
{
    fn from(value: &'a T) -> Self {
        Device {
            path: value.as_ref(),
        }
    }
}
