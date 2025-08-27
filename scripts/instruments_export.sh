#!/usr/bin/env bash
set -euo pipefail

TRACE_PATH=${1:-rust/delete_profile.trace}
OUT_DIR=${2:-rust/delete_export}

mkdir -p "$OUT_DIR"

echo "Exporting TOC to $OUT_DIR/toc.xml"
xcrun xctrace export --input "$TRACE_PATH" --toc > "$OUT_DIR/toc.xml"

echo "Exporting time-profile table to $OUT_DIR/time_profile.xml (if available)"
if ! xcrun xctrace export --input "$TRACE_PATH" --xpath '/trace-toc/run[@number="1"]/data/table[@schema="time-profile"]' > "$OUT_DIR/time_profile.xml"; then
  echo "time-profile export failed; continuing"
fi

echo "Exporting time-sample table to $OUT_DIR/time_sample.xml (if available)"
if ! xcrun xctrace export --input "$TRACE_PATH" --xpath '/trace-toc/run[@number="1"]/data/table[@schema="time-sample"]' > "$OUT_DIR/time_sample.xml"; then
  echo "time-sample export failed; continuing"
fi

echo "Exporting thread-info to $OUT_DIR/thread_info.xml"
xcrun xctrace export --input "$TRACE_PATH" --xpath '/trace-toc/run[@number="1"]/data/table[@schema="thread-info"]' > "$OUT_DIR/thread_info.xml"

echo "Exporting process-info to $OUT_DIR/process_info.xml"
xcrun xctrace export --input "$TRACE_PATH" --xpath '/trace-toc/run[@number="1"]/data/table[@schema="process-info"]' > "$OUT_DIR/process_info.xml"

echo "Exporting dyld-library-load to $OUT_DIR/dyld_library_load.xml"
xcrun xctrace export --input "$TRACE_PATH" --xpath '/trace-toc/run[@number="1"]/data/table[@schema="dyld-library-load"]' > "$OUT_DIR/dyld_library_load.xml"

echo "Done. Inspect XML files under $OUT_DIR"

