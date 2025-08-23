"""
Pytest configuration for building the C extension before tests.
"""

import sys
import subprocess
from pathlib import Path

here = Path(__file__).parent
subprocess.check_call(
    [sys.executable, "setup.py", "build_ext", "--inplace"], cwd=str(here)
)

# Ensure the C extension built in this directory is importable
sys.path.insert(0, str(here))
