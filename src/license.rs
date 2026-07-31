//! Catalog of supported licenses and their templates.

use std::fmt;
use std::str::FromStr;

use crate::error::Error;

/// Which placeholders a license template accepts.
///
/// Some licenses (GPL, MPL) have immutable text: the name and year go only
/// into the notice block for source file headers, never into LICENSE itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Fields {
    pub year: bool,
    pub holder: bool,
}

impl Fields {
    const NONE: Self = Self {
        year: false,
        holder: false,
    };
    const COPYRIGHT: Self = Self {
        year: true,
        holder: true,
    };

    /// Whether the template accepts any substitution at all.
    pub fn is_empty(self) -> bool {
        !self.year && !self.holder
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum License {
    /// The default license.
    #[default]
    Mit,
    Apache2,
    Gpl3OrLater,
    Bsd3Clause,
    Mpl2,
    Isc,
    Wtfpl,
}

impl License {
    /// Every license, in the order `jlic list` prints them.
    pub const ALL: &'static [License] = &[
        License::Mit,
        License::Apache2,
        License::Gpl3OrLater,
        License::Bsd3Clause,
        License::Mpl2,
        License::Isc,
        License::Wtfpl,
    ];

    /// Canonical SPDX identifier.
    pub fn spdx_id(self) -> &'static str {
        match self {
            License::Mit => "MIT",
            License::Apache2 => "Apache-2.0",
            License::Gpl3OrLater => "GPL-3.0-or-later",
            License::Bsd3Clause => "BSD-3-Clause",
            License::Mpl2 => "MPL-2.0",
            License::Isc => "ISC",
            License::Wtfpl => "WTFPL",
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            License::Mit => "MIT License",
            License::Apache2 => "Apache License 2.0",
            License::Gpl3OrLater => "GNU General Public License v3.0 or later",
            License::Bsd3Clause => "BSD 3-Clause \"New\" or \"Revised\" License",
            License::Mpl2 => "Mozilla Public License 2.0",
            License::Isc => "ISC License",
            License::Wtfpl => "Do What The F*ck You Want To Public License",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            License::Mit => {
                "Short and permissive: keep the copyright and license notice, do what you like."
            }
            License::Apache2 => {
                "Permissive, with an explicit patent grant and a requirement to state changes."
            }
            License::Gpl3OrLater => {
                "Strong copyleft: derivative works must ship source under the same license."
            }
            License::Bsd3Clause => {
                "Permissive, and additionally forbids using the holder's name for promotion."
            }
            License::Mpl2 => {
                "File-level copyleft: modified files stay under MPL, the rest may stay proprietary."
            }
            License::Isc => "Functionally equivalent to MIT and BSD 2-Clause, in fewer words.",
            License::Wtfpl => "The shortest license there is, with no conditions attached.",
        }
    }

    /// The license page on spdx.org.
    pub fn url(self) -> &'static str {
        match self {
            License::Mit => "https://spdx.org/licenses/MIT.html",
            License::Apache2 => "https://spdx.org/licenses/Apache-2.0.html",
            License::Gpl3OrLater => "https://spdx.org/licenses/GPL-3.0-or-later.html",
            License::Bsd3Clause => "https://spdx.org/licenses/BSD-3-Clause.html",
            License::Mpl2 => "https://spdx.org/licenses/MPL-2.0.html",
            License::Isc => "https://spdx.org/licenses/ISC.html",
            License::Wtfpl => "https://spdx.org/licenses/WTFPL.html",
        }
    }

    pub fn fields(self) -> Fields {
        match self {
            License::Gpl3OrLater | License::Mpl2 => Fields::NONE,
            _ => Fields::COPYRIGHT,
        }
    }

    /// The license text with placeholders, embedded in the binary.
    pub fn template(self) -> &'static str {
        match self {
            License::Mit => include_str!("../assets/licenses/mit.txt"),
            License::Apache2 => include_str!("../assets/licenses/apache-2.0.txt"),
            License::Gpl3OrLater => include_str!("../assets/licenses/gpl-3.0-or-later.txt"),
            License::Bsd3Clause => include_str!("../assets/licenses/bsd-3-clause.txt"),
            License::Mpl2 => include_str!("../assets/licenses/mpl-2.0.txt"),
            License::Isc => include_str!("../assets/licenses/isc.txt"),
            License::Wtfpl => include_str!("../assets/licenses/wtfpl.txt"),
        }
    }

    /// The block to paste into a source file header.
    ///
    /// Licenses without their own boilerplate get a short notice assembled
    /// from the copyright line and the SPDX identifier.
    pub fn notice_template(self) -> String {
        match self {
            License::Apache2 => include_str!("../assets/notices/apache-2.0.txt").to_string(),
            License::Gpl3OrLater => {
                include_str!("../assets/notices/gpl-3.0-or-later.txt").to_string()
            }
            License::Mpl2 => include_str!("../assets/notices/mpl-2.0.txt").to_string(),
            other => format!(
                "Copyright (c) {{{{year}}}} {{{{holder}}}}\n\nSPDX-License-Identifier: {}\n",
                other.spdx_id()
            ),
        }
    }

    /// Aliases the license can be requested by on the command line.
    pub fn aliases(self) -> &'static [&'static str] {
        match self {
            License::Mit => &["mit"],
            License::Apache2 => &["apache-2.0", "apache2", "apache", "apache-2", "asl2"],
            License::Gpl3OrLater => &[
                "gpl-3.0-or-later",
                "gpl-3.0",
                "gpl3",
                "gplv3",
                "gpl",
                "gpl-3.0+",
                "gpl-3.0-only",
            ],
            License::Bsd3Clause => &["bsd-3-clause", "bsd3", "bsd-3", "bsd"],
            License::Mpl2 => &["mpl-2.0", "mpl2", "mpl"],
            License::Isc => &["isc"],
            License::Wtfpl => &["wtfpl", "wtfpl-2.0", "wtf"],
        }
    }
}

impl fmt::Display for License {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.spdx_id())
    }
}

impl FromStr for License {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let needle = s.trim().to_ascii_lowercase();
        License::ALL
            .iter()
            .copied()
            .find(|l| l.aliases().contains(&needle.as_str()))
            .ok_or_else(|| Error::UnknownLicense {
                requested: s.to_string(),
                suggestion: suggest(&needle),
            })
    }
}

/// The closest alias by common prefix — used to hint at typos.
fn suggest(needle: &str) -> Option<&'static str> {
    if needle.is_empty() {
        return None;
    }
    License::ALL
        .iter()
        .flat_map(|l| l.aliases().iter().map(move |a| (*a, l.spdx_id())))
        .find(|(alias, _)| alias.starts_with(needle) || needle.starts_with(*alias))
        .map(|(_, id)| id)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_aliases_case_insensitively() {
        assert_eq!("MIT".parse::<License>().unwrap(), License::Mit);
        assert_eq!("Apache2".parse::<License>().unwrap(), License::Apache2);
        assert_eq!("gplv3".parse::<License>().unwrap(), License::Gpl3OrLater);
        assert_eq!("  wtfpl  ".parse::<License>().unwrap(), License::Wtfpl);
    }

    #[test]
    fn unknown_license_suggests_closest() {
        let err = "apach".parse::<License>().unwrap_err();
        assert!(matches!(
            err,
            Error::UnknownLicense {
                suggestion: Some("Apache-2.0"),
                ..
            }
        ));
    }

    #[test]
    fn aliases_are_unique_across_licenses() {
        let mut seen = std::collections::HashSet::new();
        for license in License::ALL {
            for alias in license.aliases() {
                assert!(seen.insert(*alias), "duplicate alias: {alias}");
                assert_eq!(
                    *alias,
                    alias.to_ascii_lowercase(),
                    "alias is not lowercase: {alias}"
                );
            }
        }
    }

    #[test]
    fn spdx_id_resolves_to_itself() {
        for license in License::ALL {
            assert_eq!(license.spdx_id().parse::<License>().unwrap(), *license);
        }
    }
}
