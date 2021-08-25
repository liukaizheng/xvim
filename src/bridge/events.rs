use log::{debug, trace};
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

#[derive(Debug)]
pub enum ChannelStreamType {
    Stdio,
    Stderr,
    Socket,
    Job,
}

impl Default for ChannelStreamType {
    fn default() -> Self {
        Self::Stdio
    }
}

#[derive(Debug)]
pub enum ChannelMode {
    Bytes,
    Terminal,
    Rpc,
}

impl Default for ChannelMode {
    fn default() -> Self {
        Self::Bytes
    }
}

#[derive(Debug, Default)]
pub struct ClientVersion {
    pub major: u64,
    pub minor: Option<u64>,
    pub patch: Option<u64>,
    pub prerelease: Option<String>,
    pub commit: Option<String>,
}

#[derive(Debug)]
pub enum ClientType {
    Remote,
    Ui,
    Embedder,
    Host,
    Plugin,
}

impl Default for ClientType {
    fn default() -> Self {
        Self::Remote
    }
}

#[derive(Debug, Default)]
pub struct ClientInfo {
    pub name: String,
    pub version: ClientVersion,
    pub client_type: ClientType,
}

#[derive(Debug, Default)]
pub struct ChannelInfo {
    pub id: u64,
    pub stream: ChannelStreamType,
    pub mode: ChannelMode,
    pub pty: Option<String>,
    pub buffer: Option<String>,
    pub client: Option<ClientInfo>,
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
pub fn parse_channel_list(infos: Vec<Value>) -> Result<Vec<ChannelInfo>> {
    infos
        .into_iter()
        .map(parse_channel_info)
        .collect::<Result<Vec<_>>>()
}

pub fn parse_channel_info(value: Value) -> Result<ChannelInfo> {
    let channel_map = parse_map(value)?;
    let mut channel_info = ChannelInfo::default();
    for property in channel_map {
        if let (Value::String(name), val) = property {
            match (name.as_str().unwrap(), val) {
                ("id", channel_id) => channel_info.id = parse_u64(channel_id)?,
                ("stream", stream) => channel_info.stream = parse_channel_stream_type(stream)?,
                ("mode", mode) => channel_info.mode = parse_channel_mode(mode)?,
                ("pty", pty) => channel_info.pty = Some(parse_string(pty)?),
                ("buffer", buffer) => channel_info.buffer = Some(parse_string(buffer)?),
                ("client", client_info) => {
                    channel_info.client = Some(parse_client_info(client_info)?)
                }
                _ => debug!("Ignored channel info property: {}", name),
            }
        } else {
            debug!(
                "Invalid channel info format: ({}, {})",
                property.0, property.1
            );
        }
    }
    Ok(channel_info)
}

#[inline]
fn parse_channel_stream_type(val: Value) -> Result<ChannelStreamType> {
    match parse_string(val)?.as_ref() {
        "stdio" => Ok(ChannelStreamType::Stdio),
        "stderr" => Ok(ChannelStreamType::Stderr),
        "socket" => Ok(ChannelStreamType::Socket),
        "job" => Ok(ChannelStreamType::Job),
        stream_type => Err(ParseError::Format(format!("{:?}", stream_type))),
    }
}

#[inline]
fn parse_channel_mode(val: Value) -> Result<ChannelMode> {
    match parse_string(val)?.as_ref() {
        "bytes" => Ok(ChannelMode::Bytes),
        "terminal" => Ok(ChannelMode::Terminal),
        "rpc" => Ok(ChannelMode::Rpc),
        channel_mode => Err(ParseError::Format(format!("{:?}", channel_mode))),
    }
}

fn parse_client_info(client_info_value: Value) -> Result<ClientInfo> {
    let client_info_map = parse_map(client_info_value)?;
    let mut client_info = ClientInfo::default();
    for property in client_info_map {
        if let (Value::String(name), value) = property {
            match (name.as_str().unwrap(), value) {
                ("name", name) => client_info.name = parse_string(name)?,
                ("version", version) => client_info.version = parse_client_version(version)?,
                ("type", client_type) => client_info.client_type = parse_client_type(client_type)?,
                _ => debug!("Ignored client type property: {}", name),
            }
        } else {
            debug!(
                "Invalid client info format: ({}, {})",
                property.0, property.1
            );
        }
    }
    Ok(client_info)
}

fn parse_client_version(version_value: Value) -> Result<ClientVersion> {
    let version_map = parse_map(version_value)?;
    let mut version = ClientVersion::default();
    for property in version_map {
        if let (Value::String(name), value) = property {
            match (name.as_str().unwrap(), value) {
                ("major", major) => version.major = parse_u64(major)?,
                ("minor", minor) => version.minor = Some(parse_u64(minor)?),
                ("patch", patch) => version.patch = Some(parse_u64(patch)?),
                ("prerelease", prerelease) => version.prerelease = Some(parse_string(prerelease)?),
                ("commit", commit) => version.commit = Some(parse_string(commit)?),
                _ => debug!("Ignored client version property: {}", name),
            }
        } else {
            debug!(
                "Invalid client version format: ({}, {})",
                property.0, property.1
            );
        }
    }
    Ok(version)
}

#[inline]
fn parse_client_type(value: Value) -> Result<ClientType> {
    match parse_string(value)?.as_ref() {
        "remote" => Ok(ClientType::Remote),
        "ui" => Ok(ClientType::Ui),
        "embedder" => Ok(ClientType::Embedder),
        "host" => Ok(ClientType::Host),
        "plugin" => Ok(ClientType::Plugin),
        client_type => Err(ParseError::Format(format!("{:?}", client_type))),
    }
}

#[inline]
fn parse_array(array_value: Value) -> Result<Vec<Value>> {
    array_value.try_into().map_err(ParseError::Array)
}

#[inline]
fn parse_map(value: Value) -> Result<Vec<(Value, Value)>> {
    value.try_into().map_err(ParseError::Map)
}

#[inline]
fn parse_u64(val: Value) -> Result<u64> {
    val.try_into().map_err(ParseError::U64)
}

#[inline]
fn parse_i64(val: Value) -> Result<i64> {
    val.try_into().map_err(ParseError::I64)
}

#[inline]
fn parse_f64(val: Value) -> Result<f64> {
    val.try_into().map_err(ParseError::F64)
}

#[inline]
fn parse_bool(val: Value) -> Result<bool> {
    val.try_into().map_err(ParseError::Bool)
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
