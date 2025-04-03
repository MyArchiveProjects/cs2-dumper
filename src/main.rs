use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::io::{stdin, stdout, Write};

use clap::{Parser, ArgAction};
use log::{info, warn, error, LevelFilter};
use simplelog::*;
use anyhow::{Result, bail};
use memflow::prelude::v1::*;
use memflow_native;
use output::Output;

mod analysis;
mod output;
mod source2;

#[derive(Debug, Parser)]
#[command(author = "MyArchiveProjects", version = "0.1.0-rework", about = "CS2 Offset Dumper with Memflow reworked")]
struct Args {
    #[arg(short, long)]
    connector: Option<String>,

    #[arg(short = 'a', long)]
    connector_args: Option<String>,

    #[arg(short, long, value_delimiter = ',', default_values = ["cs"])]
    file_types: Vec<String>,

    #[arg(short, long, default_value_t = 4)]
    indent_size: usize,

    #[arg(short, long, default_value = "output")]
    output: PathBuf,

    #[arg(short, long, default_value = "cs2.exe")]
    process_name: String,

    #[arg(short, long, action = ArgAction::Count)]
    verbose: u8,

    #[arg(short, long)]
    no_log_file: bool,
}

fn print_banner() {
    println!("\x1b[93m");
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║    ██████╗ ██╗   ██╗███╗   ███╗██████╗ ███████╗██████╗     ║");
    println!("║    ██╔══██╗██║   ██║████╗ ████║██╔══██╗██╔════╝██╔══██╗    ║");
    println!("║    ██║  ██║██║   ██║██╔████╔██║██████╔╝█████╗  ██████╔╝    ║");
    println!("║    ██║  ██║██║   ██║██║╚██╔╝██║██╔═══╝ ██╔══╝  ██╔══██╗    ║");
    println!("║    ██████╔╝╚██████╔╝██║ ╚═╝ ██║██║     ███████╗██║  ██║    ║");
    println!("║    ╚═════╝  ╚═════╝ ╚═╝     ╚═╝╚═╝     ╚══════╝╚═╝  ╚═╝    ║");
    println!("║                                                            ║");
    println!("║      Counter-Strike 2 Offset Dumper — REWORKED EDITION     ║");
    println!("║                                                            ║");
    println!("║  github: github.com/MyArchiveProjects                      ║");
    println!("║  original: github.com/a2x/cs2-dumper                       ║");
    println!("╠════════════════════════════════════════════════════════════╣");
    println!("║  [1] Dump .cs only                                         ║");
    println!("║  [2] Dump .cs + .json                                      ║");
    println!("║  [3] Info / About                                          ║");
    println!("║  [4] Exit                                                  ║");
    println!("╚════════════════════════════════════════════════════════════╝");
    println!("\x1b[0m");
}

fn print_prefix_info(msg: &str) {
    println!("\x1b[92m[ OK ]\x1b[0m {}", msg);
}

fn print_prefix_warn(msg: &str) {
    println!("\x1b[93m[WARN]\x1b[0m {}", msg);
}

fn print_prefix_err(msg: &str) {
    println!("\x1b[91m[ERR ]\x1b[0m {}", msg);
}

fn print_info_page() {
    println!("\x1b[94m--- About This Tool ---\x1b[0m");
    println!("This is a reworked fork of cs2-dumper for dumping offsets from Counter-Strike 2.");
    println!("Supports memflow and has selectable output formats (.cs only or .cs + .json).");
    println!("GitHub (fork): https://github.com/MyArchiveProjects");
    println!("GitHub (original): https://github.com/a2x/cs2-dumper");
}

fn pause_exit() {
    println!("\x1b[90m[>] Press Enter to exit...\x1b[0m");
    let mut dummy = String::new();
    let _ = stdin().read_line(&mut dummy);
}

fn main() -> Result<()> {
    let mut args = Args::parse();

    let level_filter = match args.verbose {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    };

    let mut loggers: Vec<Box<dyn SharedLogger>> = vec![TermLogger::new(
        level_filter,
        Config::default(),
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )];

    if !args.no_log_file {
        loggers.push(WriteLogger::new(
            LevelFilter::Info,
            Config::default(),
            File::create("cs2-dumper.log")?,
        ));
    }

    CombinedLogger::init(loggers)?;

    if args.file_types == vec!["cs"] {
        print_banner();
        print!("\x1b[96m[>]\x1b[0m Your choice: ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "2" => args.file_types = vec!["cs".to_string(), "json".to_string()],
            "3" => {
                print_info_page();
                pause_exit();
                return Ok(());
            },
            "4" => {
                print_prefix_info("Exiting...");
                pause_exit();
                return Ok(());
            },
            _ => args.file_types = vec!["cs".to_string()],
        }
    }

    let conn_args = args
        .connector_args
        .map(|s| ConnectorArgs::from_str(&s).unwrap_or_default())
        .unwrap_or_default();

    let mut os = match args.connector {
        Some(conn) => {
            let inventory = Inventory::scan();
            inventory
                .builder()
                .connector(&conn)
                .args(conn_args)
                .os("win32")
                .build()?
        }
        None => {
            #[cfg(windows)]
            {
                memflow_native::create_os(&OsArgs::default(), LibArc::default())?
            }
            #[cfg(not(windows))]
            {
                bail!("No connector specified and not running on Windows.")
            }
        }
    };

    let mut process = os.process_by_name(&args.process_name)?;

    let now = Instant::now();

    let result = analysis::analyze_all(&mut process)?;
    let output = Output::new(&args.file_types, args.indent_size, &args.output, &result)?;

    output.dump_all(&mut process)?;

    print_prefix_info(&format!("Analysis completed in {:.2?}", now.elapsed()));
    pause_exit();
    Ok(())
}
