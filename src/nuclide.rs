// internal
use crate::cli::Cli;
use crate::wrappers::Property;

// neutronics toolbox
use ntools::iaea::{self, IsomerState, Nuclide, Record, RecordSet};

// other
use anyhow::{bail, Result};
use log::{debug, error, trace};
use serde::ser::{Serialize, SerializeStruct, Serializer};

/// Parse the user provided nuclides into something useful
pub fn parse_nuclides(cli: &Cli) -> Result<Vec<NuclideData>> {
    debug!("Command line nuclides: {:?}", cli.nuclides);

    // collect all unstable nuclides that also exist in the IAEA data
    let mut nuclide_data = cli
        .nuclides
        .iter()
        .filter_map(|n| Nuclide::try_from(n).ok())
        .filter_map(|n| expand_elements(n, cli).ok())
        .flatten()
        .map(|n| NuclideData {
            name: n.name_with_state(),
            nuclide: n,
            records: Vec::new(),
        })
        .collect::<Vec<NuclideData>>();

    trace!("Nuclides sorted by name");
    nuclide_data.sort_by_key(|n| n.name.clone());

    trace!("Removing duplicates");
    nuclide_data.dedup();
    if nuclide_data.is_empty() {
        error!(
            "No {} decay data found for any requested nuclide",
            cli.rad.name()
        );
        bail!("No decay data found")
    }

    debug!(
        "Final nuclide list:\n{:?}",
        nuclide_data
            .iter()
            .map(|n| n.name.clone())
            .collect::<Vec<String>>()
    );

    Ok(nuclide_data)
}

/// Expand elements into their nuclides
fn expand_elements(nuclide: Nuclide, cli: &Cli) -> Result<Vec<Nuclide>> {
    // ok to do in a loop, this is in a oncecell and only ever loaded once
    let available = match cli.fetch {
        false => iaea::load_available(cli.rad.into())?,
        true => iaea::fetch_available()?,
    };

    if nuclide.isotope != 0 {
        return Ok(vec![nuclide]);
    };

    // todo this should expand to all excited states too?
    debug!(
        "Expanding {} element into ground state isotopes",
        nuclide.symbol
    );
    let f: Vec<Nuclide> = available
        .into_iter()
        .filter(|n| n.symbol == nuclide.symbol)
        .collect();

    trace!(
        "{:?}",
        f.iter()
            .map(|nuclide| nuclide.name_with_state())
            .collect::<Vec<String>>()
    );

    Ok(f)
}

/// Basic data structure for collecting only the relevant nuclide records
#[derive(Debug, Clone)]
pub struct NuclideData {
    pub name: String,
    pub nuclide: iaea::Nuclide,
    pub records: RecordSet,
}

/// Custom serialisation of nuclide data
impl Serialize for NuclideData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // Create a struct serializer
        let mut state = serializer.serialize_struct("Nuclide", 3)?;

        state.serialize_field("name", &self.name)?;

        let energy: Vec<Option<f32>> = self.records.iter().map(|r| r.energy).collect();
        let intensity: Vec<Option<f32>> = self.records.iter().map(|r| r.intensity).collect();

        state.serialize_field("energy", &energy)?;
        state.serialize_field("intensity", &intensity)?;

        state.end()
    }
}

impl PartialEq for NuclideData {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name && self.nuclide == other.nuclide
    }
}

impl NuclideData {
    /// Normalisation factor for the decay data
    pub fn norm(&self) -> f64 {
        (self
            .records
            .iter()
            .fold(0.0, |acc, r| acc + r.intensity.unwrap_or(0.0))
            / 100.0) as f64
    }

    /// Find the relevant records for a particular nuclide and excited state
    pub fn find_records(&mut self, radtype: iaea::RadType, fetch: bool) {
        let nuclide_records = match fetch {
            false => iaea::load_nuclide(self.nuclide.clone(), radtype),
            true => iaea::fetch_nuclide(self.nuclide.clone(), radtype),
        };

        if nuclide_records.is_none() {
            trace!("{radtype:?} decay records for {}: 0", self.name,);
            return;
        }

        if let Some(records) = nuclide_records {
            // get the list of parent energies
            let mut parent_energy = records
                .iter()
                .filter_map(|r| r.p_energy)
                .collect::<Vec<f32>>();
            parent_energy.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            parent_energy.dedup();

            // get the index of the parent energy we care about
            let index = if let IsomerState::Excited(i) = self.nuclide.state {
                i as usize
            } else {
                0
            };

            let n = parent_energy.len();

            let target = if parent_energy[0] == 0.0 {
                if index >= n {
                    trace!("No {:?} records for excied state of {}", radtype, self.name);
                    return;
                }

                parent_energy[index]
            } else {
                trace!(
                    "Note that {} records do not include a ground state",
                    self.nuclide.name()
                );

                if index == 0 {
                    trace!(
                        "No {:?} records for the ground state of {}",
                        radtype,
                        self.name
                    );
                    return;
                }

                // assume the first record is the first excited state
                trace!(
                    "Assuming {} keV is the first excited state of {}",
                    parent_energy[0],
                    self.nuclide.name()
                );

                if index > n {
                    trace!("No {:?} records for excied state of {}", radtype, self.name);
                    return;
                }

                parent_energy[index - 1]
            };

            self.records = records
                .into_iter()
                .filter(|r| {
                    if let Some(e) = r.p_energy {
                        e == target
                    } else {
                        trace!("Unknown parent energy for {}", r.parent_name());
                        true
                    }
                })
                .collect::<Vec<Record>>();

            trace!(
                "{radtype:?} decay records for {}: {}",
                self.name,
                self.records.len(),
            );
        }
    }

    /// Sort records
    pub fn sort_records(&mut self, property: &Property) {
        match property {
            Property::Energy => {
                self.records.sort_by(|a, b| {
                    a.energy
                        .unwrap_or(-1.0)
                        .partial_cmp(&b.energy.unwrap_or(-1.0))
                        .unwrap()
                });
            }
            Property::Intensity => {
                self.records.sort_by(|a, b| {
                    b.intensity
                        .unwrap_or(-1.0)
                        .partial_cmp(&a.intensity.unwrap_or(-1.0))
                        .unwrap()
                });
            }
        }
    }
}
