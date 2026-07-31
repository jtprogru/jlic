//! LICENSE file generator for open source projects.
//!
//! License texts are embedded in the binary; nothing is read from disk at runtime.
//!
//! ```
//! use jlic::{Context, License, render_license};
//!
//! let ctx = Context {
//!     year: "2026".into(),
//!     name: Some("Mikhail Savin".into()),
//!     email: None,
//! };
//! let text = render_license(License::Mit, &ctx).unwrap();
//! assert!(text.contains("Copyright (c) 2026 Mikhail Savin"));
//! ```

pub mod cli;
pub mod context;
pub mod error;
pub mod license;
pub mod output;
pub mod template;

pub use context::Context;
pub use error::{Error, Result};
pub use license::{Fields, License};

/// Renders the license text for a LICENSE file.
pub fn render_license(license: License, ctx: &Context) -> Result<String> {
    let rendered = template::render(license.template(), ctx)?;
    template::ensure_resolved(&rendered)?;
    Ok(rendered)
}

/// Renders the notice block for a source file header.
///
/// Unlike the license text, a notice always carries the copyright — including
/// for GPL and MPL, whose main text is immutable.
pub fn render_notice(license: License, ctx: &Context) -> Result<String> {
    let rendered = template::render(&license.notice_template(), ctx)?;
    template::ensure_resolved(&rendered)?;
    Ok(rendered)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx() -> Context {
        Context {
            year: "2026".into(),
            name: Some("Mikhail Savin".into()),
            email: Some("jtprogru@gmail.com".into()),
        }
    }

    #[test]
    fn every_license_renders_without_leftovers() {
        for license in License::ALL {
            let text = render_license(*license, &ctx())
                .unwrap_or_else(|e| panic!("{}: {e}", license.spdx_id()));
            assert!(!text.is_empty(), "{}: empty text", license.spdx_id());
            assert!(
                text.ends_with('\n'),
                "{}: missing trailing newline",
                license.spdx_id()
            );
        }
    }

    #[test]
    fn copyright_licenses_get_year_and_holder() {
        for license in License::ALL.iter().filter(|l| !l.fields().is_empty()) {
            let text = render_license(*license, &ctx()).unwrap();
            assert!(text.contains("2026"), "{}: no year", license.spdx_id());
            assert!(
                text.contains("Mikhail Savin <jtprogru@gmail.com>"),
                "{}: no copyright holder",
                license.spdx_id()
            );
        }
    }

    #[test]
    fn fixed_text_licenses_render_without_name() {
        let anonymous = Context {
            year: "2026".into(),
            name: None,
            email: None,
        };
        for license in License::ALL.iter().filter(|l| l.fields().is_empty()) {
            assert!(
                render_license(*license, &anonymous).is_ok(),
                "{} should render without a name",
                license.spdx_id()
            );
        }
    }

    #[test]
    fn every_notice_contains_spdx_identifier() {
        for license in License::ALL {
            let notice = render_notice(*license, &ctx()).unwrap();
            assert!(
                notice.contains(&format!("SPDX-License-Identifier: {}", license.spdx_id())),
                "{}: notice without an SPDX identifier",
                license.spdx_id()
            );
        }
    }
}
