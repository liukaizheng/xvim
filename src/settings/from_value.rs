use super::Value;
use log::error;

pub trait FromValue {
    fn from_value(&mut self, value: Value);
}

impl FromValue for f32 {
    fn from_value(&mut self, value: Value) {
        if value.is_f64() {
            *self = value.as_f64().unwrap() as f32;
        } else if value.is_i64() {
            *self = value.as_i64().unwrap() as f32;
        } else if value.is_u64() {
            *self = value.as_u64().unwrap() as f32;
        } else {
            error!("Setting expect an f32, but received {:?}", value);
        }
    }
}

impl FromValue for u64 {
    fn from_value(&mut self, value: Value) {
        if value.is_u64() {
            *self = value.as_u64().unwrap();
        } else {
            error!("Setting expect a u64, but received {:?}", value);
        }
    }
}

impl FromValue for u32 {
    fn from_value(&mut self, value: Value) {
        if value.is_u64() {
            *self = value.as_u64().unwrap() as u32;
        } else {
            error!("Setting expect u32, but received {:?}", value);
        }
    }
}

impl FromValue for i32 {
    fn from_value(&mut self, value: Value) {
        if value.is_i64() {
            *self = value.as_i64().unwrap() as i32;
        } else {
            error!("Setting expect i64, but received {:?}", value);
        }
    }
}

impl FromValue for String {
    fn from_value(&mut self, value: Value) {
        if value.is_str() {
            *self = String::from(value.as_str().unwrap());
        } else {
            error!("Setting expect a string, but received {:?}", value);
        }
    }
}

impl FromValue for bool {
    fn from_value(&mut self, value: Value) {
        if value.is_bool() {
            *self = value.as_bool().unwrap();
        } else if value.is_u64() {
            *self = value.as_u64().unwrap() != 0;
        } else {
            error!("Setting expect as bool or 0/1, but received {:?}", value);
        }
    }
}
