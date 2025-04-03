use std::fs::File;
use std::io::{stdin, stdout, Write};
use std::path::PathBuf;
use std::process::Command;
use std::str::FromStr;
use std::time::Instant;

use clap::{ArgAction, Parser};
use log::{error, info, warn, LevelFilter};
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

    #[arg(short, long, value_delimiter = ',', default_values = ["cs", "hpp", "json", "rs"])]
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

fn clear_console() {
    if cfg!(windows) {
        let _ = Command::new("cmd").args(["/C", "cls"]).status();
    } else {
        let _ = Command::new("clear").status();
    }
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

fn pause_exit() {
    println!("\x1b[90m[>] Press Enter to exit...\x1b[0m");
    let mut _s = String::new();
    let _ = stdin().read_line(&mut _s);
}

fn interactive_file_selection() -> Vec<String> {
    let mut selected = vec!["cs".to_string(), "json".to_string()];
    let all_types = vec!["cs", "hpp", "json", "rs"];

    loop {
        clear_console();
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
        println!("║  Select which files to generate:                           ║");

        for (i, ftype) in all_types.iter().enumerate() {
            let mark = if selected.contains(&ftype.to_string()) { "[x]" } else { "[ ]" };
            println!("║  [{}] {:<6} {:<44}   ║", i + 1, ftype, mark);
        }

        println!("║                                                            ║");
        println!("║  Press number to toggle, ENTER to confirm                  ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        print!("\x1b[96m[>] Your choice: \x1b[0m");
        stdout().flush().unwrap();

        let mut input = String::new();
        stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            break;
        }

        match input.parse::<usize>() {
            Ok(index) if index >= 1 && index <= all_types.len() => {
                let value = all_types[index - 1].to_string();
                if selected.contains(&value) {
                    selected.retain(|x| x != &value);
                } else {
                    selected.push(value);
                }
            }
            _ => {
                println!("\x1b[91m[ERR ] Invalid input. Press Enter to continue...\x1b[0m");
                let _ = stdin().read_line(&mut String::new());
            }
        }
    }

    if selected.is_empty() {
        println!("\x1b[93m[WARN] No types selected. Defaulting to .cs\x1b[0m");
        vec!["cs".to_string()]
    } else {
        selected
    }
}

fn main() -> Result<()> {
    clear_console();
    let mut args = Args::parse();
    println!("\x1b[93m>> Welcome to CS2 Offset Dumper REWORKED <<\x1b[0m");
    args.file_types = interactive_file_selection();
    clear_console();

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
            #[cfg(not(windows))] {
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
