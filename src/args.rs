use std::path::PathBuf;

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(author, version, about, long_about = "Todo manager on rust")]
pub struct Args {
    /// Path to the Todo list
    #[arg(short, long)]
    pub path: Option<PathBuf>,

    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// List Todos
    #[command(alias = "ls")]
    List {},
    /// Add new Todo
    Add {
        #[arg(short, long)]
        separator: Option<char>,
        content: Vec<String>,
    },
    /// Rewrite Todo
    #[command(alias = "re")]
    Rewrite { id: u16, content: Vec<String> },
    /// Remove Todo
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
    /// Switch Todo state
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
    /// Drop Todo
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
    /// Delete Done and Dropped Todos
    Clean {},
}
