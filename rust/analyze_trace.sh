#!/bin/bash

# Script to analyze the Instruments trace data
echo "Delete Operation Trace Analysis"
echo "==============================="

if [ -d "delete_profile.trace" ]; then
    echo "Trace file found: delete_profile.trace"
    echo ""
    echo "To analyze the trace data:"
    echo "1. Open with Instruments: open delete_profile.trace"
    echo "2. Look for the following hot functions:"
    echo "   - bplustree::BPlusTreeMap::remove"
    echo "   - bplustree::BPlusTreeMap::remove_recursive"
    echo "   - bplustree::BPlusTreeMap::rebalance_child"
    echo "   - Arena access methods (get_branch, get_leaf)"
    echo ""
    echo "3. Focus on:"
    echo "   - Functions with highest self time"
    echo "   - Functions called most frequently"
    echo "   - Memory allocation patterns"
    echo ""
    
    # Try to extract basic info from trace
    echo "Basic trace information:"
    xcrun xctrace export --input delete_profile.trace --xpath '/trace-toc/run/data/table[@schema=\"time-profile\"]/row' 2>/dev/null | head -20 || echo "Could not extract trace data automatically"
    
else
    echo "Trace file not found. Run the profiler first:"
    echo "xcrun xctrace record --template \"Time Profiler\" --launch ./target/release/delete_profiler --output delete_profile.trace"
fi

echo ""
echo "For detailed analysis, open the trace file in Instruments:"
echo "open delete_profile.trace"