// internal
use crate::create_file_with_fallback;
use crate::nuclide::NuclideData;

// standard lib
use std::io::Write;
use std::path::Path;

// neutronics toolbox
use ntools::iaea::Record;
use ntools::utils::{f, ValueExt};

// other
use anyhow::Result;

const KEV_TO_MEV: f32 = 1.0e-03;

/// Writes the mcnp cards to a file at the specified path.
pub fn write(nuclides: &[NuclideData], id: usize, path: &Path) -> Result<()> {
    let mut f = create_file_with_fallback(path, "i", "mcnp.i")?;
    let cards = generate_mcnp_cards(nuclides, id);
    f.write_all(cards.as_bytes())?;
    Ok(())
}

/// Make source distribution cards for every nuclide
fn generate_mcnp_cards(nuclides: &[NuclideData], id: usize) -> String {
    let mut card = String::new();
    for (i, nuclide) in nuclides.iter().enumerate() {
        card += &nuclide_distribution(nuclide, id + i);
    }
    card
}

/// Make a single source distribution for a nuclide
fn nuclide_distribution(nuclide: &NuclideData, id: usize) -> String {
    // Need to filer out any nonsense values where energy/intensity is None
    let filtered_records = nuclide
        .records
        .iter()
        .filter(|record| record.energy.is_some() && record.intensity.is_some())
        .collect::<Vec<&Record>>();

    if filtered_records.is_empty() {
        return f!("c {} records contained no valid decay data\n", nuclide.name);
    }

    // Create a comment line with nuclide name and normalization factor
    let comment = f!(
        "sc{id:<5} {} decay data, norm = {} particles/decay",
        nuclide.name,
        nuclide.norm().sci(5, 2) // this is already ignoring None intensities
    );

    // Create the SI card with energy values
    let si_card = f!(
        "si{id} L {}",
        filtered_records
            .iter()
            .map(|record| (record.energy.unwrap() * KEV_TO_MEV).sci(5, 2))
            .collect::<Vec<String>>()
            .join(" ")
    );

    // Create the SP card with intensity values
    let sp_card = f!(
        "sp{id:<6}{}",
        filtered_records
            .iter()
            .map(|record| (record.intensity.unwrap() * 1e-2).sci(5, 2))
            .collect::<Vec<String>>()
            .join(" ")
    );

    // Combine the comment, SI card, and SP card with proper formatting
    f!(
        "{}\n{}\n{}\nc\n",
        comment,
        wrap_text(si_card, 80, "        "),
        wrap_text(sp_card, 80, "        ")
    )
}

// wrap everything to a fixed number of characters for mcnp
fn wrap_text(text: String, width: usize, subsequent_indent: &str) -> String {
    let options = textwrap::Options::new(width)
        .initial_indent("")
        .subsequent_indent(subsequent_indent)
        .word_splitter(textwrap::WordSplitter::NoHyphenation)
        .break_words(false);
    textwrap::fill(&text, options)
}
