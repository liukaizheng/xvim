use parking_lot::RwLock;
pub use rmpv::Value;
use std::{
    any::{Any, TypeId},
    collections::HashMap,
};
pub use from_value::FromValue;

mod from_value;

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
}
