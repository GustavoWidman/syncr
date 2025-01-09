use jwalk::{WalkDir, WalkDirGeneric};
use notify::{
    Event, FsEventWatcher, RecursiveMode, Result, Watcher as NotifyWatcher, recommended_watcher,
};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
    any,
    path::{Path, PathBuf},
    sync::{Arc, Mutex, mpsc},
};

use crate::common::config::{
    SyncConfig,
    sync::structure::{IgnoreMode, Pattern},
};

pub struct Watcher {
    inner: FsEventWatcher,
    dir: &'static Path,
    recv: mpsc::Receiver<Result<Event>>,
    config: &'static SyncConfig,
}

impl Watcher {
    pub fn new(config: &'static SyncConfig) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel::<Result<Event>>();

        let watcher = recommended_watcher(tx)?;

        let out = Self {
            inner: watcher,
            recv: rx,
            config,
            dir: config.path.parent().ok_or(anyhow::anyhow!("No parent"))?,
        };

        match &out.config.config.mode {
            IgnoreMode::Whitelist { whitelist } => out.watch_whitelist(whitelist),
            IgnoreMode::Blacklist { blacklist } => out.watch_blacklist(blacklist),
        };

        Ok(out)
    }

    pub fn watch_blacklist(&self, patterns: &Vec<Pattern>) {
        todo!()
    }

    pub fn watch_whitelist(&self, patterns: &Vec<Pattern>) -> anyhow::Result<()> {
        let files_to_watch = WalkDirGeneric::new::<&Path>(self.dir).process_read_dir(
            |depth, path, read_dir_state, children| {
                children.retain(|result| {
                    result
                        .as_ref()
                        .map(|entry| {
                            entry
                                .file_name
                                .to_str()
                                .map(|s| patterns.par_iter().any(|p| p.pattern.matches(s)))
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
                });
            },
        );

        for entry in files_to_watch {
            println!("{:?}", entry);
        }

        todo!()
    }
}
