#[cfg(feature = "defmt")]
#[allow(unused_imports)]
pub(crate) use defmt::{debug, error, info, trace, warn};
#[cfg(feature = "tracing")]
#[allow(unused_imports)]
pub(crate) use tracing::{debug, error, info, trace, warn};
