// internal
use crate::create_file_with_fallback;
use crate::nuclide::NuclideData;

// standard lib
use std::io::Write;
use std::path::Path;

// neutronics toolbox
use ntools::iaea::Record;
use ntools::utils::{OptionExt, ValueExt};

// other
use anyhow::Result;
use colored::*;
use log::warn;

/// Represents a complete table of decay data for nuclides.
pub struct Table(String);

impl Table {
    /// Creates a new `Table` from a slice of `NuclideData`.
    ///
    /// # Arguments
    ///
    /// * `nuclides` - A slice of `NuclideData` to be included in the table.
    ///
    /// # Returns
    ///
    /// A fully generated `Table` as a colourised string.
    pub fn new(nuclides: &[NuclideData]) -> Self {
        let mut s = header();
        s += &content(nuclides);
        Self(s)
    }

    /// Prints the table to the standard output.
    pub fn print(&self, no_colour: bool) {
        if no_colour {
            std::io::stdout()
                .write_all(&strip_ansi_escapes::strip(&self.0))
                .expect("Unable to remove ansi colour escapes");
        } else {
            println!("{}", self.0)
        }
    }

    /// Writes the table to a file at the specified path.
    ///
    /// # Arguments
    ///
    /// * `path` - The path where the table should be written.
    ///
    /// # Returns
    ///
    /// A `Result` indicating success or failure.
    pub fn write(&self, path: &Path) -> Result<()> {
        let mut f = create_file_with_fallback(path, "txt", "table.txt")?;
        f.write_all(&strip_ansi_escapes::strip(&self.0))?;
        Ok(())
    }
}

/// Generates the table header.
fn header() -> String {
    let mut table = String::new();
    table.push_str(&format!("{:-<58}\n", ""));
    table.push_str(&format!(
        "  {:^5}   {:^5}  {:^5}   BR    Energy [keV]  Intensity [%]\n",
        "P", "Mode", "D"
    ));
    table.push_str(&format!("{:-<58}\n", ""));
    table
}

/// Generates the table content for all nuclide records.
fn content(nuclides: &[NuclideData]) -> String {
    let mut table = String::new();
    let mut missing_p_erg = false;

    for nuclide in nuclides {
        let mut p_energy = -1.0;
        table += &format_nuclide_header(nuclide, &mut p_energy, &mut missing_p_erg);

        for record in &nuclide.records {
            table += &format_record(nuclide, record, &mut p_energy, &mut missing_p_erg);
        }

        missing_p_erg = false;
    }

    table
}

/// Formats the header for a single nuclide.
fn format_nuclide_header(
    nuclide: &NuclideData,
    p_energy: &mut f32,
    missing_p_erg: &mut bool,
) -> String {
    let mut header = String::new();

    for record in &nuclide.records {
        let parent_energy = record.p_energy.unwrap_or_else(|| {
            if !*missing_p_erg {
                warn!(
                    "Assuming ground state for incomplete {} records",
                    nuclide.name
                );
            }
            *missing_p_erg = true;
            0.0
        });

        if parent_energy > *p_energy {
            *p_energy = parent_energy;
            header += &format!(
                "\n {} [E = {} {}, t1/2 = {}]\n",
                nuclide.name.magenta(),
                parent_energy.to_string().magenta(),
                "keV".magenta(),
                human_readable_halflife(record.half_life).magenta(),
            )
            .bold()
            .to_string();
        }
    }

    header
}

/// Formats a single record for a nuclide.
fn format_record(
    nuclide: &NuclideData,
    record: &Record,
    p_energy: &mut f32,
    missing_p_erg: &mut bool,
) -> String {
    let mut record_str = String::new();

    let parent_energy = record.p_energy.unwrap_or_else(|| {
        if !*missing_p_erg {
            warn!(
                "Assuming ground state for incomplete {} records",
                nuclide.name
            );
        }
        *missing_p_erg = true;
        0.0
    });

    if parent_energy > *p_energy {
        *p_energy = parent_energy;
        record_str += "\n";
    }

    record_str += &format!(
        "  {:<5} > {:^5} > {:<5} {:<6}     {:<7}     {:<7}\n",
        record.parent_name().blue(),
        record.decay_mode.display().cyan(),
        record.daughter_name().blue(),
        format_branching(record.branching),
        format_energy(record.energy),
        format_intensity(record.intensity)
    )
    .to_string();

    record_str
}

/// Formats the branching ratio.
fn format_branching(branching: Option<f32>) -> String {
    match branching {
        Some(br) if br >= 100.0 => String::new(),
        Some(br) if br >= 1.0 => format!("({:.0}%)", br),
        Some(_) => "(< 1%)".to_string(),
        None => "None".to_string(),
    }
}

/// Formats the energy value.
fn format_energy(energy: Option<f32>) -> String {
    match energy {
        Some(e) if e >= 10.0 => format!("{:.2}", e),
        Some(e) if e >= 0.001 => format!("{:.3}", e),
        Some(e) => format!("{:.2e}", e),
        None => "  -".to_string(),
    }
}

/// Formats the intensity value.
fn format_intensity(intensity: Option<f32>) -> String {
    match intensity {
        Some(i) if i >= 100.0 => format!("{:.1}", i),
        Some(i) if i >= 10.0 => format!("{:.2}", i),
        Some(i) if i >= 0.001 => format!("{:.3}", i),
        Some(i) => format!("{:.2e}", i),
        None => "  -".to_string(),
    }
}

/// Converts an optional half-life value in seconds to a human-readable string.
fn human_readable_halflife(halflife: Option<f32>) -> String {
    if let Some(seconds) = halflife {
        const SECONDS_IN_MINUTE: f32 = 60.0;
        const SECONDS_IN_HOUR: f32 = 60.0 * SECONDS_IN_MINUTE;
        const SECONDS_IN_DAY: f32 = 24.0 * SECONDS_IN_HOUR;
        const SECONDS_IN_YEAR: f32 = 365.0 * SECONDS_IN_DAY;
        const SECONDS_IN_MILLISECOND: f32 = 1e-3;
        const SECONDS_IN_MICROSECOND: f32 = 1e-6;
        const SECONDS_IN_NANOSECOND: f32 = 1e-9;

        match seconds {
            s if s >= 100.0 * SECONDS_IN_YEAR => {
                format!("{} years", (s / SECONDS_IN_YEAR).sci(2, 2))
            }
            s if s >= SECONDS_IN_YEAR => format!("{:.2} years", s / SECONDS_IN_YEAR),
            s if s >= SECONDS_IN_DAY => format!("{:.2} days", s / SECONDS_IN_DAY),
            s if s >= SECONDS_IN_HOUR => format!("{:.2} hours", s / SECONDS_IN_HOUR),
            s if s >= SECONDS_IN_MINUTE => format!("{:.2} minutes", s / SECONDS_IN_MINUTE),
            s if s >= 1.0 => format!("{:.2} s", s),
            s if s >= SECONDS_IN_MILLISECOND => format!("{:.2} ms", s / SECONDS_IN_MILLISECOND),
            s if s >= SECONDS_IN_MICROSECOND => format!("{:.2} us", s / SECONDS_IN_MICROSECOND),
            _ => format!("{:.2} ns", seconds / SECONDS_IN_NANOSECOND),
        }
    } else {
        "-".to_string()
    }
}
