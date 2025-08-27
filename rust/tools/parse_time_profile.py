#!/usr/bin/env python3
import sys
import xml.etree.ElementTree as ET
from collections import Counter

"""
Best-effort parser for Instruments xctrace XML exports to list top functions/frames.
Usage:
  python3 rust/tools/parse_time_profile.py rust/delete_export/time_profile.xml

Notes:
- XML schema varies across Xcode versions; this script attempts to be robust.
- If time_profile.xml is empty or missing, try time_sample.xml instead:
  python3 rust/tools/parse_time_profile.py rust/delete_export/time_sample.xml
"""

def main(path: str) -> int:
    try:
        tree = ET.parse(path)
    except Exception as e:
        print(f"Failed to parse {path}: {e}")
        return 1

    root = tree.getroot()
    # Find all leaf text that look like function symbols; Instruments usually
    # includes stacks as text content or attributes in nested elements. We will
    # count any text nodes that look like code symbols (contain '::' or '['file:line']').
    counter = Counter()
    for elem in root.iter():
        text = (elem.text or '').strip()
        if not text:
            continue
        if '::' in text or ' - [' in text or ' + ' in text:
            # Normalize long frames by splitting on ' + ' (address offsets)
            frame = text.split(' + ')[0]
            counter[frame] += 1

    print("Top frames by sample count (heuristic):")
    for frame, count in counter.most_common(50):
        print(f"{count:>8}  {frame}")

    return 0

if __name__ == '__main__':
    if len(sys.argv) != 2:
        print("Usage: parse_time_profile.py <exported_xml>")
        sys.exit(2)
    sys.exit(main(sys.argv[1]))

