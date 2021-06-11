extern crate log;

use anyhow::{Context, Result};
use log::{debug, info};
use sha1::{Digest, Sha1};
use std::path::PathBuf;
use std::process::{Command, ExitStatus};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
  name = "tmux-lazy-session",
  about = "A simple app for lazy tmux users."
)]
struct Cli {
  /// Kill session
  #[structopt(short, long)]
  kill: bool,

  /// Session prefix
  #[structopt(short, long, default_value = "tls")]
  prefix: String,

  /// Custom session name. Will be used literally and therefore override prefix.
  #[structopt(short, long)]
  name: Option<String>,

  /// Verbose logging
  #[structopt(short, long)]
  verbose: bool,
}

fn main() -> Result<()> {
  let args = Cli::from_args();
  env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

  let pwd = std::env::current_dir().with_context(|| "Error getting pwd")?;
  let pwd_str = pwd_as_string(pwd)?;
  debug!("pwd: `{}`", pwd_str);

  let hash_str = short_hash(&pwd_str);
  debug!("hash: `{}`", hash_str);

  let session_name = match args.name {
    Some(name) => name,
    None => args.prefix + &hash_str,
  };

  debug!("session name: `{}`", session_name);

  if args.kill {
    kill_session(&session_name)?;
  } else {
    let res = attach_session(&session_name)?;
    if !res.success() {
      new_session(&session_name)?;
    }
  };

  Ok(())
}

fn pwd_as_string(pwd: PathBuf) -> Result<String> {
  return pwd
    .to_str()
    .map(|x| x.to_string())
    .with_context(|| "Error converting pathbuf to string");
}

fn short_hash(string: &String) -> String {
  let hash = Sha1::digest(&string.as_bytes());
  return format!("{:X}", hash).split_at(10).0.to_string();
}

fn kill_session(session_name: &String) -> Result<ExitStatus> {
  info!("Killing session with name `{}`...", session_name);
  return run_tmux(&["kill-session", "-t", session_name])
    .with_context(|| format!("could create session `{}`", session_name));
}

fn attach_session(session_name: &String) -> Result<ExitStatus> {
  info!("Attaching to session with name `{}`...", session_name);
  return run_tmux(&["attach-session", "-d", "-t", session_name])
    .with_context(|| format!("could not attach to session `{}`", session_name));
}

fn new_session(session_name: &String) -> Result<ExitStatus> {
  info!("Creating session with name `{}`...", session_name);
  return run_tmux(&["new-session", "-s", session_name])
    .with_context(|| format!("could create session `{}`", session_name));
}

fn run_tmux(args: &[&str]) -> Result<ExitStatus, std::io::Error> {
  return Command::new("tmux").args(args).spawn()?.wait();
}
