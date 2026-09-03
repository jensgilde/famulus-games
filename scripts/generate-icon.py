#!/usr/bin/env python3
"""Famulus Games – App-Icon-Generator (Motiv: Gamecontroller).
Phoenix-Stil via gemeinsamer Basis in famulus-icons/. Ersetzt "FG"
durch ein Controller-Symbol (passend zu Famulus Games).
"""
import os, sys
sys.path.insert(0, os.path.expanduser("~/KI Agenten/famulus-icons"))
from famulus_icon import icon, schreibe
from motive import motiv_controller

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
ICONS = os.path.join(ROOT, "swift-app", "FamulusGames", "Assets.xcassets",
                     "AppIcon.appiconset")


def main():
    ico = icon(motiv_controller)
    n = schreibe(ico, ICONS)
    print(f"Famulus-Games-Icon (Controller) geschrieben ({n} Größen) nach {ICONS}")


if __name__ == "__main__":
    main()