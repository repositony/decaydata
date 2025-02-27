//! Basic wrappers for external crate types

// Wrapper for ntools VTK format variants
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum CliRadType {
    Alpha,
    BetaPlus,
    BetaMinus,
    Gamma,
    Xray,
    Electron,
}

impl CliRadType {
    pub fn name(&self) -> &str {
        match self {
            CliRadType::Alpha => "alpha",
            CliRadType::BetaPlus => "beta plus",
            CliRadType::BetaMinus => "beta minus",
            CliRadType::Gamma => "gamma",
            CliRadType::Xray => "x-ray",
            CliRadType::Electron => "electron",
        }
    }
}

impl From<CliRadType> for ntools::iaea::RadType {
    fn from(format: CliRadType) -> Self {
        match format {
            CliRadType::Alpha => ntools::iaea::RadType::Alpha,
            CliRadType::BetaPlus => ntools::iaea::RadType::BetaPlus,
            CliRadType::BetaMinus => ntools::iaea::RadType::BetaMinus,
            CliRadType::Gamma => ntools::iaea::RadType::Gamma,
            CliRadType::Xray => ntools::iaea::RadType::Xray,
            CliRadType::Electron => ntools::iaea::RadType::Electron,
        }
    }
}

impl std::fmt::Display for CliRadType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Property {
    Intensity,
    #[default]
    Energy,
}

impl Property {
    pub fn name(&self) -> &str {
        match self {
            Property::Intensity => "intensity",
            Property::Energy => "energy",
        }
    }
}

impl From<String> for Property {
    fn from(property: String) -> Self {
        match property.to_lowercase().as_str() {
            "i" | "intensity" => Property::Intensity,
            "e" | "energy" => Property::Energy,
            _ => Property::default(),
        }
    }
}

impl std::fmt::Display for Property {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}
