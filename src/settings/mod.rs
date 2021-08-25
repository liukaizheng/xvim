mod from_value;

pub use from_value::FromValue;
pub use nvim_rs::Value;
use parking_lot::RwLock;
use std::convert::TryInto;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};

use crate::bridge::TxWrapper;
use log::trace;
use nvim_rs::Neovim;

lazy_static! {
    pub static ref SETTINGS: Settings = Settings::new();
}

pub trait SettingGroup {
    fn register(&self);
}

type UpdateHandlerFunc = fn(Value);
type ReaderFunc = fn() -> Value;

pub struct Settings {
    settings: RwLock<HashMap<TypeId, Box<dyn Any + Send + Sync>>>,
    listeners: RwLock<HashMap<String, UpdateHandlerFunc>>,
    readers: RwLock<HashMap<String, ReaderFunc>>,
}

impl Settings {
    fn new() -> Self {
        Self {
            settings: RwLock::new(HashMap::new()),
            listeners: RwLock::new(HashMap::new()),
            readers: RwLock::new(HashMap::new()),
        }
    }

    pub fn set_setting_handlers(
        &self,
        property_name: &str,
        update_func: UpdateHandlerFunc,
        reader_func: ReaderFunc,
    ) {
        self.listeners
            .write()
            .insert(String::from(property_name), update_func);
        self.readers
            .write()
            .insert(String::from(property_name), reader_func);
    }

    pub fn set<T: Clone + Send + Sync + 'static>(&self, t: &T) {
        let type_id = TypeId::of::<T>();
        let t: T = t.clone();
        unsafe {
            self.settings.force_unlock_write();
        }
        let mut write_lock = self.settings.write();
        write_lock.insert(type_id, Box::new(t));
    }

    pub fn get<T: Clone + Send + Sync + 'static>(&self) -> T {
        let read_lock = self.settings.read();
        let boxed = read_lock
            .get(&TypeId::of::<T>())
            .expect("Trying to retrieve a settings object that doesn't exist: {:?}");
        let value: &T = boxed
            .downcast_ref::<T>()
            .expect("Attempted to extract as setting object of the wrong type");
        value.clone()
    }

    pub async fn read_initial_values(&self, nvim: &Neovim<TxWrapper>) {
        let keys: Vec<String> = self.listeners.read().keys().cloned().collect();
        for name in keys {
            let variable_name = format!("xvim_{}", name);
            match nvim.get_var(&variable_name).await {
                Ok(value) => {
                    self.listeners.read().get(&name).unwrap()(value);
                }
                Err(error) => {
                    trace!("Initial value load failed for {}: {}", name, error);
                    let setting = self.readers.read().get(&name).unwrap()();
                    nvim.set_var(&variable_name, setting).await.ok();
                }
            }
        }
    }

    pub async fn setup_changed_listeners(&self, nvim: &Neovim<TxWrapper>) {
        let keys = self.listeners.read().keys().cloned().collect::<Vec<_>>();
        for name in keys {
            let vimscript = format!(
                concat!(
                    "exe \"",
                    "fun! XvimNotify{0}Changed(d, k, z)\n",
                    "call rpcnotify(1, 'setting_changed', '{0}', g:xvim_{0})\n",
                    "endf\n",
                    "call dictwatcheradd(g:, 'xvim_{0}', 'XvimNotify{0}Changed')\"",
                ),
                name
            );
            nvim.command(&vimscript)
                .await
                .expect(&format!("Could not setup setting notifier for {}", name));
        }
    }

    pub fn handle_changed_notification(&self, arguments: Vec<Value>) {
        let mut arguments = arguments.into_iter();
        let (name, value) = (arguments.next().unwrap(), arguments.next().unwrap());
        let name: Result<String, _> = name.try_into();
        let name = name.unwrap();
        self.listeners.read().get(&name).unwrap()(value);
    }
}
