#[macro_use]
extern crate log;

use batmon::{Battery, ChargingStatus};
use clap::Parser;
use libnotify::{Notification, Urgency};

mod cli;
use cli::{Cli, Command};

type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;

static APP_NAME: &'static str = "batmon";

fn main() {
    if std::env::var("RUST_LOG").is_err() {
        std::env::set_var(
            "RUST_LOG",
            if cfg!(debug_assertions) {
                "trace"
            } else {
                "warn"
            },
        );
    }
    pretty_env_logger::init();
    match libnotify::init(APP_NAME) {
        Ok(_) => debug!("Initialized libnotify"),
        Err(e) => {
            error!("Failed to initialize libnotify: {e}");
            std::process::exit(1);
        }
    };

    let res = run();
    libnotify::uninit();

    if let Err(ref e) = res {
        error!("Fatal error: {}", e.to_string());
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let args = Cli::parse();
    let mut bat = match args.device {
        Some(d) => {
            Battery::new(&d).map_err(|e| format!("Failed to load specified battery: {e}"))?
        }
        None => Battery::find().ok_or("Failed to detect a valid battery")?,
    };
    let s = bat.state();
    match args.command {
        Some(Command::Capacity) => println!("{}", s.capacity),
        Some(Command::Charge) => println!("{}", s.charge),
        Some(Command::Current) => println!("{}", s.current),
        Some(Command::Cycles) => println!("{}", s.cycles),
        Some(Command::Level) => println!("{}", s.level),
        Some(Command::Name) => println!("{}", bat.name),
        Some(Command::Status) => println!("{}", s.status),
        Some(Command::Time) => println!("{}", bat.remaining()),
        Some(Command::Summary) | None => println!("{bat}"),
        Some(Command::Daemon(d)) => loop {
            update_battery_and_notify(&mut bat)?;
            info!("{bat}");
            std::thread::sleep(std::time::Duration::from_secs(d.interval));
        },
    }
    Ok(())
}

struct BatteryLevelSettings {
    level: u8,
    label: &'static str,
    urgency: Urgency,
}

static LEVELS: [BatteryLevelSettings; 3] = [
    BatteryLevelSettings {
        level: 50,
        label: "at half",
        urgency: Urgency::Low,
    },
    BatteryLevelSettings {
        level: 25,
        label: "low",
        urgency: Urgency::Normal,
    },
    BatteryLevelSettings {
        level: 15,
        label: "critical",
        urgency: Urgency::Critical,
    },
];

fn update_battery_and_notify(battery: &mut Battery) -> Result<()> {
    let old_state = battery.state();
    battery.update();
    let new_state = battery.state();

    match new_state.status.edge(old_state.status) {
        Some(ChargingStatus::Discharging) => {
            info!("Battery started discharging");
            let body = format!(
                "{} is discharging\n{}",
                battery.name,
                battery.remaining_labelled()
            );
            let n = Notification::new("Discharging", Some(body.as_str()), None);
            n.set_urgency(Urgency::Normal);
            n.show()?;
        }
        Some(ChargingStatus::Charging) => {
            info!("Battery started charging");
            let body = format!(
                "{} is charging\n{}",
                battery.name,
                battery.remaining_labelled()
            );
            let n = Notification::new("Charging", Some(body.as_str()), None);
            n.set_urgency(Urgency::Low);
            n.show()?;
        }
        Some(ChargingStatus::Full) => {
            info!("Battery full");
            let body = format!("{} @ 100%", battery.name);
            let n = Notification::new("Battery full", Some(body.as_str()), None);
            n.set_urgency(Urgency::Low);
            n.show()?;
        }
        None => {}
    }

    for level in LEVELS.iter() {
        if old_state.level > level.level {
            if new_state.level <= level.level {
                info!("Battery at {}%", new_state.level);
                let title = format!("Battery {}", level.label);
                let body = format!(
                    "{} @ {}%\n{}",
                    battery.name,
                    new_state.level,
                    battery.remaining_labelled()
                );
                let n = Notification::new(title.as_str(), Some(body.as_str()), None);
                n.set_urgency(level.urgency);
                n.show()?;
            }

            break;
        }
    }

    Ok(())
}
