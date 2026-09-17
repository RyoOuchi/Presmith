#!/usr/bin/env python3
"""Materialize shared assets from canonical sources; never overwrite authored slides."""
from pathlib import Path
import shutil
root = Path(__file__).resolve().parents[1]
deck = root / 'examples/product'
for source, destination in [('library/decksmith.css','lib/decksmith.css'),('library/decksmith.js','lib/decksmith.js'),('library/themes/ink.css','styles/theme.css'),('library/themes/paper.css','styles/paper.css'),('templates/starter/AGENTS.md','AGENTS.md'),('templates/starter/README.md','README.md')]:
    target = deck / destination
    target.parent.mkdir(parents=True, exist_ok=True)
    shutil.copyfile(root / source, target)
for source, destination in [('renderer','tooling/renderer'),('docs/project','docs')]:
    shutil.copytree(root / source, deck / destination, dirs_exist_ok=True,
                    ignore=shutil.ignore_patterns('node_modules','.browsers','.npm-cache'))
print(deck)
