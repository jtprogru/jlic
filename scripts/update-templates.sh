#!/usr/bin/env bash
# Refreshes the license texts and the references used to verify them.
#
# assets/licenses/ — templates carrying the {{year}} and {{holder}} placeholders,
#                    sourced from choosealicense.com (what GitHub's picker uses).
# assets/spdx/     — reference texts from SPDX, used by the test suite only.
#
# Always run `make test` afterwards: templates_match_spdx_reference_text
# compares every template against its reference.

set -euo pipefail

cd "$(dirname "$0")/.."

CHOOSEALICENSE="https://raw.githubusercontent.com/github/choosealicense.com/gh-pages/_licenses"
SPDX="https://raw.githubusercontent.com/spdx/license-list-data/main/text"

# choosealicense id : template file name : SPDX id
LICENSES=(
	"mit:mit:MIT"
	"apache-2.0:apache-2.0:Apache-2.0"
	"gpl-3.0:gpl-3.0-or-later:GPL-3.0-or-later"
	"bsd-3-clause:bsd-3-clause:BSD-3-Clause"
	"mpl-2.0:mpl-2.0:MPL-2.0"
	"isc:isc:ISC"
	"wtfpl:wtfpl:WTFPL"
)

mkdir -p assets/licenses assets/spdx

for entry in "${LICENSES[@]}"; do
	IFS=':' read -r source target spdx_id <<<"$entry"

	curl -sfL "$SPDX/$spdx_id.txt" -o "assets/spdx/$spdx_id.txt"

	raw="$(mktemp)"
	curl -sfL "$CHOOSEALICENSE/$source.txt" -o "$raw"

	target_file="assets/licenses/$target.txt"

	# 1. Drop the YAML frontmatter and the blank lines before the license text.
	# 2. Switch to our own placeholder syntax: square brackets occur inside the
	#    GPL and Apache texts as part of the license itself.
	# 3. WTFPL ships with its author's copyright; per wtfpl.net that line is
	#    meant to be replaced by the copyright of whoever applies the license.
	awk 'BEGIN { delim = 0 } /^---$/ { delim++; next } delim >= 2' "$raw" |
		sed '/./,$!d' |
		sed -e 's/\[year\]/{{year}}/g' \
			-e 's/\[fullname\]/{{holder}}/g' \
			-e 's/Copyright \[yyyy\] \[name of copyright owner\]/Copyright {{year}} {{holder}}/' \
			-e 's|Copyright (C) 2004 Sam Hocevar <sam@hocevar\.net>|Copyright (C) {{year}} {{holder}}|' \
			>"$target_file"

	rm -f "$raw"

	# Guarantee a trailing newline.
	if [ -n "$(tail -c1 "$target_file")" ]; then
		printf '\n' >>"$target_file"
	fi

	placeholders="$(grep -c '{{' "$target_file" || true)"
	printf '%-18s %6s bytes, lines with placeholders: %s\n' \
		"$target" "$(wc -c <"$target_file" | tr -d ' ')" "$placeholders"
done

# Pin the digests of the reference texts. CI re-checks them, so a silently
# changed upstream text cannot slip in without a visible line in the diff.
(cd assets/spdx && shasum -a 256 -- *.txt >CHECKSUMS.sha256)

echo
echo "Done. Review the result: git diff assets/ && make test"
echo "assets/spdx/CHECKSUMS.sha256 was regenerated — every changed digest must"
echo "be explained by a change you can see in the diff of the text next to it."
