use kay::{World, Actor, TypedID};
use compact::{CHashMap, COption, CString, Compact};
use arrayvec::ArrayString;
use serde::de::DeserializeOwned;

pub type Name = ArrayString<[u8; 16]>;
pub trait Config: Compact + 'static {}

#[derive(Compact, Clone)]
pub struct ConfigManager<C: Config> {
    id: ConfigManagerID<C>,
    entries: CHashMap<Name, C>,
}

impl<C: Config> ConfigManager<C> {
    pub fn spawn(
        id: ConfigManagerID<C>,
        initial_entries: &CHashMap<Name, C>,
        _: &mut World,
    ) -> ConfigManager<C> {
        ConfigManager {
            id,
            entries: initial_entries.clone(),
        }
    }

    pub fn request_current(&self, requester: ConfigUserID<C>, world: &mut World) {
        for (name, value) in self.entries.pairs() {
            requester.on_config_change(*name, COption(Some(value.clone())), world);
        }
    }

    pub fn update_entry(&mut self, name: Name, maybe_value: &COption<C>, world: &mut World) {
        if let COption(Some(ref value)) = *maybe_value {
            ConfigUserID::<C>::global_broadcast(world).on_config_change(
                name,
                COption(Some(value.clone())),
                world,
            );
            self.entries.insert(name, value.clone());
        } else {
            ConfigUserID::<C>::global_broadcast(world).on_config_change(name, COption(None), world);
            self.entries.remove(name);
        }
    }

    pub fn update_all_entries(&mut self, entries: &CHashMap<Name, C>, world: &mut World) {
        // TODO: handle disappearing entries?
        self.entries = entries.clone();
        for (name, value) in self.entries.pairs() {
            ConfigUserID::<C>::global_broadcast(world).on_config_change(
                name.clone(),
                COption(Some(value.clone())),
                world,
            );
        }
    }
}

use std::{
    collections::HashMap,
    fs::File,
    io::BufReader,
    sync::mpsc::{Receiver, channel},
};
#[cfg(feature = "server")]
use std::sync::{Mutex, OnceLock};
use cb_time::actors::{Temporal, TemporalID};
use cb_time::units::{Instant};
#[cfg(feature = "server")]
use notify::Watcher;

#[cfg(feature = "server")]
struct ConfigFileWatcherState {
    receiver: Receiver<notify::DebouncedEvent>,
    watcher: notify::RecommendedWatcher,
}

/// The `notify` watchers live outside the actor, keyed by config type + file.
///
/// This used to be a `kay::External<Option<ConfigFileWatcherState>>` field,
/// which is the only way an actor can hold non-persistable state - but actor
/// state is written into the mmap savegame and `External::decompact` restores
/// the raw box pointer verbatim (`ptr::read`), so a watcher started before the
/// save is a dangling pointer in the next process. That segfaulted every load
/// of an existing city in this actor's `tick`, right after "Simulation
/// running." Keeping the watcher in a process-global registry instead means
/// the actor's own state is plain data, and the watcher is simply started
/// again on the first tick after a load.
#[cfg(feature = "server")]
fn config_watchers() -> &'static Mutex<HashMap<String, ConfigFileWatcherState>> {
    static WATCHERS: OnceLock<Mutex<HashMap<String, ConfigFileWatcherState>>> = OnceLock::new();
    WATCHERS.get_or_init(|| Mutex::new(HashMap::new()))
}

#[derive(Compact, Clone)]
pub struct ConfigFileWatcher<CD: Config + DeserializeOwned> {
    id: ConfigFileWatcherID<CD>,
    target: ConfigManagerID<CD>,
    file: CString,
}

impl<CD: Config + DeserializeOwned> ConfigFileWatcher<CD> {
    pub fn spawn(
        id: ConfigFileWatcherID<CD>,
        target: ConfigManagerID<CD>,
        file: &CString,
        _: &mut World,
    ) -> ConfigFileWatcher<CD> {
        ConfigFileWatcher {
            id,
            target,
            file: file.clone(),
        }
    }

    pub fn reload(&mut self, world: &mut World) {
        #[cfg(feature = "server")]
        {
            let file = File::open(&*(self.file))
                .expect(&format!("Couldn't find config file {:?}", &*self.file));
            let reader = BufReader::new(file);
            let new_entries: HashMap<Name, CD> =
                serde_yaml::from_reader(reader).expect("parsing failed");
            self.target
                .update_all_entries(new_entries.into_iter().collect(), world);
        }
    }
}

impl<CD: Config + DeserializeOwned> Temporal for ConfigFileWatcher<CD> {
    fn tick(&mut self, _dt: f32, _current_instant: Instant, world: &mut World) {
        #[cfg(feature = "server")]
        {
            let file_path = (*self.file).to_owned();
            let key = format!("{}\u{0}{}", std::any::type_name::<CD>(), file_path);

            let watching = {
                let watchers = config_watchers().lock().unwrap();
                watchers.contains_key(&key)
            };

            if !watching {
                self.reload(world);

                let mut watchers = config_watchers().lock().unwrap();
                if !watchers.contains_key(&key) {
                    let (sender, receiver) = channel();
                    let mut watcher =
                        notify::watcher(sender, std::time::Duration::from_secs(1)).unwrap();
                    watcher
                        .watch(&file_path, notify::RecursiveMode::Recursive)
                        .unwrap();
                    println!("Started watching config file . {:?}", &file_path);

                    watchers.insert(key.clone(), ConfigFileWatcherState { receiver, watcher });
                }
                return;
            }

            let event = {
                let mut watchers = config_watchers().lock().unwrap();
                watchers
                    .get_mut(&key)
                    .and_then(|state| state.receiver.try_recv().ok())
            };

            if let Some(event) = event {
                println!("Config file updated. {:?} {:?}", &file_path, event);
                self.reload(world);
            }
        }
    }
}

pub trait ConfigUser<C: Config>: Actor {
    fn local_cache(&mut self) -> &mut CHashMap<Name, C>;
    fn apply_config_change(&mut self, name: Name, maybe_value: &COption<C>, _: &mut World) {
        if let COption(Some(ref value)) = *maybe_value {
            self.local_cache().insert(name, value.clone());
        } else {
            self.local_cache().remove(name);
        }
    }
    fn on_config_change(&mut self, name: Name, maybe_value: &COption<C>, world: &mut World) {
        self.apply_config_change(name, maybe_value, world);
    }
    fn get_initial_config(&self, world: &mut World) {
        ConfigManagerID::<C>::global_first(world).request_current(self.id_as(), world);
    }
}

mod kay_auto;
pub use self::kay_auto::*;
