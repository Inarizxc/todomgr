use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = "Todo manager on rust")]
pub struct Args {
    /// Path to the Todo list
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    /// Print todos list without nerd fonts
    #[arg(short, long)]
    pub no_nerd_fonts: bool,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Print todos list
    #[command(alias = "ls")]
    List {},
    /// Add new todo
    Add {
        #[arg(short, long)]
        separator: Option<char>,
        content: Vec<String>,
    },
    /// Rewrite todo
    #[command(alias = "re")]
    Rewrite { id: u16, content: Vec<String> },
    /// Remove todo
    #[command(alias = "rm")]
    Remove {
        /// Start of ids range
        #[arg(long, short)]
        from: Option<u16>,
        /// End of ids range
        #[arg(long, short)]
        to: Option<u16>,

        ids: Option<Vec<u16>>,
    },
    /// Switch todo state
    #[command(alias = "sw")]
    Switch {
        /// Start of ids range
        #[arg(long, short)]
        from: Option<u16>,
        /// End of ids range
        #[arg(long, short)]
        to: Option<u16>,

        ids: Option<Vec<u16>>,
    },
    /// Drop todo
    #[command(alias = "d")]
    Drop {
        /// Start of ids range
        #[arg(long, short)]
        from: Option<u16>,
        /// End of ids range
        #[arg(long, short)]
        to: Option<u16>,

        ids: Option<Vec<u16>>,
    },
    /// Delete Done and Dropped todos
    Clean {},
}
