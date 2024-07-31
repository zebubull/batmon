# batmon
batmon is a simple battery monitoring tool for linux that can report various battery statistics and provide battery related notifications.

## Disclaimer
This program was written and tested only on my personal laptop. I cannot guarantee that it will work on every laptop, but I'm fairly certain it should.
Feel free to open an issue if it does not work on your machine or, if you are able to, fix the bug and open a pull request.
I'm not very experienced in rust so the codebase may or may not be a complete mess. View at your own risk.

## Installation
1. Clone the repo
2. Run `cargo build --release`
3. Run `target/release/batmon` or make it available in your path and run `batmon`

## Usage
Run `batmon --help` to view the program help.

## Copyright
Copyright (c) 2024 zebubull. All Rights Reserved.
