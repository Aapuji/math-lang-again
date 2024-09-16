use std::{io, path};

#[derive(Debug, Clone)]
pub struct Config {
    mode: Mode
}

impl Config {
    pub fn build<I: Iterator<Item = String>>(args: I) -> io::Result<Self> {
        Ok(match args.skip(1).next() {
            Some(arg) => Self {
                mode: if path::Path::new(&arg).try_exists()? {
                    Mode::File(arg)
                } else {
                    return Err(io::Error::new(io::ErrorKind::NotFound, "File not found"))
                }
            },
            None => Self {
                mode: Mode::Repl
            }
        })
    }

    pub fn mode(&self) -> &Mode {
        &self.mode
    }
}

#[derive(Debug, Clone)]
pub enum Mode {
    Repl,
    File(String)
}
