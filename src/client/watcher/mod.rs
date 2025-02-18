use debounce::EventDebouncer;
use globset::{Glob, GlobSet, GlobSetBuilder};
use log::info;
use notify::{
    Error, Event, EventHandler, EventKind, Watcher as _,
    event::{CreateKind, DataChange, ModifyKind, RemoveKind},
    recommended_watcher,
};
use std::{
    ops::{Deref, DerefMut},
    path::PathBuf,
    sync::mpsc::{Receiver, Sender},
};

use notify::{Config, FsEventWatcher, RecursiveMode};

use crate::common::config::SyncConfig;

use std::{
    // sync::mpsc,
    time::{Duration, Instant},
};

pub struct Watcher {
    pub path: PathBuf,
    pub parent: PathBuf,
    pub config: SyncConfig,
    pub watcher: FsEventWatcher,
    patterns: GlobSet,
    recv: Option<Receiver<notify::Result<Event>>>,
}

impl Watcher {
    pub async fn new(config: SyncConfig) -> anyhow::Result<Self> {
        let (tx, rx) = std::sync::mpsc::channel();

        let watcher = recommended_watcher(tx)?;

        let path = config
            .path
            .to_owned() // clone is acceptable but not the best way... easier though
            .canonicalize()?;

        let parent = path
            .parent()
            .ok_or(anyhow::anyhow!("No parent"))?
            .to_owned();

        let patterns = Self::build_globset(
            config
                .patterns
                .as_ref()
                .map(|p| p.iter().map(|p| p.into()).collect::<Vec<_>>()),
        );

        let mut inner = Self {
            watcher,
            config,
            path,
            parent,
            patterns,
            recv: Some(rx),
        };

        let watch_start = Instant::now();
        inner.watch()?;
        info!("Watching started in {:?}", watch_start.elapsed());

        Ok(inner)
    }

    fn build_globset(patterns: Option<Vec<&str>>) -> GlobSet {
        let mut builder = GlobSetBuilder::new();
        let patterns = patterns.unwrap_or(vec!["**/*"]);
        for pat in patterns {
            // If a pattern is invalid, you might choose to log and skip it.
            builder.add(Glob::new(pat.into()).expect("Invalid glob pattern"));
        }
        builder.build().expect("Failed to build glob set")
    }

    fn unwatch_all(&mut self) -> anyhow::Result<()> {
        self.watcher.unwatch(&self.parent)?;

        Ok(())
    }

    fn watch(&mut self) -> anyhow::Result<()> {
        self.watcher.configure(Config::with_follow_symlinks(
            Config::default(),
            !self.config.ignore_symlinks, // follow symlinks if not ignore
        ))?;

        self.watcher.watch(&self.parent, RecursiveMode::Recursive)?;

        Ok(())
    }

    fn recv(&mut self) -> Option<Receiver<notify::Result<Event>>> {
        self.recv.take()
    }

    pub async fn run(
        mut self,
        func: impl FnMut(Event) + std::marker::Send + 'static,
    ) -> anyhow::Result<()> {
        let delay = Duration::from_millis(self.config.debounce);
        let recv = self
            .recv()
            .ok_or(anyhow::anyhow!("No receiver, already watching?"))?;
        let globset = self.patterns.clone();

        let debouncer = EventDebouncer::new(delay, func);
        let hot_reload_path = self.path.clone();
        let hot_reload_debouncer = EventDebouncer::new(Duration::from_millis(1000), move |_| {
            if self.config.update() {
                // only reload if config has changed
                info!("Hot reloading watcher...");
                self.unwatch_all().unwrap();
                self.watch().unwrap();
            }
        });

        while let Ok(Ok(event)) = recv.recv() {
            println!("{:?}", event);

            if !event.paths.iter().all(|path| globset.is_match(path)) {
                continue;
            }

            match event.kind {
                EventKind::Modify(ModifyKind::Data(DataChange::Content)) => {
                    if event.paths.contains(&hot_reload_path) {
                        hot_reload_debouncer.put(());
                    } else {
                        debouncer.put(event);
                    }
                }
                EventKind::Create(_) => {
                    debouncer.put(event);
                }
                EventKind::Remove(_) => {
                    debouncer.put(event);
                }
                _ => {}
            }
        }

        Ok(())
    }
}
