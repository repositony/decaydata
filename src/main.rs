//! Command line tool to interact with IAEA decay data
#![doc(hidden)]

// crate modules
mod cli;
mod csv;
mod json;
mod mcnp;
mod nuclide;
mod table;
mod wrappers;

// Standard lib
use std::fs::{self, File};
use std::path::Path;

// external crates
use anyhow::{Context, Ok, Result};
use clap::Parser;
use log::{debug, error, warn};

fn main() -> Result<()> {
    // set up the command line interface and logging
    let cli = cli::Cli::parse();
    cli::init_logging(&cli)?;

    debug!("Parsing command line nuclides");
    let mut nuclides = nuclide::parse_nuclides(&cli)?;

    // fill with records for the relevant decay type
    debug!("Retrieving decay data");
    for n in nuclides.iter_mut() {
        n.find_records(cli.rad.into(), cli.fetch);
        n.sort_records(&cli.sort);
    }

    // filter out anything with no remaining records
    nuclides.retain(|n| !n.records.is_empty());

    // if none of them had decay data, then sources will be empty
    if nuclides.is_empty() {
        error!("No nuclides have relevant decay data records");
        return Ok(());
    }

    // sort the sources by name because why not
    nuclides.sort_by_key(|n| n.name.clone());

    let path = Path::new(&cli.output);

    // Gnerate a table for printing/writing
    let table = table::Table::new(&nuclides);
    if !cli.quiet {
        table.print(cli.no_colour);
    }

    if cli.text {
        debug!("Writing table to plain TEXT");
        table.write(path)?;
    }

    if cli.json {
        debug!("Writing to JSON");
        json::write(&nuclides, path)?;
    }

    if cli.mcnp {
        debug!("Writing MCNP cards");
        mcnp::write(&nuclides, cli.id, path)?;
    }

    if cli.csv {
        debug!("Fetching raw csv");
        csv::write(&nuclides, cli.rad.into(), path)?;
    }

    debug!("Done");
    Ok(())
}

/// Try to create a file, including all dirs, with a default to fallback on
fn create_file_with_fallback(path: &Path, extension: &str, default: &str) -> Result<File> {
    let mut p = path.to_path_buf();

    // Ensure all parent directories exist
    if let Some(parent) = path.parent() {
        if let Err(e) = fs::create_dir_all(parent) {
            warn!("{e}. Falling back to working directory.");
            p = p.file_name().expect("No file name provided").into();
        }
    }

    // Create the file, fall back to a default if not
    let f = File::create(p.with_extension(extension)).or_else(|e| {
        warn!("{e}. Falling back to \"{default}\".",);
        File::create(default).context("Unable to create fallback file")
    })?;

    Ok(f)
}
