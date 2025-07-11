import os
import sys
sys.path.insert(0, os.path.abspath('../py_api'))

# Configuration file for the Sphinx documentation builder.
#
# For the full list of built-in configuration values, see the documentation:
# https://www.sphinx-doc.org/en/master/usage/configuration.html

# -- Project information -----------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#project-information
project = 'NI Pulse Streamer'
copyright = '2025, Project contributors'
author = 'Project contributors'
release = '0.1.0'

# -- General configuration ---------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#general-configuration
extensions = [
    'myst_nb',
    'sphinx.ext.autodoc',
    'sphinx.ext.viewcode',
    'sphinx_autodoc_typehints',
]
myst_enable_extensions = ["colon_fence"]
nb_execution_mode = "off"  # Don't execute notebooks on build
templates_path = ['_templates']
exclude_patterns = ['_build', 'Thumbs.db', '.DS_Store']

# -- Options for HTML output -------------------------------------------------
# https://www.sphinx-doc.org/en/master/usage/configuration.html#options-for-html-output
html_theme = 'sphinx_rtd_theme'
html_static_path = ['_static']


# Use the Makefile to build the docs, like so:
#    make builder
# where "builder" is one of the supported builders, e.g. html, latex or linkcheck.
