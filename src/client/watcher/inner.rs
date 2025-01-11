use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
};

use notify::{Config, FsEventWatcher, RecursiveMode, Watcher};

use crate::common::config::SyncConfig;

use super::utils::{SplitResult, split_i32};

pub struct ImpartialWatcher {
    pub path: PathBuf,
    pub parent: PathBuf,
    pub watching: Vec<PathBuf>,
    pub config: SyncConfig,
    pub watcher: FsEventWatcher,
}

impl ImpartialWatcher {
    pub fn unwatch_all(&mut self) -> anyhow::Result<()> {
        if self.watching.is_empty() {
            return Ok(());
        }

        for entry in self.watching.iter() {
            self.watcher.unwatch(entry)?;
        }

        self.watching = Vec::new();

        Ok(())
    }

    pub fn watch(&mut self) -> anyhow::Result<()> {
        let mut patterns = match &self.config.patterns {
            Some(patterns) => patterns
                .iter()
                .map(|p| p.pattern.as_str())
                .collect::<Vec<_>>(),
            None => {
                return self.watch_full();
            }
        };

        if patterns.len() == 1 && patterns[0] == "**/*" {
            return self.watch_full();
        }

        match self.config.ignore_hidden {
            true => patterns.push("!**/.*"),
            false => {}
        }

        let mut files = match split_i32(self.config.max_depth) {
            SplitResult::Positive(n) => {
                globwalk::GlobWalkerBuilder::from_patterns(&self.parent, &patterns).max_depth(n)
            }
            SplitResult::Negative => {
                globwalk::GlobWalkerBuilder::from_patterns(&self.parent, &patterns)
            }
        }
        .follow_links(!self.config.ignore_symlinks)
        .build()?
        .filter_map(|r| r.ok())
        .map(|entry| entry.into_path())
        .collect::<Vec<_>>();

        files.push(self.path.clone());

        for file in files.iter() {
            self.watcher.watch(&file, RecursiveMode::NonRecursive)?;
        }

        self.watching = files;

        Ok(())
    }

    fn watch_full(&mut self) -> anyhow::Result<()> {
        self.watcher.configure(Config::with_follow_symlinks(
            Config::default(),
            !self.config.ignore_symlinks, // follow symlinks if not ignore
        ))?;

        self.watcher.watch(&self.parent, RecursiveMode::Recursive)?;

        self.watching = vec![self.parent.clone()];

        Ok(())
    }
}

impl Deref for ImpartialWatcher {
    type Target = FsEventWatcher;

    fn deref(&self) -> &Self::Target {
        &self.watcher
    }
}

impl DerefMut for ImpartialWatcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.watcher
    }
}
