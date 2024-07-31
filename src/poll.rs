use std::{error::Error, path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]
pub struct PolledValue<T> {
    value: T,
    path: PathBuf,
}

impl<T> PolledValue<T> {
    pub fn new(initial_value: T, path: impl Into<PathBuf>) -> Self {
        let p = Self {
            value: initial_value,
            path: path.into(),
        };

        if !std::fs::metadata(&p.path).is_ok() {
            warn!(
                "Poll target {} does not exist or is not accessible",
                p.path.to_string_lossy()
            );
        }

        p
    }
}

impl<T> PolledValue<T>
where
    T: FromStr + Copy,
{
    pub fn update(&mut self) -> Result<(), Box<dyn Error>> {
        let data = std::fs::read_to_string(&self.path)?;
        self.value = data.trim().parse().map_err(|_| "failed to parse")?;
        Ok(())
    }
}

impl<T> std::ops::Deref for PolledValue<T> {
    type Target = T;
    fn deref(&self) -> &Self::Target {
        &self.value
    }
}
