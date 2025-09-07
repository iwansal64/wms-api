use std::fmt;
use rocket::time::{format_description::{self, well_known::Iso8601}, PrimitiveDateTime};
use serde::{de, Serializer, Deserializer};

pub fn serialize<S>(dt: &Option<PrimitiveDateTime>, serializer: S) -> Result<S::Ok, S::Error>
where
  S: Serializer {
  let formatter = format_description::parse("[year]-[month]-[day]T[hour]:[minute]:[second]").unwrap();
  match dt {
    Some(value) => serializer.serialize_str(value.format(&formatter).unwrap().as_str()),
    None => serializer.serialize_none()
  }
}

pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<PrimitiveDateTime>, D::Error>
where
  D: Deserializer<'de> {
  deserializer.deserialize_string(OptionalPrimitiveDateTimeVisitor)
}

struct OptionalPrimitiveDateTimeVisitor;

impl<'de> de::Visitor<'de> for OptionalPrimitiveDateTimeVisitor {
  type Value = Option<PrimitiveDateTime>;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a datetime that following ISO format")
  }

  fn visit_none<E>(self) -> Result<Self::Value, E>
      where
        E: de::Error, {
    return Ok(None);
  }
  
  fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
      E: de::Error, {

    if v == "" {
      return Ok(None);
    }
    
    let result: Result<PrimitiveDateTime, rocket::time::error::Parse> = PrimitiveDateTime::parse(v, &Iso8601::DEFAULT);

    match result {
      Ok(res) => Ok(Some(res)),
      Err(_) => Err(de::Error::custom("The given string is not date time in ISO format!"))
    }
  }
}