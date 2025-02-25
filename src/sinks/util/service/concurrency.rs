use serde::de::{self, Unexpected, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt;

pub trait ConcurrencyOption {
    fn parse_concurrency(&self, default: &Self) -> Option<usize>;
    fn is_none(&self) -> bool;
    fn is_some(&self) -> bool {
        !self.is_none()
    }
}

impl ConcurrencyOption for Option<usize> {
    fn parse_concurrency(&self, default: &Self) -> Option<usize> {
        let limit = match self {
            None => *default,
            Some(x) => Some(*x),
        };
        limit.or(Some(1024))
    }

    fn is_none(&self) -> bool {
        matches!(self, None)
    }
}

impl ConcurrencyOption for Concurrency {
    fn parse_concurrency(&self, default: &Self) -> Option<usize> {
        match self.if_none(*default) {
            Concurrency::None => Some(1024),
            Concurrency::Adaptive => None,
            Concurrency::Fixed(limit) => Some(limit),
        }
    }

    fn is_none(&self) -> bool {
        matches!(self, Concurrency::None)
    }
}

#[derive(Clone, Copy, Debug, Derivative, Eq, PartialEq, Serialize)]
pub enum Concurrency {
    None,
    Adaptive,
    Fixed(usize),
}

impl Default for Concurrency {
    fn default() -> Self {
        Self::None
    }
}

impl Concurrency {
    pub fn if_none(self, other: Self) -> Self {
        match self {
            Self::None => other,
            _ => self,
        }
    }
}

impl<'de> Deserialize<'de> for Concurrency {
    // Deserialize either a positive integer or the string "adaptive"
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct UsizeOrAdaptive;

        impl<'de> Visitor<'de> for UsizeOrAdaptive {
            type Value = Concurrency;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str(r#"positive integer or "adaptive""#)
            }

            fn visit_str<E: de::Error>(self, value: &str) -> Result<Concurrency, E> {
                if value == "adaptive" {
                    Ok(Concurrency::Adaptive)
                } else {
                    Err(de::Error::unknown_variant(value, &["adaptive"]))
                }
            }

            fn visit_i64<E: de::Error>(self, value: i64) -> Result<Concurrency, E> {
                if value > 0 {
                    Ok(Concurrency::Fixed(value as usize))
                } else {
                    Err(de::Error::invalid_value(
                        Unexpected::Signed(value),
                        &"positive integer",
                    ))
                }
            }

            fn visit_u64<E: de::Error>(self, value: u64) -> Result<Concurrency, E> {
                if value > 0 {
                    Ok(Concurrency::Fixed(value as usize))
                } else {
                    Err(de::Error::invalid_value(
                        Unexpected::Unsigned(value),
                        &"positive integer",
                    ))
                }
            }
        }

        deserializer.deserialize_any(UsizeOrAdaptive)
    }
}
