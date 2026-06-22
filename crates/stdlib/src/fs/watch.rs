//! File watching utilities.

use std::path::{Path, PathBuf};
use std::sync::mpsc;

use notify::{Config, EventKind, RecommendedWatcher, RecursiveMode, Watcher};

use crate::error::{Result, StdlibError};

/// File system event types.
#[derive(Debug, Clone)]
pub enum FileEvent {
    Created(PathBuf),
    Modified(PathBuf),
    Removed(PathBuf),
}

/// Watch files and directories for changes.
pub struct FileWatcher {
    _watcher: RecommendedWatcher,
    rx: mpsc::Receiver<FileEvent>,
}

impl FileWatcher {
    /// Create a new file watcher with a callback.
    pub fn new<F>(callback: F) -> Result<Self>
    where
        F: Fn(FileEvent) + Send + 'static,
    {
        let (tx, rx) = mpsc::channel();

        let watcher = RecommendedWatcher::new(
            move |res: std::result::Result<notify::Event, notify::Error>| {
                if let Ok(event) = res {
                    for path in event.paths {
                        let file_event = match event.kind {
                            EventKind::Create(_) => FileEvent::Created(path),
                            EventKind::Modify(_) => FileEvent::Modified(path),
                            EventKind::Remove(_) => FileEvent::Removed(path),
                            _ => continue,
                        };
                        callback(file_event.clone());
                        let _ = tx.send(file_event);
                    }
                }
            },
            Config::default(),
        )
        .map_err(|e| StdlibError::Watch(e.to_string()))?;

        Ok(Self {
            _watcher: watcher,
            rx,
        })
    }

    /// Watch a path for changes.
    pub fn watch(&mut self, path: &Path, recursive: bool) -> Result<()> {
        let mode = if recursive {
            RecursiveMode::Recursive
        } else {
            RecursiveMode::NonRecursive
        };
        self._watcher
            .watch(path, mode)
            .map_err(|e| StdlibError::Watch(e.to_string()))
    }

    /// Try to receive the next event (non-blocking).
    pub fn try_recv(&self) -> Option<FileEvent> {
        self.rx.try_recv().ok()
    }
}
