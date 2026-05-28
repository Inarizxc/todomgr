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
        args::Command::Add { content } => todomgr.add(content),
        args::Command::Rewrite { id, content } => {
            todomgr.rewrite(id, content).context("Rewrite todo error")?
        }
        args::Command::Remove { ids } => todomgr.remove(ids).context("Remove todo error")?,
        args::Command::Switch { ids } => todomgr.switch(ids).context("Switch todo state error")?,
        args::Command::Drop { ids } => todomgr.drop(ids).context("Drop todo error")?,
    }
    todomgr.print();
    todomgr.save().context("Save to file error")?;
    Ok(())
}
