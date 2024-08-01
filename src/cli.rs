use clap::{Args, Parser, Subcommand};
#[derive(Subcommand)]
pub enum Command {
    /// Print out the capacity, in uAh
    Capacity,
    /// Print out the current charge level, in uAh
    Charge,
    /// Print out the current draw, in uA
    Current,
    /// Print out the number of charge cycles
    Cycles,
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
pub struct DaemonArgs {
    /// The refresh interval when running, in seconds
    #[arg(short, long, default_value_t = 15)]
    pub interval: u64,
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Print a specific battery parameter to standard output
    #[command(subcommand)]
    pub command: Option<Command>,
    
    /// Use a specific device instead of trying to detect the system battery
    #[arg(short, long)]
    pub device: Option<String>,
}
