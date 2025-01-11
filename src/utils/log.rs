use std::fs::File;

use colog::format::CologStyle;
use env_logger::Builder;
use log::{Level, LevelFilter};
use regex::Regex;

struct CustomLevelTokens;

impl CologStyle for CustomLevelTokens {
    fn level_token(&self, level: &Level) -> &str {
        match *level {
            Level::Error => "ERR",
            Level::Warn => "WRN",
            Level::Info => "INF",
            Level::Debug => "DBG",
            Level::Trace => "TRC",
        }
    }
}

pub struct Logger;

impl Logger {
    pub fn init(level: Option<LevelFilter>) {
        let path = dirs::home_dir()
            .map(|dir| dir.join(".syncr").join("logs"))
            .unwrap();

        std::fs::create_dir_all(&path).unwrap();

        let file = Decolorifier {
            inner: File::options()
                .append(true)
                .create(true)
                .open(path.join(format!(
                    "{}-syncr.log",
                    chrono::Utc::now().format("%Y-%m-%d-%H-%M-%S")
                )))
                .unwrap(),
            regex: Regex::new(r"\u001b\[.*?m").unwrap(),
        };
        let stdout = std::io::stdout();

        let tee_writer = io_tee::TeeWriter::new(stdout, file);

        Builder::new()
            .filter(None, level.unwrap_or(LevelFilter::Info))
            .target(env_logger::Target::Pipe(Box::new(tee_writer)))
            .format(colog::formatter(CustomLevelTokens))
            .write_style(env_logger::WriteStyle::Always)
            .init();
    }
}

struct Decolorifier<W> {
    inner: W,
    regex: Regex,
}

impl<W: std::io::Write> std::io::Write for Decolorifier<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.inner.write(
            self.regex
                .replace_all(
                    std::str::from_utf8(buf).map_err(|_| std::io::ErrorKind::InvalidData)?,
                    "",
                )
                .as_bytes(),
        )
    }

    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.write(buf).map(|_| ())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}
