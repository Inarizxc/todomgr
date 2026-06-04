use anyhow::{Context, Result, anyhow, bail};
use colored::*;
use std::{
    collections::BTreeSet,
    fs::{self, File},
    io::{BufWriter, Write},
    path::PathBuf,
};

#[derive(Debug)]
enum State {
    Todo,
    Doing,
    Done,
    Dropped,
}

impl State {
    fn as_str(&self) -> &str {
        match self {
            State::Todo => "Todo",
            State::Doing => "Doing",
            State::Done => "Done",
            State::Dropped => "Dropped",
        }
    }
}

#[derive(Debug)]
struct Todo {
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
                    file_path.push(".todo");
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
                file_path: PathBuf::from(".todo"),
            }),
        }
    }
    pub fn open(&mut self) -> Result<()> {
        if !fs::exists(&self.file_path).context("File existence check error")? {
            self.create()?;
            return Ok(());
        }

        let content = fs::read_to_string(&self.file_path).context("File read error")?;
        for line in content.lines().filter(|line| !line.trim().is_empty()) {
            self.list
                .push(self.parse(line).context("Line parse error")?);
        }
        Ok(())
    }

    pub fn save(&self) -> Result<()> {
        let file = fs::File::create(&self.file_path)?;
        let mut writer = BufWriter::new(file);
        for todo in &self.list {
            writeln!(writer, "[{}] {}", todo.state.as_str(), todo.content)?;
        }
        Ok(())
    }

    pub fn print(&self) {
        for (id, todo) in self.list.iter().enumerate() {
            println!(
                "{}) {}",
                id + 1,
                match todo.state {
                    State::Todo => {
                        format!(
                            "{}  {}",
                            nerd_font_symbols::cod::COD_CIRCLE_LARGE,
                            todo.content
                        )
                        .normal()
                    }
                    State::Doing => {
                        format!(
                            "{}  {}",
                            nerd_font_symbols::md::MD_HELP_CIRCLE,
                            todo.content
                        )
                        .yellow()
                    }
                    State::Done => {
                        format!(
                            "{}  {}",
                            nerd_font_symbols::md::MD_CHECK_CIRCLE,
                            todo.content
                        )
                        .green()
                    }
                    State::Dropped => {
                        format!(
                            "{}  {}",
                            nerd_font_symbols::md::MD_CLOSE_CIRCLE,
                            todo.content.strikethrough()
                        )
                        .red()
                    }
                },
            )
        }
    }

    pub fn print_no_nerd_fonts(&self) {
        for (id, todo) in self.list.iter().enumerate() {
            println!(
                "{}) {}",
                id + 1,
                match todo.state {
                    State::Todo => {
                        format!("[ ]  {}", todo.content).normal()
                    }
                    State::Doing => {
                        format!("[/]  {}", todo.content).yellow()
                    }
                    State::Done => {
                        format!("[x]  {}", todo.content).green()
                    }
                    State::Dropped => {
                        format!("[-]  {}", todo.content.strikethrough()).red()
                    }
                },
            )
        }
    }

    pub fn add(&mut self, vec_content: Vec<String>, sep: Option<char>) {
        let content = vec_content.join(" ");
        match sep {
            Some(s) => {
                for c in content.split(s) {
                    self.list.push(Todo {
                        state: State::Todo,
                        content: c.trim().to_string(),
                    })
                }
            }
            None => self.list.push(Todo {
                state: State::Todo,
                content: content.clone(),
            }),
        }
    }

    pub fn rewrite(&mut self, id: u16, vec_content: Vec<String>) -> Result<()> {
        let content = vec_content.join(" ");
        let todo = self.get_todo(id).context("Get todo error")?;
        todo.content = content;
        Ok(())
    }

    pub fn remove(
        &mut self,
        ids: Option<Vec<u16>>,
        from: Option<u16>,
        to: Option<u16>,
    ) -> Result<()> {
        let todo_set = self.collect_ids(ids, from, to)?;
        for id in todo_set.iter().rev() {
            if *id as usize == 0 || (*id as usize) > self.list.len() {
                bail!("Unknown ID");
            }
            self.list.remove((id - 1) as usize);
        }

        Ok(())
    }

    pub fn switch(
        &mut self,
        ids: Option<Vec<u16>>,
        from: Option<u16>,
        to: Option<u16>,
    ) -> Result<()> {
        let todo_set = self.collect_ids(ids, from, to)?;
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
        ids: Option<Vec<u16>>,
        from: Option<u16>,
        to: Option<u16>,
    ) -> Result<()> {
        let todo_set = self.collect_ids(ids, from, to)?;
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

    fn get_todo(&mut self, id: u16) -> Result<&mut Todo> {
        if id == 0 {
            bail!("Unknown ID")
        }
        self.list
            .get_mut((id - 1) as usize)
            .ok_or_else(|| anyhow!("Unknown ID"))
    }

    fn create(&mut self) -> Result<()> {
        File::create_new(&self.file_path).context("File create error")?;
        println!("File `{}` created", self.file_path.display());
        Ok(())
    }

    fn parse(&self, todo_str: &str) -> Result<Todo> {
        let splited: Vec<&str> = todo_str.split_whitespace().collect();

        let mut iter = splited.iter();

        let state: State = match iter
            .next()
            .ok_or_else(|| anyhow!("State field is empty"))?
            .trim()
            .trim_matches(|c| c == '[' || c == ']')
        {
            "Todo" => State::Todo,
            "Doing" => State::Doing,
            "Done" => State::Done,
            "Dropped" => State::Dropped,

            _ => {
                bail!("Unknown state")
            }
        };
        let content: String = splited[1..].join(" ");

        Ok(Todo { state, content })
    }

    fn collect_ids(
        &self,
        ids: Option<Vec<u16>>,
        from: Option<u16>,
        to: Option<u16>,
    ) -> Result<BTreeSet<u16>> {
        match (ids, from, to) {
            (None, Some(f), Some(t)) => {
                if f >= t {
                    bail!("--from must be less them --to")
                }
                Ok((f..=t).collect())
            }
            (Some(mut list), None, None) => {
                list.sort();
                list.dedup();
                Ok(list.iter().copied().collect())
            }
            (Some(list), Some(f), Some(t)) => {
                let mut id_set: BTreeSet<u16> = (f..=t).collect();
                id_set.extend(list);
                Ok(id_set)
            }
            (_, Some(_), None) => bail!("Add --to flag"),
            (_, None, Some(_)) => bail!("Add --from flag"),
            _ => bail!("Specify range of ids"),
        }
    }
}
