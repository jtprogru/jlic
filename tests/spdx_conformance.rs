//! Checks the embedded templates against the reference texts from SPDX.
//!
//! Templates come from choosealicense.com (the same source GitHub's license
//! picker uses); the reference comes from spdx/license-list-data. The two
//! differ in line wrapping and typography, so they are compared normalized:
//! letters and digits only, lowercased.
//!
//! Only the legally meaningful part is compared — up to
//! "END OF TERMS AND CONDITIONS" where such a marker exists.

use jlic::License;

const TERMS_END: &str = "END OF TERMS AND CONDITIONS";

fn spdx_reference(license: License) -> &'static str {
    match license {
        License::Mit => include_str!("../assets/spdx/MIT.txt"),
        License::Apache2 => include_str!("../assets/spdx/Apache-2.0.txt"),
        License::Gpl3OrLater => include_str!("../assets/spdx/GPL-3.0-or-later.txt"),
        License::Bsd3Clause => include_str!("../assets/spdx/BSD-3-Clause.txt"),
        License::Mpl2 => include_str!("../assets/spdx/MPL-2.0.txt"),
        License::Isc => include_str!("../assets/spdx/ISC.txt"),
        License::Wtfpl => include_str!("../assets/spdx/WTFPL.txt"),
    }
}

/// Stands in for any copyright line: the reference has SPDX placeholders or
/// the license publisher's own copyright there, the template has ours.
/// Consecutive markers collapse — ISC carries two copyright lines, our
/// template carries one.
const COPYRIGHT_MARKER: &str = "\u{1}";

fn normalize(text: &str) -> String {
    let body = match text.find(TERMS_END) {
        Some(idx) => &text[..idx],
        None => text,
    };

    let mut lines: Vec<String> = Vec::new();
    for line in body.lines() {
        // Capitalized on purpose: a lowercase "copyright" starts lines broken
        // mid-sentence by wrapping (Apache-2.0).
        let is_copyright_line = line.trim_start().starts_with("Copyright");
        let normalized: String = if is_copyright_line {
            COPYRIGHT_MARKER.to_string()
        } else {
            line.chars()
                .filter(|c| c.is_alphanumeric())
                .flat_map(|c| c.to_lowercase())
                .collect()
        };
        if normalized.is_empty() {
            continue;
        }
        if normalized == COPYRIGHT_MARKER
            && lines.last().map(String::as_str) == Some(COPYRIGHT_MARKER)
        {
            continue;
        }
        lines.push(normalized);
    }

    let joined = lines.concat();
    // Everything before the first copyright line is presentation, not license
    // text: choosealicense.com prepends a "BSD 3-Clause License" heading that
    // the SPDX reference does not have.
    match joined.split_once(COPYRIGHT_MARKER) {
        Some((_, tail)) => tail.to_string(),
        None => joined,
    }
}

/// Part of the ISC text is replaceable: the SPDX reference names the holder
/// "ISC" (after Internet Systems Consortium), while the common redaction says
/// "THE AUTHOR". SPDX matching guidelines treat both as the same license.
fn unify_isc_wording(text: String, license: License) -> String {
    if license == License::Isc {
        text.replace("theauthor", "isc")
    } else {
        text
    }
}

#[test]
fn templates_match_spdx_reference_text() {
    for license in License::ALL {
        let ours = unify_isc_wording(normalize(license.template()), *license);
        let theirs = unify_isc_wording(normalize(spdx_reference(*license)), *license);

        assert_eq!(
            ours,
            theirs,
            "{}: template text diverges from the SPDX reference",
            license.spdx_id()
        );
    }
}

/// The SPDX copy of ISC carries ISC's own copyright, ours carries placeholders.
/// Check separately that a copyright line is present at all.
#[test]
fn copyright_licenses_keep_their_copyright_line() {
    for license in License::ALL.iter().filter(|l| !l.fields().is_empty()) {
        let template = license.template();
        assert!(
            template.contains("{{year}}") && template.contains("{{holder}}"),
            "{}: template has no copyright placeholders",
            license.spdx_id()
        );
        assert!(
            template.to_ascii_lowercase().contains("copyright"),
            "{}: template has no copyright line",
            license.spdx_id()
        );
    }
}

/// The GPL and MPL texts are immutable — they must carry no substitutions.
#[test]
fn fixed_text_licenses_have_no_placeholders() {
    for license in License::ALL.iter().filter(|l| l.fields().is_empty()) {
        assert!(
            !license.template().contains("{{"),
            "{}: found a placeholder inside immutable text",
            license.spdx_id()
        );
    }
}
