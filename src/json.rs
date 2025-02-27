// internal
use crate::create_file_with_fallback;
use crate::nuclide::NuclideData;

// standard lib
use std::path::Path;

// other
use anyhow::{Context, Result};

/// Writes the nuclide data to a JSON file at the specified path.
///
/// # Arguments
///
/// * `path` - The path where the JSON data should be written.
///
/// # Returns
///
/// A `Result` indicating success or failure.
pub fn write(nuclides: &[NuclideData], path: &Path) -> Result<()> {
    let f = create_file_with_fallback(path, "json", "decay_data.json")?;
    serde_json::to_writer_pretty(f, &nuclides).context("Unable to serialise to JSON")
}
