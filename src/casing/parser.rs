use regex::Regex;
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;
use std::sync::OnceLock;

pub const STANDARD_SYSTEM_ASSETS: &[&str] = &[
    "Window",
    "IconSet",
    "Balloon",
    "Shadow1",
    "Shadow2",
    "Damage",
    "States",
    "Weapons1",
    "Weapons2",
    "Weapons3",
    "ButtonSet",
    "GameOver",
    "Loading",
    "MadeWithMv",
];

pub fn read_json_value(path: &Path) -> Option<Value> {
    let content = fs::read_to_string(path).ok()?;
    let clean = content.trim_start_matches('\u{feff}');
    serde_json::from_str(clean).ok()
}

pub fn extract_all_references(asset_root: &Path) -> BTreeMap<String, HashMap<String, usize>> {
    let mut folder_refs: BTreeMap<String, HashMap<String, usize>> = BTreeMap::new();

    let mut add_ref = |folder: &str, name: &str| {
        let cleaned = name.trim();
        if !cleaned.is_empty() {
            let map = folder_refs.entry(folder.to_string()).or_default();
            *map.entry(cleaned.to_string()).or_insert(0) += 1;
        }
    };

    // Standard system assets
    for sa in STANDARD_SYSTEM_ASSETS {
        add_ref("img/system", sa);
    }

    // Discover data directories: check asset_root/data, asset_root/dataEx, and any data* directories
    let mut data_dirs = Vec::new();
    let candidates = [
        asset_root.join("data"),
        asset_root.join("dataEx"),
        asset_root.join("www").join("data"),
        asset_root.join("www").join("dataEx"),
    ];
    for c in &candidates {
        if c.is_dir() && !data_dirs.contains(c) {
            data_dirs.push(c.clone());
        }
    }
    // Also scan for any custom data* directory directly in asset_root
    if let Ok(entries) = fs::read_dir(asset_root) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_dir() {
                if let Some(name) = p.file_name().and_then(|s| s.to_str()) {
                    if name.starts_with("data") && !data_dirs.contains(&p) {
                        data_dirs.push(p);
                    }
                }
            }
        }
    }

    // Process all discovered data directories
    for data_dir in &data_dirs {
        // 1. Tilesets.json
        let tilesets_file = data_dir.join("Tilesets.json");
        if let Some(Value::Array(arr)) = read_json_value(&tilesets_file) {
            for ts in arr {
                if let Some(names) = ts.get("tilesetNames").and_then(|v| v.as_array()) {
                    for tn in names {
                        if let Some(s) = tn.as_str() {
                            add_ref("img/tilesets", s);
                        }
                    }
                }
            }
        }

        // 2. System.json
        let system_file = data_dir.join("System.json");
        if let Some(val) = read_json_value(&system_file) {
            if let Some(s) = val.get("title1Name").and_then(|v| v.as_str()) {
                add_ref("img/titles1", s);
            }
            if let Some(s) = val.get("title2Name").and_then(|v| v.as_str()) {
                add_ref("img/titles2", s);
            }
            if let Some(s) = val.get("battleback1Name").and_then(|v| v.as_str()) {
                add_ref("img/battlebacks1", s);
            }
            if let Some(s) = val.get("battleback2Name").and_then(|v| v.as_str()) {
                add_ref("img/battlebacks2", s);
            }
            if let Some(s) = val.get("battlerName").and_then(|v| v.as_str()) {
                add_ref("img/enemies", s);
                add_ref("img/sv_enemies", s);
            }
            if let Some(sounds) = val.get("sounds").and_then(|v| v.as_array()) {
                for snd in sounds {
                    if let Some(s) = snd.get("name").and_then(|v| v.as_str()) {
                        add_ref("audio/se", s);
                    }
                }
            }
            for k in &["titleBgm", "battleBgm"] {
                if let Some(s) = val
                    .get(*k)
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref("audio/bgm", s);
                }
            }
            for k in &["defeatMe", "victoryMe", "gameoverMe"] {
                if let Some(s) = val
                    .get(*k)
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref("audio/me", s);
                }
            }
            for vh in &["boat", "ship", "airship"] {
                if let Some(obj) = val.get(*vh) {
                    if let Some(s) = obj.get("characterName").and_then(|v| v.as_str()) {
                        add_ref("img/characters", s);
                    }
                    if let Some(s) = obj
                        .get("bgm")
                        .and_then(|b| b.get("name"))
                        .and_then(|v| v.as_str())
                    {
                        add_ref("audio/bgm", s);
                    }
                }
            }
        }

        // 3. Actors.json
        let actors_file = data_dir.join("Actors.json");
        if let Some(Value::Array(arr)) = read_json_value(&actors_file) {
            for a in arr {
                if let Some(s) = a.get("characterName").and_then(|v| v.as_str()) {
                    add_ref("img/characters", s);
                }
                if let Some(s) = a.get("faceName").and_then(|v| v.as_str()) {
                    add_ref("img/faces", s);
                }
                if let Some(s) = a.get("battlerName").and_then(|v| v.as_str()) {
                    add_ref("img/sv_actors", s);
                }
            }
        }

        // 4. Animations.json
        let animations_file = data_dir.join("Animations.json");
        if let Some(Value::Array(arr)) = read_json_value(&animations_file) {
            for a in arr {
                if let Some(s) = a.get("animation1Name").and_then(|v| v.as_str()) {
                    add_ref("img/animations", s);
                }
                if let Some(s) = a.get("animation2Name").and_then(|v| v.as_str()) {
                    add_ref("img/animations", s);
                }
                if let Some(s) = a.get("effectName").and_then(|v| v.as_str()) {
                    add_ref("effects", s);
                }
                if let Some(timings) = a.get("timings").and_then(|v| v.as_array()) {
                    for t in timings {
                        if let Some(s) = t
                            .get("se")
                            .and_then(|se| se.get("name"))
                            .and_then(|v| v.as_str())
                        {
                            add_ref("audio/se", s);
                        }
                    }
                }
            }
        }

        // 5. Enemies.json
        let enemies_file = data_dir.join("Enemies.json");
        if let Some(Value::Array(arr)) = read_json_value(&enemies_file) {
            for e in arr {
                if let Some(s) = e.get("battlerName").and_then(|v| v.as_str()) {
                    add_ref("img/enemies", s);
                    add_ref("img/sv_enemies", s);
                }
            }
        }

        // 6. CommonEvents.json
        let ce_file = data_dir.join("CommonEvents.json");
        if let Some(Value::Array(arr)) = read_json_value(&ce_file) {
            for ce in arr {
                if let Some(list) = ce.get("list") {
                    process_event_list(list, &mut add_ref);
                }
            }
        }

        // 7. Troops.json
        let troops_file = data_dir.join("Troops.json");
        if let Some(Value::Array(arr)) = read_json_value(&troops_file) {
            for tr in arr {
                if let Some(pages) = tr.get("pages").and_then(|v| v.as_array()) {
                    for p in pages {
                        if let Some(list) = p.get("list") {
                            process_event_list(list, &mut add_ref);
                        }
                    }
                }
            }
        }

        // 8. Map*.json and custom json files
        if let Ok(entries) = fs::read_dir(data_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if file_name.starts_with("Map") && file_name.ends_with(".json") {
                        if let Some(m_data) = read_json_value(&path) {
                            if let Some(s) = m_data.get("battleback1Name").and_then(|v| v.as_str())
                            {
                                add_ref("img/battlebacks1", s);
                            }
                            if let Some(s) = m_data.get("battleback2Name").and_then(|v| v.as_str())
                            {
                                add_ref("img/battlebacks2", s);
                            }
                            if let Some(s) = m_data.get("parallaxName").and_then(|v| v.as_str()) {
                                add_ref("img/parallaxes", s);
                            }
                            if let Some(s) = m_data
                                .get("bgm")
                                .and_then(|b| b.get("name"))
                                .and_then(|v| v.as_str())
                            {
                                add_ref("audio/bgm", s);
                            }
                            if let Some(s) = m_data
                                .get("bgs")
                                .and_then(|b| b.get("name"))
                                .and_then(|v| v.as_str())
                            {
                                add_ref("audio/bgs", s);
                            }
                            if let Some(events) = m_data.get("events").and_then(|v| v.as_array()) {
                                for ev in events {
                                    if let Some(pages) = ev.get("pages").and_then(|v| v.as_array())
                                    {
                                        for p in pages {
                                            if let Some(s) = p
                                                .get("image")
                                                .and_then(|img| img.get("characterName"))
                                                .and_then(|v| v.as_str())
                                            {
                                                add_ref("img/characters", s);
                                            }
                                            // Autonomous movement route on event page
                                            if let Some(mr) = p.get("moveRoute") {
                                                process_move_route(mr, &mut add_ref);
                                            }
                                            if let Some(list) = p.get("list") {
                                                process_event_list(list, &mut add_ref);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    } else if file_name.ends_with(".json")
                        && !matches!(
                            file_name,
                            "Tilesets.json"
                                | "System.json"
                                | "Actors.json"
                                | "Animations.json"
                                | "Enemies.json"
                                | "CommonEvents.json"
                                | "Troops.json"
                        )
                    {
                        // Custom plugin JSON file (e.g. TrpParticles.json or plugin data)
                        if let Some(val) = read_json_value(&path) {
                            traverse_json_for_assets(&val, "", &mut add_ref);
                        }
                    }
                }
            }
        }
    }

    // 9. Scan .efkefc Effekseer files in effects/ directory
    let effects_candidates = [
        asset_root.join("effects"),
        asset_root.join("www").join("effects"),
    ];
    for eff_dir in &effects_candidates {
        if eff_dir.is_dir() {
            scan_effects_directory(eff_dir, &mut add_ref);
        }
    }

    // 10. plugins.js
    let plugins_candidates = [
        asset_root.join("js").join("plugins.js"),
        asset_root.join("www").join("js").join("plugins.js"),
    ];

    for p_file in &plugins_candidates {
        if p_file.is_file() {
            extract_plugin_assets(p_file, &mut add_ref);
        }
    }

    folder_refs
}

fn process_move_route(route: &Value, add_ref_fn: &mut dyn FnMut(&str, &str)) {
    if let Some(route_list) = route.get("list").and_then(|v| v.as_array()) {
        for r_cmd in route_list {
            if r_cmd.get("code").and_then(|v| v.as_i64()) == Some(41) {
                // ROUTE_CHANGE_IMAGE
                if let Some(r_params) = r_cmd.get("parameters").and_then(|v| v.as_array()) {
                    if let Some(s) = r_params.first().and_then(|v| v.as_str()) {
                        add_ref_fn("img/characters", s);
                    }
                }
            }
        }
    }
}

fn process_command(cmd: &Value, add_ref_fn: &mut dyn FnMut(&str, &str)) {
    let code = cmd.get("code").and_then(|v| v.as_i64()).unwrap_or(-1);
    let params = cmd.get("parameters").and_then(|v| v.as_array());
    if let Some(p) = params {
        match code {
            205 => {
                // Set Movement Route -> route is p[1]
                if p.len() > 1 {
                    process_move_route(&p[1], add_ref_fn);
                }
            }
            231 => {
                if p.len() > 1 {
                    if let Some(s) = p[1].as_str() {
                        add_ref_fn("img/pictures", s);
                    }
                }
            }
            241 => {
                if let Some(s) = p
                    .first()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref_fn("audio/bgm", s);
                }
            }
            245 => {
                if let Some(s) = p
                    .first()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref_fn("audio/bgs", s);
                }
            }
            249 => {
                if let Some(s) = p
                    .first()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref_fn("audio/me", s);
                }
            }
            250 => {
                if let Some(s) = p
                    .first()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref_fn("audio/se", s);
                }
            }
            261 => {
                if let Some(s) = p.first().and_then(|v| v.as_str()) {
                    add_ref_fn("movies", s);
                }
            }
            283 => {
                if p.len() > 1 {
                    if let Some(s) = p[0].as_str() {
                        add_ref_fn("img/battlebacks1", s);
                    }
                    if let Some(s) = p[1].as_str() {
                        add_ref_fn("img/battlebacks2", s);
                    }
                }
            }
            284 => {
                if let Some(s) = p.first().and_then(|v| v.as_str()) {
                    add_ref_fn("img/parallaxes", s);
                }
            }
            322 => {
                if p.len() > 5 {
                    if let Some(s) = p[1].as_str() {
                        add_ref_fn("img/characters", s);
                    }
                    if let Some(s) = p[3].as_str() {
                        add_ref_fn("img/faces", s);
                    }
                    if let Some(s) = p[5].as_str() {
                        add_ref_fn("img/sv_actors", s);
                    }
                }
            }
            323 => {
                if p.len() > 1 {
                    if let Some(s) = p[1].as_str() {
                        add_ref_fn("img/characters", s);
                    }
                }
            }
            132 => {
                if let Some(s) = p
                    .first()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref_fn("audio/bgm", s);
                }
            }
            133 => {
                if let Some(s) = p
                    .first()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref_fn("audio/me", s);
                }
            }
            139 => {
                if let Some(s) = p
                    .first()
                    .and_then(|o| o.get("name"))
                    .and_then(|v| v.as_str())
                {
                    add_ref_fn("audio/me", s);
                }
            }
            140 => {
                if p.len() > 1 {
                    if let Some(s) = p[1].get("name").and_then(|v| v.as_str()) {
                        add_ref_fn("audio/bgm", s);
                    }
                }
            }
            355 | 655 => {
                // JavaScript Script calls (355 = start line, 655 = continuation line)
                if let Some(script_str) = p.first().and_then(|v| v.as_str()) {
                    extract_script_references(script_str, add_ref_fn);
                }
            }
            356 => {
                // MV Plugin Command: string like "D_TEXT ..."
                if let Some(cmd_str) = p.first().and_then(|v| v.as_str()) {
                    for token in cmd_str.split_whitespace() {
                        add_plugin_val(token, "", add_ref_fn);
                    }
                }
            }
            357 if p.len() > 3 => {
                // MZ Plugin Command: [pluginName, commandName, friendlyName, argsObject]
                traverse_json_for_assets(&p[3], "", add_ref_fn);
            }
            _ => {}
        }
    }
}

fn extract_script_references(script: &str, add_ref_fn: &mut dyn FnMut(&str, &str)) {
    // 1. $gameScreen.showPicture(id, "filename", ...)
    static SHOW_PIC_RE: OnceLock<Regex> = OnceLock::new();
    let show_pic_re = SHOW_PIC_RE.get_or_init(|| {
        Regex::new(r#"\$gameScreen\.showPicture\s*\(\s*[^,]+,\s*["'`]([^"'`]+)["'`]"#).unwrap()
    });
    for cap in show_pic_re.captures_iter(script) {
        if let Some(m) = cap.get(1) {
            add_ref_fn("img/pictures", m.as_str());
        }
    }

    // 2. ImageManager.load*(...)
    static LOAD_METHOD_RE: OnceLock<Regex> = OnceLock::new();
    let load_method_re = LOAD_METHOD_RE.get_or_init(|| {
        Regex::new(r#"ImageManager\.load([A-Za-z0-9]+)\s*\(\s*["'`]([^"'`]+)["'`]"#).unwrap()
    });
    for cap in load_method_re.captures_iter(script) {
        if let (Some(m_type), Some(m_name)) = (cap.get(1), cap.get(2)) {
            let t = m_type.as_str();
            let name = m_name.as_str();
            let folder = match t {
                "Picture" => "img/pictures",
                "Character" => "img/characters",
                "Face" => "img/faces",
                "Parallax" => "img/parallaxes",
                "SvActor" => "img/sv_actors",
                "SvEnemy" => "img/sv_enemies",
                "Tileset" => "img/tilesets",
                "Title1" => "img/titles1",
                "Title2" => "img/titles2",
                "Animation" => "img/animations",
                "Battleback1" => "img/battlebacks1",
                "Battleback2" => "img/battlebacks2",
                _ => continue,
            };
            add_ref_fn(folder, name);
        }
    }

    // 3. ImageManager.loadBitmap("img/subfolder/", "filename")
    static LOAD_BITMAP_RE: OnceLock<Regex> = OnceLock::new();
    let load_bitmap_re = LOAD_BITMAP_RE.get_or_init(|| {
        Regex::new(
            r#"ImageManager\.loadBitmap\s*\(\s*["'`]([^"'`]+)["'`]\s*,\s*["'`]([^"'`]+)["'`]"#,
        )
        .unwrap()
    });
    for cap in load_bitmap_re.captures_iter(script) {
        if let (Some(m_path), Some(m_name)) = (cap.get(1), cap.get(2)) {
            let path_clean = m_path.as_str().trim_matches('/');
            add_ref_fn(path_clean, m_name.as_str());
        }
    }

    // 4. AudioManager.play*(...)
    static AUDIO_PLAY_RE: OnceLock<Regex> = OnceLock::new();
    let audio_play_re = AUDIO_PLAY_RE.get_or_init(|| {
        Regex::new(
            r#"AudioManager\.play(Bgm|Bgs|Me|Se)\s*\(\s*(?:\{[^}]*?name:\s*)?["'`]([^"'`]+)["'`]"#,
        )
        .unwrap()
    });
    for cap in audio_play_re.captures_iter(script) {
        if let (Some(m_type), Some(m_name)) = (cap.get(1), cap.get(2)) {
            let folder = match m_type.as_str() {
                "Bgm" => "audio/bgm",
                "Bgs" => "audio/bgs",
                "Me" => "audio/me",
                "Se" => "audio/se",
                _ => continue,
            };
            add_ref_fn(folder, m_name.as_str());
        }
    }
}

fn process_event_list(cmd_list: &Value, add_ref_fn: &mut dyn FnMut(&str, &str)) {
    if let Some(arr) = cmd_list.as_array() {
        for cmd in arr {
            process_command(cmd, add_ref_fn);
        }
    }
}

fn scan_effects_directory(eff_dir: &Path, add_ref_fn: &mut dyn FnMut(&str, &str)) {
    if let Ok(entries) = fs::read_dir(eff_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("efkefc") {
                if let Ok(bytes) = fs::read(&path) {
                    scan_efkefc_bytes(&bytes, add_ref_fn);
                }
            }
        }
    }
}

fn scan_efkefc_bytes(bytes: &[u8], add_ref_fn: &mut dyn FnMut(&str, &str)) {
    let texture_prefix: &[u8] = b"T\0e\0x\0t\0u\0r\0e\0/\0";
    let model_prefix: &[u8] = b"M\0o\0d\0e\0l\0/\0";

    for (prefix, folder) in [
        (texture_prefix, "effects/Texture"),
        (model_prefix, "effects/Model"),
    ] {
        let mut cursor = 0;
        while let Some(pos) = bytes[cursor..]
            .windows(prefix.len())
            .position(|w| w == prefix)
        {
            let abs_start = cursor + pos + prefix.len();
            let mut u16_chars = Vec::new();
            let mut p = abs_start;
            while p + 1 < bytes.len() {
                let code = u16::from_le_bytes([bytes[p], bytes[p + 1]]);
                if code == 0 || !(32..=126).contains(&code) {
                    break;
                }
                u16_chars.push(code);
                p += 2;
            }
            if !u16_chars.is_empty() {
                if let Ok(s) = String::from_utf16(&u16_chars) {
                    let clean = s.trim();
                    if !clean.is_empty() {
                        let stem = Path::new(clean)
                            .file_stem()
                            .and_then(|st| st.to_str())
                            .unwrap_or(clean);
                        add_ref_fn(folder, stem);
                    }
                }
            }
            cursor = abs_start + 1;
        }
    }
}

fn extract_plugin_assets(plugins_file: &Path, add_ref_fn: &mut dyn FnMut(&str, &str)) {
    let content = match fs::read_to_string(plugins_file) {
        Ok(c) => c,
        Err(_) => return,
    };

    let re = Regex::new(r"(?s)var\s+\$plugins\s*=\s*(\[.*?\])\s*;?\s*$").unwrap();
    let cap = match re.captures(&content) {
        Some(c) => c,
        None => return,
    };

    let json_str = cap.get(1).map(|m| m.as_str()).unwrap_or("");
    let clean_json = json_str.trim_start_matches('\u{feff}');
    let plugins: Value = match serde_json::from_str(clean_json) {
        Ok(v) => v,
        Err(_) => return,
    };

    if let Some(arr) = plugins.as_array() {
        for p in arr {
            if let Some(params) = p.get("parameters") {
                traverse_json_for_assets(params, "", add_ref_fn);
            }
        }
    }
}

pub fn traverse_json_for_assets(
    val: &Value,
    key_hint: &str,
    add_ref_fn: &mut dyn FnMut(&str, &str),
) {
    match val {
        Value::String(s) => {
            let trimmed = s.trim();
            let clean = trimmed.trim_start_matches('\u{feff}');
            if clean.starts_with('{') || clean.starts_with('[') {
                if let Ok(nested) = serde_json::from_str::<Value>(clean) {
                    traverse_json_for_assets(&nested, key_hint, add_ref_fn);
                    return;
                }
            }
            add_plugin_val(clean, key_hint, add_ref_fn);
        }
        Value::Array(arr) => {
            for item in arr {
                traverse_json_for_assets(item, key_hint, add_ref_fn);
            }
        }
        Value::Object(map) => {
            for (k, v) in map {
                traverse_json_for_assets(v, k, add_ref_fn);
            }
        }
        _ => {}
    }
}

fn add_plugin_val(val: &str, key_hint: &str, add_ref_fn: &mut dyn FnMut(&str, &str)) {
    let val_clean = val
        .replace('\\', "/")
        .trim()
        .trim_matches('"')
        .trim_matches('\'')
        .to_string();
    if val_clean.is_empty() || val_clean.len() > 120 || val_clean.contains('\n') {
        return;
    }

    let key_lower = key_hint.to_lowercase();
    if key_lower.contains("picture") || key_lower.contains("pic") {
        add_ref_fn("img/pictures", &val_clean);
    } else if key_lower.contains("bgm") {
        add_ref_fn("audio/bgm", &val_clean);
    } else if key_lower.contains("bgs") {
        add_ref_fn("audio/bgs", &val_clean);
    } else if key_lower.contains("me") {
        add_ref_fn("audio/me", &val_clean);
    } else if key_lower.contains("se") || key_lower.contains("sound") {
        add_ref_fn("audio/se", &val_clean);
    } else if key_lower.contains("character") {
        add_ref_fn("img/characters", &val_clean);
    } else if key_lower.contains("face") {
        add_ref_fn("img/faces", &val_clean);
    } else if key_lower.contains("enemy") {
        add_ref_fn("img/enemies", &val_clean);
    } else if key_lower.contains("actor") {
        add_ref_fn("img/sv_actors", &val_clean);
    } else if key_lower.contains("parallax") {
        add_ref_fn("img/parallaxes", &val_clean);
    } else if key_lower.contains("tileset") {
        add_ref_fn("img/tilesets", &val_clean);
    } else if key_lower.contains("animation") {
        add_ref_fn("img/animations", &val_clean);
    } else if key_lower.contains("font") {
        add_ref_fn("fonts", &val_clean);
    } else if val_clean.contains('/')
        && (val_clean.starts_with("img/")
            || val_clean.starts_with("audio/")
            || val_clean.starts_with("effects/"))
    {
        let parts: Vec<&str> = val_clean.splitn(3, '/').collect();
        if parts.len() >= 2 {
            let folder = format!("{}/{}", parts[0], parts[1]);
            let base = Path::new(&val_clean)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("");
            add_ref_fn(&folder, base);
        }
    }
}
