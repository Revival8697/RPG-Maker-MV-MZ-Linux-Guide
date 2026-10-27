#!/usr/bin/env python3
"""
RPG Maker MV/MZ Asset Case-Sensitivity Fixer
Extracts asset references from data/*.json, scans disk case-insensitively,
and performs safe two-step renames to match declared engine casing.
"""

import sys
import os
import json
import glob
from collections import defaultdict, Counter


def main():
    if len(sys.argv) < 2:
        print(f"Usage: {sys.argv[0]} <asset_root> [dry_run: true|false]", file=sys.stderr)
        sys.exit(1)

    asset_root = sys.argv[1]
    dry_run = sys.argv[2].lower() == "true" if len(sys.argv) > 2 else False
    data_dir = os.path.join(asset_root, "data")

    if not os.path.isdir(data_dir):
        print(f"Error: Data directory not found at {data_dir}", file=sys.stderr)
        sys.exit(1)

    print("\n[Phase 1] Parsing game data references...")

    folder_refs = defaultdict(Counter)

    def add_ref(folder, name):
        if name and isinstance(name, str):
            cleaned = name.strip()
            if cleaned:
                folder_refs[folder][cleaned] += 1

    def process_command(cmd):
        code = cmd.get("code")
        params = cmd.get("parameters", [])
        if not isinstance(params, list):
            return

        # 231: Show Picture -> parameters[1]: picture name
        if code == 231 and len(params) > 1 and isinstance(params[1], str):
            add_ref("img/pictures", params[1])
        # 241: Play BGM -> parameters[0]: { name: string }
        elif code == 241 and len(params) > 0 and isinstance(params[0], dict):
            add_ref("audio/bgm", params[0].get("name"))
        # 245: Play BGS -> parameters[0]: { name: string }
        elif code == 245 and len(params) > 0 and isinstance(params[0], dict):
            add_ref("audio/bgs", params[0].get("name"))
        # 249: Play ME -> parameters[0]: { name: string }
        elif code == 249 and len(params) > 0 and isinstance(params[0], dict):
            add_ref("audio/me", params[0].get("name"))
        # 250: Play SE -> parameters[0]: { name: string }
        elif code == 250 and len(params) > 0 and isinstance(params[0], dict):
            add_ref("audio/se", params[0].get("name"))
        # 283: Change Battle Background -> parameters[0]: battleback1, parameters[1]: battleback2
        elif code == 283 and len(params) > 1:
            if isinstance(params[0], str): add_ref("img/battlebacks1", params[0])
            if isinstance(params[1], str): add_ref("img/battlebacks2", params[1])
        # 284: Change Parallax -> parameters[0]: parallax name
        elif code == 284 and len(params) > 0 and isinstance(params[0], str):
            add_ref("img/parallaxes", params[0])
        # 322: Change Actor Graphic -> [1]: character, [3]: face, [5]: battler
        elif code == 322 and len(params) > 5:
            if isinstance(params[1], str): add_ref("img/characters", params[1])
            if isinstance(params[3], str): add_ref("img/faces", params[3])
            if isinstance(params[5], str): add_ref("img/sv_actors", params[5])
        # 323: Change Vehicle Graphic -> [1]: character
        elif code == 323 and len(params) > 1 and isinstance(params[1], str):
            add_ref("img/characters", params[1])
        # 132: Change Battle BGM
        elif code == 132 and len(params) > 0 and isinstance(params[0], dict):
            add_ref("audio/bgm", params[0].get("name"))
        # 133: Change Victory ME
        elif code == 133 and len(params) > 0 and isinstance(params[0], dict):
            add_ref("audio/me", params[0].get("name"))
        # 139: Change Defeat ME
        elif code == 139 and len(params) > 0 and isinstance(params[0], dict):
            add_ref("audio/me", params[0].get("name"))
        # 140: Change Vehicle BGM
        elif code == 140 and len(params) > 1 and isinstance(params[1], dict):
            add_ref("audio/bgm", params[1].get("name"))

    def process_event_list(cmd_list):
        if not isinstance(cmd_list, list):
            return
        for cmd in cmd_list:
            if isinstance(cmd, dict):
                process_command(cmd)

    def safe_load_json(file_path):
        # Handle UTF-8 with optional BOM
        try:
            with open(file_path, "r", encoding="utf-8-sig") as f:
                return json.load(f)
        except Exception as e:
            print(f"  [!] Warning: Failed to parse {os.path.basename(file_path)}: {e}", file=sys.stderr)
            return None

    json_files = glob.glob(os.path.join(data_dir, "*.json"))
    print(f"  Found {len(json_files)} data JSON files in {data_dir}")

    # Add standard implicit engine system assets
    STANDARD_SYSTEM_ASSETS = [
        "Window", "IconSet", "Balloon", "Shadow1", "Shadow2",
        "Damage", "States", "Weapons1", "Weapons2", "Weapons3",
        "ButtonSet", "GameOver", "Loading", "MadeWithMv"
    ]
    for sa in STANDARD_SYSTEM_ASSETS:
        add_ref("img/system", sa)

    # 1. Tilesets.json
    ts_data = safe_load_json(os.path.join(data_dir, "Tilesets.json"))
    if isinstance(ts_data, list):
        for ts in ts_data:
            if isinstance(ts, dict) and "tilesetNames" in ts:
                for tn in ts["tilesetNames"]:
                    add_ref("img/tilesets", tn)

    # 2. System.json
    sys_data = safe_load_json(os.path.join(data_dir, "System.json"))
    if isinstance(sys_data, dict):
        add_ref("img/titles1", sys_data.get("title1Name"))
        add_ref("img/titles2", sys_data.get("title2Name"))
        add_ref("img/battlebacks1", sys_data.get("battleback1Name"))
        add_ref("img/battlebacks2", sys_data.get("battleback2Name"))
        add_ref("img/enemies", sys_data.get("battlerName"))
        add_ref("img/sv_enemies", sys_data.get("battlerName"))
        for snd in sys_data.get("sounds", []):
            if isinstance(snd, dict): add_ref("audio/se", snd.get("name"))
        for k in ["titleBgm", "battleBgm"]:
            if isinstance(sys_data.get(k), dict): add_ref("audio/bgm", sys_data[k].get("name"))
        for k in ["defeatMe", "victoryMe", "gameoverMe"]:
            if isinstance(sys_data.get(k), dict): add_ref("audio/me", sys_data[k].get("name"))
        for vh in ["boat", "ship", "airship"]:
            obj = sys_data.get(vh)
            if isinstance(obj, dict):
                add_ref("img/characters", obj.get("characterName"))
                if isinstance(obj.get("bgm"), dict):
                    add_ref("audio/bgm", obj["bgm"].get("name"))

    # 3. Actors.json
    actors_data = safe_load_json(os.path.join(data_dir, "Actors.json"))
    if isinstance(actors_data, list):
        for a in actors_data:
            if isinstance(a, dict):
                add_ref("img/characters", a.get("characterName"))
                add_ref("img/faces", a.get("faceName"))
                add_ref("img/sv_actors", a.get("battlerName"))

    # 4. Animations.json
    anim_data = safe_load_json(os.path.join(data_dir, "Animations.json"))
    if isinstance(anim_data, list):
        for a in anim_data:
            if isinstance(a, dict):
                add_ref("img/animations", a.get("animation1Name"))
                add_ref("img/animations", a.get("animation2Name"))
                for timing in a.get("timings", []):
                    if isinstance(timing, dict) and isinstance(timing.get("se"), dict):
                        add_ref("audio/se", timing["se"].get("name"))

    # 5. Enemies.json
    enemies_data = safe_load_json(os.path.join(data_dir, "Enemies.json"))
    if isinstance(enemies_data, list):
        for e in enemies_data:
            if isinstance(e, dict):
                add_ref("img/enemies", e.get("battlerName"))
                add_ref("img/sv_enemies", e.get("battlerName"))

    # 6. CommonEvents.json
    ce_data = safe_load_json(os.path.join(data_dir, "CommonEvents.json"))
    if isinstance(ce_data, list):
        for ce in ce_data:
            if isinstance(ce, dict) and "list" in ce:
                process_event_list(ce["list"])

    # 7. Troops.json
    troops_data = safe_load_json(os.path.join(data_dir, "Troops.json"))
    if isinstance(troops_data, list):
        for tr in troops_data:
            if isinstance(tr, dict) and "pages" in tr:
                for p in tr["pages"]:
                    if isinstance(p, dict) and "list" in p:
                        process_event_list(p["list"])

    # 8. Map*.json
    for map_file in glob.glob(os.path.join(data_dir, "Map[0-9]*.json")):
        m_data = safe_load_json(map_file)
        if isinstance(m_data, dict):
            add_ref("img/battlebacks1", m_data.get("battleback1Name"))
            add_ref("img/battlebacks2", m_data.get("battleback2Name"))
            add_ref("img/parallaxes", m_data.get("parallaxName"))
            if isinstance(m_data.get("bgm"), dict): add_ref("audio/bgm", m_data["bgm"].get("name"))
            if isinstance(m_data.get("bgs"), dict): add_ref("audio/bgs", m_data["bgs"].get("name"))
            for ev in m_data.get("events", []):
                if isinstance(ev, dict) and "pages" in ev:
                    for p in ev["pages"]:
                        if isinstance(p, dict):
                            if isinstance(p.get("image"), dict):
                                add_ref("img/characters", p["image"].get("characterName"))
                            if "list" in p:
                                process_event_list(p["list"])

    total_refs = sum(len(counter) for counter in folder_refs.values())
    print(f"  Extracted {total_refs} unique asset declarations across {len(folder_refs)} target folders.")
    for folder in sorted(folder_refs.keys()):
        if folder_refs[folder]:
            print(f"    - {folder}: {len(folder_refs[folder])} references")

    print("\n[Phase 2] Case-insensitive filesystem scan & matching...")

    # Supported extensions (standard + RPG Maker MV/MZ encrypted)
    IMAGE_EXTS = {".png", ".rpgmvp", ".png_"}
    AUDIO_EXTS = {".ogg", ".m4a", ".rpgmvo", ".rpgmvm", ".ogg_", ".m4a_"}
    ALL_VALID_EXTS = IMAGE_EXTS | AUDIO_EXTS

    renamed_files = []
    created_symlinks = []
    total_files_scanned = 0
    total_matches = 0

    pid = os.getpid()
    seq = 0

    for folder, counter in sorted(folder_refs.items()):
        disk_folder = os.path.join(asset_root, folder)
        if not os.path.isdir(disk_folder):
            continue

        # Group references by lowercase stem
        grouped_refs = defaultdict(Counter)
        for ref_name, count in counter.items():
            base = os.path.basename(ref_name)
            stem, _ = os.path.splitext(base)
            if stem:
                grouped_refs[stem.lower()][stem] += count

        # Walk directory recursively
        for root, dirs, files in os.walk(disk_folder):
            for f in files:
                old_path = os.path.join(root, f)

                # Skip symlinks during file renaming to avoid loops or corrupting alias links
                if os.path.islink(old_path):
                    continue

                total_files_scanned += 1
                disk_stem, disk_ext = os.path.splitext(f)
                if disk_ext.lower() not in ALL_VALID_EXTS:
                    continue

                lower_key = disk_stem.lower()
                if lower_key in grouped_refs:
                    total_matches += 1
                    casing_counts = grouped_refs[lower_key]

                    # Determine canonical target casing:
                    # 1. Most frequently referenced casing in game data
                    # 2. Preference for uppercase/capitalized characters on ties
                    # 3. Alphabetical order for deterministic behavior
                    canonical_stem = sorted(
                        casing_counts.keys(),
                        key=lambda s: (-casing_counts[s], -sum(1 for c in s if c.isupper()), s)
                    )[0]

                    target_filename = canonical_stem + disk_ext
                    new_path = os.path.join(root, target_filename)

                    # Check if disk casing differs from declared canonical casing
                    if f != target_filename:
                        seq += 1
                        tmp_filename = f"{f}.tmp_case_{pid}_{seq}"
                        tmp_path = os.path.join(root, tmp_filename)

                        rel_old = os.path.relpath(old_path, asset_root)
                        rel_new = os.path.relpath(new_path, asset_root)

                        if not dry_run:
                            # Two-step rename: file -> file.tmp_case -> TargetCase
                            try:
                                os.rename(old_path, tmp_path)
                                os.rename(tmp_path, new_path)
                                renamed_files.append((rel_old, rel_new))
                                print(f"  [RENAME] {rel_old} -> {rel_new}")
                            except Exception as e:
                                print(f"  [!] Error renaming {rel_old} to {rel_new}: {e}", file=sys.stderr)
                                # Attempt recovery if in tmp state
                                if os.path.exists(tmp_path) and not os.path.exists(old_path):
                                    os.rename(tmp_path, old_path)
                        else:
                            renamed_files.append((rel_old, rel_new))
                            print(f"  [DRY-RUN RENAME] {rel_old} -> {rel_new}")

                    # If alternate casing was also declared, create a case-alias symlink
                    for declared_casing in casing_counts:
                        if declared_casing != canonical_stem:
                            alias_filename = declared_casing + disk_ext
                            alias_path = os.path.join(root, alias_filename)
                            rel_alias = os.path.relpath(alias_path, asset_root)

                            # Check if alias already correctly points to target_filename
                            already_valid_link = False
                            if os.path.islink(alias_path):
                                try:
                                    if os.readlink(alias_path) == target_filename:
                                        already_valid_link = True
                                except OSError:
                                    pass

                            if not already_valid_link:
                                if not dry_run:
                                    try:
                                        if os.path.islink(alias_path) or os.path.exists(alias_path):
                                            os.unlink(alias_path)
                                        os.symlink(target_filename, alias_path)
                                        created_symlinks.append((rel_alias, target_filename))
                                        print(f"  [SYMLINK ALIAS] {rel_alias} -> {target_filename}")
                                    except Exception as e:
                                        print(f"  [!] Error creating symlink {rel_alias}: {e}", file=sys.stderr)
                                else:
                                    created_symlinks.append((rel_alias, target_filename))
                                    print(f"  [DRY-RUN SYMLINK] {rel_alias} -> {target_filename}")

    print("\n============================================================")
    print(" Execution Summary")
    print("============================================================")
    print(f"Total files scanned:       {total_files_scanned}")
    print(f"Engine references matched: {total_matches}")
    print(f"Files renamed:             {len(renamed_files)}")
    print(f"Case-alias symlinks:       {len(created_symlinks)}")

    if renamed_files:
        print("\nModified Files List:")
        for old_name, new_name in renamed_files:
            print(f"  - {old_name}  ==>  {new_name}")

    if created_symlinks:
        print("\nCreated Symlinks List:")
        for alias_name, target in created_symlinks:
            print(f"  - {alias_name}  -->  {target}")

    # Verification phase
    print("\n[Phase 3] Verification & Validation...")
    verification_errors = 0

    for folder, counter in sorted(folder_refs.items()):
        disk_folder = os.path.join(asset_root, folder)
        if not os.path.isdir(disk_folder):
            continue

        for ref_name in counter:
            base = os.path.basename(ref_name)
            stem, _ = os.path.splitext(base)
            if not stem:
                continue

            # Look for exact matching file on disk with valid extension
            found_exact = False
            for ext in ALL_VALID_EXTS:
                check_file = os.path.join(disk_folder, stem + ext)
                if os.path.exists(check_file):
                    found_exact = True
                    break

            if not found_exact:
                # Check if it exists with another casing (which would mean a missed fix)
                found_ci = False
                for f in os.listdir(disk_folder):
                    f_stem, _ = os.path.splitext(f)
                    if f_stem.lower() == stem.lower():
                        found_ci = True
                        break
                if found_ci and not dry_run:
                    print(f"  [!] Verification Failure: '{ref_name}' exists in '{folder}' but casing still mismatches!", file=sys.stderr)
                    verification_errors += 1

    if verification_errors == 0:
        print("  [✓] Verification successful: All declared asset references match disk casing!")
        print("  [✓] Missing-asset loading errors (such as img/tilesets/Futuristic_A5.png) are resolved.")
    else:
        print(f"  [✗] Verification failed with {verification_errors} errors.", file=sys.stderr)
        sys.exit(1)


if __name__ == "__main__":
    main()
