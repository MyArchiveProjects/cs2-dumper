use std::fs::File;
use std::path::PathBuf;
use std::str::FromStr;
use std::time::Instant;
use std::io::{stdin, stdout, Write};

use clap::{Parser, ArgAction};
use log::{info, LevelFilter};
use simplelog::*;
use anyhow::{Result, bail};
use memflow::prelude::v1::*;
use memflow_native;
use output::Output;

mod analysis;
mod output;
mod source2;

#[derive(Debug, Parser)]
#[command(author = "VacBan Team", version = "2.0", about = "CS2 Offset Dumper with Memflow")]
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
        println!("\n[?] Select output format:");
        println!("[1] cs only");
        println!("[2] cs + json");
        print!("> Enter 1 or 2 and press Enter: ");
        stdout().flush()?;

        let mut input = String::new();
        stdin().read_line(&mut input)?;
        let input = input.trim();

        args.file_types = match input {
            "2" => vec!["cs".to_string(), "json".to_string()],
            _ => vec!["cs".to_string()],
        };
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

    info!("Analysis completed in {:.2?}", now.elapsed());

    Ok(())
}
