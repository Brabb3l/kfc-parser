pub use kfc_base::*;

#[cfg(feature = "resource")]
pub use kfc_resource as resource;

#[cfg(feature = "content")]
pub use kfc_content as content;

mod schema;
mod snapshot;

/// Name-based helpers around physical KFC resource identifiers.
pub use container::resource_lookup as kfc_format;
pub use schema::*;
pub use snapshot::{SnapshotError, SnapshotResult, export_api_snapshot};
