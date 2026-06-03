mod app;
mod args;

use anyhow::{Context, Result};
use args::Args;
use clap::Parser;

use app::TodoMgr;

fn main() -> Result<()> {
    let args = Args::parse();

    let path = args.path;
    let mut todomgr = TodoMgr::init(path).context("Todomgr init error")?;
    todomgr.open().context("Todo list open error")?;

    match args.command {
        args::Command::List {} => {}
        args::Command::Add { content, separator } => todomgr.add(content, separator),
        args::Command::Rewrite { id, content } => {
            todomgr.rewrite(id, content).context("Rewrite todo error")?
        }
        args::Command::Remove { ids, from, to } => {
            todomgr.remove(ids, from, to).context("Remove todo error")?
        }
        args::Command::Switch { ids, from, to } => todomgr
            .switch(ids, from, to)
            .context("Switch todo state error")?,
        args::Command::Drop { ids, from, to } => {
            todomgr.drop(ids, from, to).context("Drop todo error")?
        }
        args::Command::Clean {} => todomgr.clean(),
    }
    todomgr.print();
    todomgr.save().context("Save to file error")?;
    Ok(())
}
