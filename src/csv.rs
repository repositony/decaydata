// internal
use crate::create_file_with_fallback;
use crate::nuclide::NuclideData;

// standard lib
use std::io::Write;
use std::path::Path;

// other
use anyhow::Result;

// neutronics toolbox
use ntools::iaea::{self, RadType};
use ntools::utils::f;

/// Writes the completely unedited data to a CSV direct from IAEA
pub fn write(nuclides: &[NuclideData], rad_type: RadType, path: &Path) -> Result<()> {
    let mut f = create_file_with_fallback(path, "csv", "decay_data.csv")?;

    let csv_records = fetch_csv_records(nuclides, rad_type);
    f.write_all(csv_records.as_bytes())?;
    Ok(())
}

/// Make source distribution cards for every nuclide
fn fetch_csv_records(nuclides: &[NuclideData], rad_type: RadType) -> String {
    let mut csv = String::new();

    // can only get all records, so will need to dedup excied states and just
    // return everything
    let mut requests = nuclides
        .iter()
        .map(|n| n.nuclide.name())
        .collect::<Vec<String>>();
    requests.dedup();

    for nuclide in &requests {
        csv += &f!("\nIAEA {nuclide} CSV records for {:?} decay\n", rad_type);
        csv += &iaea::fetch_csv(nuclide, rad_type).unwrap_or(f!(
            "\nNo CSV data found for {:?} records of {}",
            rad_type,
            nuclide
        ))
    }

    csv
}
