"""
Setup script for B+ Tree package with C extension.

This setup.py works with pyproject.toml for modern Python packaging.
Build C extension: python setup.py build_ext --inplace
Build package: python -m build
"""

from setuptools import setup, Extension, find_packages
import os
from pathlib import Path


# Read version from __init__.py
def get_version():
    init_file = Path(__file__).parent / "__init__.py"
    if init_file.exists():
        with open(init_file, "r") as f:
            for line in f:
                if line.startswith("__version__"):
                    return line.split("=")[1].strip().strip("\"'")
    return "0.1.0"


# Read long description from README
def get_long_description():
    readme_file = Path(__file__).parent / "README.md"
    if readme_file.exists():
        with open(readme_file, "r", encoding="utf-8") as f:
            return f.read()
    return ""


# Default compile flags: safe baseline with optimization
extra_compile_args = [
    "-O3",
    "-Wall",
    "-Wextra",
    "-Wno-unused-parameter",  # Common in Python C API
    "-std=c99",
]

# Platform-specific optimizations
import platform

if platform.system() != "Windows":
    extra_compile_args.extend(
        [
            "-fPIC",
            "-fno-strict-aliasing",
        ]
    )

# Opt-in flags for additional optimizations
if os.environ.get("BPLUSTREE_C_FAST_MATH"):
    extra_compile_args.append("-ffast-math")
if os.environ.get("BPLUSTREE_C_MARCH_NATIVE"):
    extra_compile_args.append("-march=native")

# Debug and sanitizer flags
extra_link_args = []
if os.environ.get("BPLUSTREE_C_DEBUG"):
    extra_compile_args.extend(["-g", "-O0", "-DDEBUG"])
    extra_compile_args.remove("-O3")
    # Remove NDEBUG for debug builds
    define_macros = []
else:
    define_macros = [("NDEBUG", "1")]

if os.environ.get("BPLUSTREE_C_SANITIZE"):
    sanitize_flags = ["-fsanitize=address", "-fno-omit-frame-pointer"]
    extra_compile_args.extend(sanitize_flags)
    extra_link_args.extend(sanitize_flags)

# Define the C extension module (temporarily disabled for stable builds)
bplustree_c = None
if os.environ.get("BPLUSTREE_BUILD_C_EXTENSION"):
    bplustree_c = Extension(
        "bplustree_c",
        sources=[
            "bplustree_c_src/bplustree_module.c",
            "bplustree_c_src/node_ops.c",
            "bplustree_c_src/tree_ops.c",
        ],
        include_dirs=["bplustree_c_src"],
        extra_compile_args=extra_compile_args,
        extra_link_args=extra_link_args,
        define_macros=define_macros,
        language="c",
    )

# Setup configuration
# Note: Most metadata now comes from pyproject.toml, but setup.py still needed for C extensions
setup(
    name="bplustree",
    version=get_version(),
    description="High-performance B+ Tree implementation for Python with dict-like API",
    long_description=get_long_description(),
    long_description_content_type="text/markdown",
    author="Kent Beck",
    author_email="kent@kentbeck.com",
    url="https://github.com/KentBeck/BPlusTree3",
    project_urls={
        "Homepage": "https://github.com/KentBeck/BPlusTree3",
        "Documentation": "https://github.com/KentBeck/BPlusTree3/tree/main/python",
        "Repository": "https://github.com/KentBeck/BPlusTree3",
        "Issues": "https://github.com/KentBeck/BPlusTree3/issues",
        "Changelog": "https://github.com/KentBeck/BPlusTree3/blob/main/python/CHANGELOG.md",
    },
    packages=find_packages(exclude=["tests*", "examples*", "docs*"]),
    ext_modules=[bplustree_c] if bplustree_c else [],
    include_package_data=True,
    zip_safe=False,
    python_requires=">=3.8",
    classifiers=[
        "Development Status :: 4 - Beta",
        "Intended Audience :: Developers",
        "Topic :: Software Development :: Libraries :: Python Modules",
        "Topic :: Database :: Database Engines/Servers",
        "Topic :: Software Development :: Libraries :: Data Structures",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Programming Language :: Python :: 3.12",
        "Programming Language :: Python :: Implementation :: CPython",
        "Programming Language :: C",
        "Operating System :: OS Independent",
        "Typing :: Typed",
    ],
    keywords=[
        "btree",
        "bplustree",
        "b+tree",
        "data-structure",
        "database",
        "indexing",
        "performance",
        "range-query",
        "ordered-dict",
        "sorted-dict",
    ],
)
