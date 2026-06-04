use anyhow::{Context, Result, anyhow, bail};
use colored::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{BufReader, BufWriter, Write},
    path::PathBuf,
};

pub trait SymbolSet {
    fn format(id: usize, todo: &Todo) -> ColoredString;
}

pub struct NerdFont;
pub struct Ascii;

impl SymbolSet for NerdFont {
    fn format(id: usize, todo: &Todo) -> ColoredString {
        let (icon, text) = match todo.state {
            State::Todo => (
                nerd_font_symbols::cod::COD_CIRCLE_LARGE.normal(),
                todo.content.normal(),
            ),
            State::Doing => (
                nerd_font_symbols::md::MD_HELP_CIRCLE.yellow(),
                todo.content.yellow(),
            ),
            State::Done => (
                nerd_font_symbols::md::MD_CHECK_CIRCLE.green(),
                todo.content.green(),
            ),
            State::Dropped => (
                nerd_font_symbols::md::MD_CLOSE_CIRCLE.red(),
                todo.content.strikethrough().red(),
            ),
        };
        format!("{}) {}  {}", id + 1, icon, text).normal()
    }
}

impl SymbolSet for Ascii {
    fn format(id: usize, todo: &Todo) -> ColoredString {
        let text = match todo.state {
            State::Todo => format!("[ ]  {}", todo.content).normal(),
            State::Doing => format!("[/]  {}", todo.content).yellow(),
            State::Done => format!("[x]  {}", todo.content).green(),
            State::Dropped => format!("[-]  {}", todo.content).strikethrough().red(),
        };
        format!("{}) {}", id + 1, text).normal()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum State {
    Todo,
    Doing,
    Done,
    Dropped,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Todo {
    state: State,
    content: String,
}

pub struct TodoMgr {
    list: Vec<Todo>,
    file_path: PathBuf,
}

impl TodoMgr {
    pub fn init(path: Option<PathBuf>) -> Result<TodoMgr> {
        match path {
            Some(mut file_path) => {
                if file_path.is_dir() {
                    file_path.push(".todo.json");
                } else if fs::exists(&file_path).context("File existence check error")?
                    && !file_path.is_file()
                {
                    bail!("Path is not a dir or file")
                }
                Ok(TodoMgr {
                    list: vec![],
                    file_path,
                })
            }
            None => Ok(TodoMgr {
                list: vec![],
                file_path: PathBuf::from(".todo.json"),
            }),
        }
    }
    pub fn open(&mut self) -> Result<()> {
        let file = match File::open(&self.file_path) {
            Ok(f) => f,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                self.list.clear();
                return Ok(());
            }
            Err(e) => return Err(e).context("File open error"),
        };
        let reader = BufReader::new(file);
        match serde_json::from_reader(reader) {
            Ok(list) => self.list = list,
            Err(e) if e.is_eof() => self.list.clear(),
            Err(e) => bail!("Convert from JSON error: {e}"),
        };
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let file = File::create(&self.file_path).context("File create error")?;
        let mut writer = BufWriter::new(file);
        serde_json::to_writer_pretty(&mut writer, &self.list).context("Convert to JSON error")?;
        writer.flush().context("Flush error")?;
        Ok(())
    }

    pub fn print<S: SymbolSet>(&self) {
        for (id, todo) in self.list.iter().enumerate() {
            println!("{}", S::format(id, todo));
        }
    }

    pub fn add(&mut self, vec_content: Vec<String>, sep: Option<char>) {
        match sep {
            Some(s) => {
                for item in vec_content {
                    for part in item.split(s) {
                        let trimmed = part.trim();
                        if !trimmed.is_empty() {
                            self.list.push(Todo {
                                state: State::Todo,
                                content: trimmed.to_string(),
                            });
                        }
                    }
                }
            }
            None => {
                let content = vec_content.join(" ");
                if !content.is_empty() {
                    self.list.push(Todo {
                        state: State::Todo,
                        content,
                    });
                }
            }
        }
    }

    pub fn rewrite(&mut self, id: usize, vec_content: Vec<String>) -> Result<()> {
        let content = vec_content.join(" ");
        self.validate_id(id).context("Validate ID error")?;
        let todo = self.get_todo(id).context("Get todo error")?;
        todo.content = content;
        Ok(())
    }

    pub fn remove(
        &mut self,
        ids: Option<Vec<usize>>,
        from: Option<usize>,
        to: Option<usize>,
    ) -> Result<()> {
        let todo_set = self
            .collect_ids(ids, from, to)
            .context("Collection todos error")?;
        for &id in &todo_set {
            self.validate_id(id)?;
        }
        for id in todo_set.iter().rev() {
            self.list.remove(id - 1);
        }

        Ok(())
    }

    pub fn switch(
        &mut self,
        ids: Option<Vec<usize>>,
        from: Option<usize>,
        to: Option<usize>,
    ) -> Result<()> {
        let todo_set = self
            .collect_ids(ids, from, to)
            .context("Collection todos error")?;
        for &id in &todo_set {
            self.validate_id(id)?;
        }
        for id in todo_set {
            let todo = self.get_todo(id).context("Get todo error")?;
            match todo.state {
                State::Todo => todo.state = State::Doing,
                State::Doing => todo.state = State::Done,
                State::Done => todo.state = State::Todo,
                State::Dropped => todo.state = State::Todo,
            }
        }
        Ok(())
    }

    pub fn drop(
        &mut self,
        ids: Option<Vec<usize>>,
        from: Option<usize>,
        to: Option<usize>,
    ) -> Result<()> {
        let todo_set = self
            .collect_ids(ids, from, to)
            .context("Collection todos error")?;
        for &id in &todo_set {
            self.validate_id(id)?;
        }
        for id in todo_set {
            let todo = self.get_todo(id).context("Get todo error")?;
            match todo.state {
                State::Dropped => todo.state = State::Todo,
                _ => todo.state = State::Dropped,
            }
        }
        Ok(())
    }

    pub fn clean(&mut self) {
        self.list
            .retain(|todo| !matches!(todo.state, State::Done | State::Dropped));
    }

    fn get_todo(&mut self, id: usize) -> Result<&mut Todo> {
        self.list
            .get_mut(id - 1)
            .ok_or_else(|| anyhow!("Unknown ID"))
    }

    fn validate_id(&self, id: usize) -> Result<()> {
        if id == 0 || id > self.list.len() {
            bail!("Unknown ID");
        }
        Ok(())
    }

    fn collect_ids(
        &self,
        ids: Option<Vec<usize>>,
        from: Option<usize>,
        to: Option<usize>,
    ) -> Result<BTreeSet<usize>> {
        match (ids, from, to) {
            (None, Some(f), Some(t)) => {
                if f > t {
                    bail!("--from must be <= --to")
                }
                Ok((f..=t).collect())
            }
            (Some(mut list), None, None) => {
                list.sort();
                list.dedup();
                Ok(list.into_iter().collect())
            }
            (Some(list), Some(f), Some(t)) => {
                let mut id_set: BTreeSet<usize> = (f..=t).collect();
                id_set.extend(list);
                Ok(id_set)
            }
            (_, Some(_), None) => bail!("Add --to flag"),
            (_, None, Some(_)) => bail!("Add --from flag"),
            _ => bail!("Specify range of ids"),
        }
    }
}
