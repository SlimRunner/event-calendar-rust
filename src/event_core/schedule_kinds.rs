use core::fmt;
use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, serde::rfc3339};

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Exception {
    pub(super) index: usize,

    #[serde(flatten)]
    pub(super) kind: ExceptionKind,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind")]
pub(super) enum ExceptionKind {
    #[serde(rename = "skip")]
    Skip { with_replacement: bool },

    #[serde(rename = "multiplicity")]
    Multiplicity { count: NonZeroUsize },

    #[serde(rename = "shift")]
    Shift { shift: Shift },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Repeat {
    pub(super) starts: Rfc3339Date,
    pub(super) times: Option<usize>,
    pub(super) interval: Interval,
    pub(super) duration: Option<Interval>,
    #[serde(rename = "offset-count")]
    pub(super) offset_count: Option<usize>,
    #[serde(default)]
    pub(super) exceptions: Vec<Exception>,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Shift {
    pub(super) unit: TimeUnit,
    pub(super) count: isize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Interval {
    pub(super) unit: TimeUnit,
    pub(super) length: NonZeroUsize,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) enum TimeUnit {
    #[serde(rename = "hour")]
    Hour,
    #[serde(rename = "day")]
    Day,
    #[serde(rename = "week")]
    Week,
    #[serde(rename = "month")]
    Month,
    #[serde(rename = "year")]
    Year,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub struct Rfc3339Date(#[serde(with = "rfc3339")] pub(super) OffsetDateTime);

impl fmt::Display for Rfc3339Date {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
