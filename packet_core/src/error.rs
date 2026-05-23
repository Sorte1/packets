use serde_value::Value;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueKind {
    Bool,
    I8,
    I16,
    I32,
    I64,
    U8,
    U16,
    U32,
    U64,
    F32,
    F64,
    Char,
    String,
    Bytes,
    Seq,
    Map,
    Unit,
    Option,
    Newtype,
}

impl ValueKind {
    pub fn of(value: &Value) -> Self {
        match value {
            Value::Bool(_) => ValueKind::Bool,
            Value::I8(_) => ValueKind::I8,
            Value::I16(_) => ValueKind::I16,
            Value::I32(_) => ValueKind::I32,
            Value::I64(_) => ValueKind::I64,
            Value::U8(_) => ValueKind::U8,
            Value::U16(_) => ValueKind::U16,
            Value::U32(_) => ValueKind::U32,
            Value::U64(_) => ValueKind::U64,
            Value::F32(_) => ValueKind::F32,
            Value::F64(_) => ValueKind::F64,
            Value::Char(_) => ValueKind::Char,
            Value::String(_) => ValueKind::String,
            Value::Bytes(_) => ValueKind::Bytes,
            Value::Seq(_) => ValueKind::Seq,
            Value::Map(_) => ValueKind::Map,
            Value::Unit => ValueKind::Unit,
            Value::Option(_) => ValueKind::Option,
            Value::Newtype(_) => ValueKind::Newtype,
        }
    }
}

impl std::fmt::Display for ValueKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            ValueKind::Bool => "bool",
            ValueKind::I8 => "i8",
            ValueKind::I16 => "i16",
            ValueKind::I32 => "i32",
            ValueKind::I64 => "i64",
            ValueKind::U8 => "u8",
            ValueKind::U16 => "u16",
            ValueKind::U32 => "u32",
            ValueKind::U64 => "u64",
            ValueKind::F32 => "f32",
            ValueKind::F64 => "f64",
            ValueKind::Char => "char",
            ValueKind::String => "string",
            ValueKind::Bytes => "bytes",
            ValueKind::Seq => "seq",
            ValueKind::Map => "map",
            ValueKind::Unit => "unit",
            ValueKind::Option => "option",
            ValueKind::Newtype => "newtype",
        };
        f.write_str(s)
    }
}

#[derive(Debug, Error)]
pub enum DecodeErrorKind {
    #[error("event {event:?} field #{idx} {field:?}: missing required value")]
    Missing {
        event: &'static str,
        field: &'static str,
        idx: usize,
    },

    #[error("event {event:?} field #{idx} {field:?}: expected {expected}, got {got}")]
    TypeMismatch {
        event: &'static str,
        field: &'static str,
        idx: usize,
        expected: &'static str,
        got: ValueKind,
    },

    #[error("event {event:?} field #{idx} {field:?}: failed to decode (tried {tried:?}): {source}")]
    Coercion {
        event: &'static str,
        field: &'static str,
        idx: usize,
        tried: Vec<&'static str>,
        #[source]
        source: Box<DecodeError>,
    },

    #[error("event {event:?} field #{idx} {field:?}: expected multiple of {expected_multiple_of} elements, got {got}")]
    ChunkSize {
        event: &'static str,
        field: &'static str,
        idx: usize,
        expected_multiple_of: usize,
        got: usize,
    },

    #[error("frame error: {0}")]
    Frame(String),

    #[error("{0}")]
    Custom(String),
}

#[derive(Debug)]
pub struct DecodeError {
    pub kind: DecodeErrorKind,
    pub path: DecodePath,
}

impl DecodeError {
    pub fn new(kind: DecodeErrorKind) -> Self {
        Self {
            kind,
            path: DecodePath::default(),
        }
    }

    pub fn custom(msg: impl Into<String>) -> Self {
        Self::new(DecodeErrorKind::Custom(msg.into()))
    }

    pub fn prepend(&mut self, seg: PathSegment) {
        self.path.0.insert(0, seg);
    }
}

impl std::fmt::Display for DecodeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if !self.path.0.is_empty() {
            write!(f, "{}: ", self.path)?;
        }
        std::fmt::Display::fmt(&self.kind, f)
    }
}

impl std::error::Error for DecodeError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        std::error::Error::source(&self.kind)
    }
}

impl From<DecodeErrorKind> for DecodeError {
    fn from(kind: DecodeErrorKind) -> Self {
        Self::new(kind)
    }
}

#[derive(Debug, Clone)]
pub enum PathSegment {
    Event(&'static str),
    Field { name: &'static str, idx: usize },
    Chunk(usize),
    Sub(&'static str),
}

impl std::fmt::Display for PathSegment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PathSegment::Event(name) => f.write_str(name),
            PathSegment::Field { name, .. } => write!(f, ".{name}"),
            PathSegment::Chunk(i) => write!(f, "[{i}]"),
            PathSegment::Sub(name) => write!(f, ".{name}"),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct DecodePath(pub Vec<PathSegment>);

impl std::fmt::Display for DecodePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for seg in &self.0 {
            write!(f, "{seg}")?;
        }
        Ok(())
    }
}
