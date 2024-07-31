use strum::Display;

#[derive(Debug, Clone, Copy, Display, PartialEq, Eq)]
pub enum ChargingStatus {
    Charging,
    Discharging,
    Full,
}

impl ChargingStatus {
    pub fn edge(self, other: Self) -> Option<Self> {
        if self != other {
            Some(self)
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct StatusParseError;

impl std::str::FromStr for ChargingStatus {
    type Err = StatusParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Charging" => Ok(Self::Charging),
            "Discharging" => Ok(Self::Discharging),
            "Full" => Ok(Self::Full),
            _ => Err(StatusParseError),
        }
    }
}
