
use std::fmt;
use rocket::time::{format_description::{self, well_known::Iso8601}, PrimitiveDateTime};
use serde::{de, Serializer, Deserializer};

pub fn serialize<S>(dt: &PrimitiveDateTime, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer {
  let formatter = format_description::parse("[year]-[month]-[date]T[hour]:[minute]:[second]").unwrap();
  serializer.serialize_str(dt.format(&formatter).unwrap().as_str())
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<PrimitiveDateTime, D::Error>
where
  D: Deserializer<'de> {
  deserializer.deserialize_string(PrimitiveDateTimeVisitor)
}

struct PrimitiveDateTimeVisitor;

impl<'de> de::Visitor<'de> for PrimitiveDateTimeVisitor {
  type Value = PrimitiveDateTime;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a datetime that following ISO format")
  }

  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
      where
          E: de::Error, {
      
      let result = PrimitiveDateTime::parse(v, &Iso8601::DEFAULT);

      match result {
        Ok(res) => Ok(res),
        Err(_) => Err(de::Error::custom("The given string is not date time in ISO format!"))
      }
  }
}