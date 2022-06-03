use log::error;

pub trait AsString {
  fn as_string(&self) -> String;
  fn as_string_option(&self) -> Option<String>;
  fn into_inner(self) -> Vec<json::JsonValue>;
}

impl AsString for json::JsonValue {
  fn as_string(&self) -> String {
    match *self {
      json::JsonValue::Short(ref value)  => value.to_string(),
      json::JsonValue::String(ref value) => value.to_string(),
      _ => {
        error!("Expected a JSON String, however got: {}", self.dump());
        panic!("Expected a JSON String, however got: {}", self.dump())
      }
    }
  }

  fn as_string_option(&self) -> Option<String> {
    match *self {
      json::JsonValue::Short(ref value)  => Some(value.to_string()),
      json::JsonValue::String(ref value) => Some(value.to_string()),
      _ => None
    }
  }

  fn into_inner(self) -> Vec<json::JsonValue> {
    match self {
      json::JsonValue::Array(vec) => {
        vec
      },
      _ => vec![]
    }
  }
}