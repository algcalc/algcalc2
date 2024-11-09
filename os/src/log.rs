#[cfg(feature = "defmt")]
pub(crate) use defmt::{debug, error, info, trace, warn};
#[cfg(feature = "tracing")]
pub(crate) use tracing::{debug, error, info, trace, warn};
