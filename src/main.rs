#[macro_use] extern crate log;
use clap::{Args, Parser, Subcommand};

use batmon::{Battery, BatteryStatus};
use libnotify::{Notification, Urgency};

type Result<T> = std::result::Result<T, std::boxed::Box<dyn std::error::Error>>;

static APP_NAME: &'static str = "batmon";

#[derive(Subcommand)]
enum Command {
    /// Print out the capacity, in uAh
    Capacity,
    /// Print out the current charge level, in uAh
    Charge,
    /// Print out the current draw, in uA
    Current,
    /// Print out the battery level as a percentage
    Level,
    /// Print out the name of the battery
    Name,
    /// Print out the status of the battery
    Status,
    /// Print out the time remaining until the battery is either charged or discharged
    Time,
    /// [DEFAULT] Print out a summary of the battery
    Summary,
    /// Run batmon as a battery state notification daemon
    Daemon(DaemonArgs),
}

#[derive(Args)]
struct DaemonArgs {
    /// The refresh interval when running, in seconds
    #[arg(short, long, default_value_t = 15)]
    interval: u64,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Print a specific battery parameter to standard output
    #[command(subcommand)]
    command: Option<Command>,
    
}

fn main() -> Result<()> {
    pretty_env_logger::init();
    match libnotify::init(APP_NAME) {
        Ok(_) => debug!("Initialized libnotify"),
        Err(e) => {
            error!("Failed to initialize libnotify: {e}");
            return Err(e.into());
        }
    };

    let res = run();

    if let Err(ref e) = res {
        error!("Fatal error: {}", e.to_string());
    }

    libnotify::uninit();
    Ok(res?)
}

fn run() -> Result<()> {
    let args = Cli::parse();
    let mut bat = Battery::find().ok_or("Failed to detect battery")?;
    match args.command {
        Some(Command::Capacity) => println!("{}", bat.capacity),
        Some(Command::Charge) => println!("{}", bat.charge),
        Some(Command::Current) => println!("{}", bat.current),
        Some(Command::Level) => println!("{}", bat.level),
        Some(Command::Name) => println!("{}", bat.name),
        Some(Command::Status) => println!("{}", bat.status),
        Some(Command::Time) => println!("{}", bat.remaining()),
        Some(Command::Summary) | None => println!("{bat}"),
        Some(Command::Daemon(d)) => {
            loop {
                update_battery_and_notify(&mut bat)?;
                info!("{bat}");
                std::thread::sleep(std::time::Duration::from_secs(d.interval));
            }
        }
    }
    Ok(())
}

fn update_battery_and_notify(battery: &mut Battery) -> Result<()> {
    let old_state = battery.status;
    let old_level = battery.level;
    battery.update();
    match (old_state, battery.status) {
        (BatteryStatus::Discharging, BatteryStatus::Charging) => {
            info!("Battery started charging");
            let body = format!("{} is charging\n{}", battery.name, battery.remaining_labelled());
            let n = Notification::new("Charging", Some(body.as_str()), None);
            n.set_urgency(Urgency::Low);
            n.show()?;
        },
        (BatteryStatus::Charging, BatteryStatus::Discharging) => {
            info!("Battery started discharging");
            let body = format!("{} is discharging\n{}", battery.name, battery.remaining_labelled());
            let n = Notification::new("Discharging", Some(body.as_str()), None);
            n.set_urgency(Urgency::Normal);
            n.show()?;
        },
        (BatteryStatus::Charging, BatteryStatus::Full) => {
            info!("Battery full");
            let body = format!("{} @ 100%", battery.name);
            let n = Notification::new("Battery full", Some(body.as_str()), None);
            n.set_urgency(Urgency::Low);
            n.show()?;
        }
        _ => {},
    }

    if old_level > 50 {
        if battery.level <= 50 {
            info!("Battery at 50%");
            let body = format!("{} @ 50%\n{}", battery.name, battery.remaining_labelled());
            let n = Notification::new("Battery at half", Some(body.as_str()), None);
            n.set_urgency(Urgency::Low);
            n.show()?;
        }
    } else if old_level > 25 {
        if battery.level <= 25 {
            info!("Battery at 25%");
            let body = format!("{} @ 25%\n{}", battery.name, battery.remaining_labelled());
            let n = Notification::new("Battery low", Some(body.as_str()), None);
            n.set_urgency(Urgency::Normal);
            n.show()?;
        }
    } else if old_level > 10 {
        if battery.level <= 10 {
            info!("Battery at 10%");
            let body = format!("{} @ 10%\n{}", battery.name, battery.remaining_labelled());
            let n = Notification::new("Battery critical", Some(body.as_str()), None);
            n.set_urgency(Urgency::Critical);
            n.show()?;
        }
    }

    Ok(())
}
