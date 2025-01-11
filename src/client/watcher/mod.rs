use debounce::EventDebouncer;
use log::info;
use notify::{
    Error, Event, EventKind,
    event::{DataChange, ModifyKind},
    recommended_watcher,
};

use std::{
    ops::{Deref, DerefMut},
    sync::mpsc,
    time::{Duration, Instant},
};

use crate::common::config::SyncConfig;

mod inner;
mod utils;
use inner::ImpartialWatcher;

pub struct Watcher {
    inner: ImpartialWatcher,
    recv: Receiver,
}

impl Watcher {
    pub async fn new(config: SyncConfig) -> anyhow::Result<Self> {
        let (tx, rx) = mpsc::channel();

        let watcher = recommended_watcher(tx)?;

        let path = config
            .path
            .to_owned() // clone is acceptable but not the best way... easier though
            .canonicalize()?;

        let parent = path
            .parent()
            .ok_or(anyhow::anyhow!("No parent"))?
            .to_owned();

        let mut inner = ImpartialWatcher {
            watcher,
            config,
            path,
            parent,
            watching: Vec::new(),
        };

        let watch_start = Instant::now();
        inner.watch()?;
        info!("Watching started in {:?}", watch_start.elapsed());

        Ok(Self { inner, recv: rx })
    }

    pub async fn run(
        self,
        func: impl FnMut(Event) + std::marker::Send + 'static,
    ) -> anyhow::Result<()> {
        let delay = Duration::from_millis(self.config.debounce);

        let Watcher {
            inner: mut watcher,
            recv: rx,
        } = self;

        let debouncer = EventDebouncer::new(delay, func);
        let hot_reload_path = watcher.path.clone();
        let hot_reload_debouncer = EventDebouncer::new(Duration::from_millis(1000), move |_| {
            if watcher.config.update() {
                // only reload if config has changed
                info!("Hot reloading watcher...");
                watcher.unwatch_all().unwrap();
                watcher.watch().unwrap();
            }
        });

        while let Ok(Ok(event)) = rx.recv() {
            if let EventKind::Modify(ModifyKind::Data(DataChange::Content)) = event.kind {
                if event.paths.contains(&hot_reload_path) {
                    hot_reload_debouncer.put(());
                } else {
                    debouncer.put(event);
                }
            }
        }

        Ok(())
    }
}

impl Deref for Watcher {
    type Target = ImpartialWatcher;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for Watcher {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

pub type Receiver = mpsc::Receiver<core::result::Result<Event, Error>>;
