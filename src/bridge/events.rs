use log::trace;
use nvim_rs::Value;
use std::convert::TryInto;
use std::fmt;

#[derive(Clone, Debug)]
pub enum ParseError {
    Array(Value),
    Map(Value),
    String(Value),
    U64(Value),
    I64(Value),
    F64(Value),
    Bool(Value),
    WindowAnchor(Value),
    Format(String),
}

type Result<T> = std::result::Result<T, ParseError>;

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ParseError::Array(value) => write!(f, "invalid array format {}", value),
            ParseError::Map(value) => write!(f, "invalid map format {}", value),
            ParseError::String(value) => write!(f, "invalid string format {}", value),
            ParseError::U64(value) => write!(f, "invalid u64 format {}", value),
            ParseError::I64(value) => write!(f, "invalid i64 format {}", value),
            ParseError::F64(value) => write!(f, "invalid f64 format {}", value),
            ParseError::Bool(value) => write!(f, "invalid bool format {}", value),
            ParseError::WindowAnchor(value) => {
                write!(f, "invalid window anchor format {}", value)
            }
            ParseError::Format(debug_text) => {
                write!(f, "invalid event format {}", debug_text)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum RedrawEvent {
    SetTitle { title: String },
}

pub fn parse_redraw_event(event_value: Value) -> Result<Vec<RedrawEvent>> {
    let mut event_contents = parse_array(event_value)?.into_iter();
    let event_name = event_contents
        .next()
        .ok_or_else(|| ParseError::Format(format!("{:?}", event_contents)))
        .and_then(parse_string)?;
    let mut parsed_events = Vec::with_capacity(event_contents.len());
    match event_name.as_str() {
        "set_title" => {
            for event in event_contents {
                parsed_events.push(parse_set_title(parse_array(event)?)?);
            }
        }
        _ => {
            trace!("un-parsed event {}", event_name);
        }
    }
    Ok(parsed_events)
}

#[inline]
fn parse_array(array_value: Value) -> Result<Vec<Value>> {
    array_value.try_into().map_err(ParseError::Array)
}

#[inline]
fn parse_string(string_value: Value) -> Result<String> {
    string_value.try_into().map_err(ParseError::String)
}

#[inline]
fn parse_set_title(set_title_arguments: Vec<Value>) -> Result<RedrawEvent> {
    let [title] = extract_values(set_title_arguments, [Value::Nil])?;
    Ok(RedrawEvent::SetTitle {
        title: parse_string(title)?,
    })
}

#[inline]
fn extract_values<Arr: AsMut<[Value]>>(values: Vec<Value>, mut arr: Arr) -> Result<Arr> {
    let arr_ref = arr.as_mut();
    if values.len() != arr_ref.len() {
        Err(ParseError::Format(format!("{:?}", values)))
    } else {
        for (i, val) in values.into_iter().enumerate() {
            arr_ref[i] = val;
        }
        Ok(arr)
    }
}
