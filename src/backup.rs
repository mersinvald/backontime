use std::collections::HashMap;
use std::sync::Mutex;
use std::sync::Arc;
use std::sync::mpsc;
use std::time::Duration;
use backup_entity::BackupEntity;
use std::thread;

pub struct Backuper {
    entities: Vec<Arc<Mutex<BackupEntity>>>
}

impl Backuper {
    pub fn new(entities: Vec<BackupEntity>) -> Self {
        Backuper {
            entities: entities.into_iter()
                .map(|entity| Arc::new(Mutex::new(entity)))
                .collect()
        }
    }

    pub fn start(self) {
        if self.entities.is_empty() {
            error!("no backup entities, nothing to be done.");
            return;
        }

        // Spawn fs watcher thread
        let cloned_entities = self.entities.clone();
        thread::spawn(|| Self::watcher_thread_main(cloned_entities));

        loop {
            
            for entity in &self.entities {
                let mut entity = entity.lock().unwrap();
                let need_backup = {
                    let time_trigger = match entity.last_triggered.elapsed() {
                        Ok(duration) => duration >= Duration::from_secs(entity.trigger_timer * 60),
                        Err(_) => false
                    };

                    let changes_trigger = entity.changed >= entity.trigger_changes;

                    time_trigger || changes_trigger
                };

                if need_backup {
                    entity.backup().unwrap_or_else(|error| {
                        error!("fatal error for backup target {:?}: {}", entity.path, error);
                    })
               }
            }
            
            thread::sleep(Duration::from_secs(60));
        }

    }

    fn watcher_thread_main(entities: Vec<Arc<Mutex<BackupEntity>>>) {
        use notify::*;

        let mut lookup_table = entities.iter()
            .map(|entity| (
                entity.lock().unwrap().path.clone(),
                entity.clone()
            ))
            .collect::<HashMap<_, _>>();
        
        trace!("watching pathes:");
        for (path, _) in &lookup_table {
            trace!("{:?}", path);
        }

        // Create watcher
        let (tx, rx) = mpsc::channel();
        let mut watcher = watcher(tx, Duration::from_secs(60))
            .map_err(|e| error!("{}", e))
            .expect("failed to start file watcher");

        // Register watch pathes
        for (path, entity) in &lookup_table {
            let recursive_mode = if entity.lock().unwrap().recursive {
                RecursiveMode::Recursive
            } else {
                RecursiveMode::NonRecursive
            };

            watcher.watch(&path, recursive_mode)
                .unwrap_or_else(|e| error!("failed to start watching {}: {}", path.display(), e));
        }

        use notify::DebouncedEvent::*;

        loop {
            let path = match rx.recv() {
                Ok(event) => {
                    trace!("received fs event: {:?}", event);
                    match event {
                        NoticeWrite(path) | 
                        NoticeRemove(path) |
                        Create(path) |
                        Write(path) | 
                        Chmod(path) |
                        Remove(path) => path,
                        Rename(path, new_path) => {
                            // remove old path from lookup table
                            lookup_table.remove(&path).map(|entity| {
                                // modify path in the entity
                                entity.lock().unwrap().path = new_path.clone();
                                // push entity back into the lookup table
                                lookup_table.insert(new_path.clone(), entity);
                            });
                            new_path
                        },
                        _ => continue
                    }
                },
                Err(e) => {
                    error!("watching error: {}", e);
                    continue
                }
            };
            
            // Find path starting with one of watching pathes
            for (watching_path, entity) in &lookup_table {
                if path.starts_with(watching_path) {
                    let mut entity = entity.lock().unwrap();
                    entity.changed += 1;
                    trace!("incremented change counter for {:?}: {} -> {}", 
                        entity.path.display(),
                        entity.changed - 1,
                        entity.changed);
                }
            }
        }        
    }
}


