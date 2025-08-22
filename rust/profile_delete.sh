#!/bin/bash

# Delete Operation Profiling Script
# Runs comprehensive profiling on delete operations

echo "Building profiling target..."
cargo build --release --bin delete_profiler

echo "Running basic timing profiler..."
./target/release/delete_profiler

echo ""
echo "Running with Instruments (macOS profiler)..."
echo "This will generate detailed function-level profiling data"

# Use Instruments on macOS for detailed profiling
if command -v xcrun &> /dev/null; then
    echo "Starting Instruments profiling..."
    xcrun xctrace record --template "Time Profiler" --launch ./target/release/delete_profiler --output delete_profile.trace
    echo "Profiling data saved to delete_profile.trace"
    echo "Open with: open delete_profile.trace"
else
    echo "Instruments not available. Using alternative profiling..."
    
    # Fallback to time and basic profiling
    echo "Running with time profiling..."
    time ./target/release/delete_profiler
fi

echo ""
echo "Profiling complete. Check delete_profile.trace for detailed analysis."