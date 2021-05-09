use std::{collections::HashMap, sync::Arc};
use std::env;
use std::io;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::{IpAddr, TcpStream};
use std::process::{exit, Command, Stdio};
use std::str;
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use std::thread;
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use clap::{App, AppSettings, Arg};
use config::LoadConfigError;
use tui::{Terminal, backend::CrosstermBackend};

mod config;
mod exploits;
mod submit;
mod templates;
mod ui;

fn main() -> Result<(), LoadConfigError> {
    let matches = App::new("enoxploit")
        .about("Streamline your exploit development!")
        // only positional arguments follow after the last positional argument
        // needed so that e.g. "enoxploit python3 -u test.py" works,
        // otherwise "-u" is treated as an argument for enoxploit
        .setting(AppSettings::TrailingVarArg)
        .arg(Arg::with_name("command")
            .multiple(true)
            .required(true)
            .help("Command to run"))
            /* // TODO
        .arg(Arg::with_name("python-force-buffered")
            .help("Do not automatically set -u for python scripts")
            .long("python-force-buffered"))
            */
        .arg(Arg::with_name("config") // TODO decide how we do bambi/enowars default config
            .help("Location of the config file. Can be a URL starting with http(s):// or a file URI starting with file:///. At the moment everything else is treated as a file path but this may change in the future.")
            .short("c")
            .long("config")
            .takes_value(true))
        .get_matches();

    let config = config::load_config(matches.value_of("config"))?;
    let command: Vec<String> = matches.values_of("command").unwrap().map(String::from).collect();
    ui::initialize()?;

    exploits::run(Arc::new(command), Arc::new(config));
    Ok(())
}
