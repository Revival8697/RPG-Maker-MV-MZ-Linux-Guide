use rpgmaker_core::casing::transaction::execute_fix_casing;
use rpgmaker_core::core::layout::GameLayout;
use rpgmaker_core::core::manifest::read_system_info;
use rpgmaker_core::crypto::{Decrypter, EncryptionKey, FileProcessor, ProcessorOptions};
use rpgmaker_core::libs::{run_update_libs, UpdateMode};
use rpgmaker_core::setup::{run_setup, SetupOptions};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_mv_setup_in_isolated_tempdir() {
    let temp_dir = tempdir().unwrap();
    let game_dir = temp_dir.path().join("game");

    // Create a minimal MV structure
    let www = game_dir.join("www");
    let data = www.join("data");
    let js_libs = www.join("js").join("libs");
    fs::create_dir_all(&data).unwrap();
    fs::create_dir_all(&js_libs).unwrap();

    fs::write(www.join("index.html"), "<html></html>").unwrap();
    fs::write(
        game_dir.join("package.json"),
        r#"{"name":"test-game","main":"www/index.html"}"#,
    )
    .unwrap();
    fs::write(data.join("System.json"), r#"{"gameTitle":"My Test Game"}"#).unwrap();

    // Create dummy Windows files that should be cleaned up
    fs::write(game_dir.join("Game.exe"), "dummy exe").unwrap();
    fs::write(game_dir.join("ffmpeg.dll"), "dummy dll").unwrap();

    let opts = SetupOptions {
        non_interactive: true,
        custom_id: Some("my-test-game".to_string()),
        install_desktop: false,
        clean_junk: false,
    };
    run_setup(&game_dir, opts).unwrap();

    // Verify Windows files purged
    assert!(!game_dir.join("Game.exe").exists());
    assert!(!game_dir.join("ffmpeg.dll").exists());

    // Verify launcher created
    let launcher = game_dir.join("Game");
    assert!(launcher.is_file());
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let perms = fs::metadata(&launcher).unwrap().permissions();
        assert_eq!(perms.mode() & 0o111, 0o111); // Executable
    }

    // Verify desktop entry created
    let desktop = game_dir.join("my-test-game.desktop");
    assert!(desktop.is_file());
    let desktop_content = fs::read_to_string(desktop).unwrap();
    assert!(desktop_content.contains("Name=My Test Game"));
}

#[test]
fn test_mv_casing_and_decryption_slice() {
    let temp_dir = tempdir().unwrap();
    let slice_dir = temp_dir.path().join("mv_slice");
    let www = slice_dir.join("www");
    let data_dir = www.join("data");
    let tilesets_dir = www.join("img").join("tilesets");

    fs::create_dir_all(&data_dir).unwrap();
    fs::create_dir_all(&tilesets_dir).unwrap();
    fs::write(www.join("index.html"), "<html></html>").unwrap();

    // Write System.json and Tilesets.json with canonical reference "Futuristic_A3"
    let system_json = serde_json::json!({
        "gameTitle": "Test MV Game",
        "hasEncryptedImages": true,
        "hasEncryptedAudio": false,
        "encryptionKey": "d41d8cd98f00b204e9800998ecf8427e"
    });
    fs::write(data_dir.join("System.json"), system_json.to_string()).unwrap();

    let tilesets_json = serde_json::json!([
        null,
        {
            "id": 1,
            "name": "World",
            "tilesetNames": ["Futuristic_A3", "", "", "", ""]
        }
    ]);
    fs::write(data_dir.join("Tilesets.json"), tilesets_json.to_string()).unwrap();

    // Create encrypted mismatched tileset: futuristic_a3.rpgmvp
    let test_key = EncryptionKey::from_hex("d41d8cd98f00b204e9800998ecf8427e").unwrap();
    let encrypter = Decrypter::new(Some(test_key));
    let mut raw_png = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR\x00\x00\x00\x01\x00\x00\x00\x01\x08\x06\x00\x00\x00\x1f\x15c4".to_vec();
    raw_png.extend_from_slice(b"fake png payload data for tileset");
    let encrypted = encrypter.encrypt_bytes(&raw_png).unwrap();
    fs::write(tilesets_dir.join("futuristic_a3.rpgmvp"), encrypted).unwrap();

    // 1. Decrypt assets first
    let sys_info = read_system_info(GameLayout::Mv, &slice_dir).unwrap();
    let key = EncryptionKey::from_hex(&sys_info.encryption_key.unwrap()).unwrap();
    let engine = Decrypter::new(Some(key));
    let proc_opts = ProcessorOptions {
        output_dir: None,
        in_place: true,
        overwrite: true,
        restore_images: false,
        worker_threads: None,
        progress_callback: None,
    };
    let processor = FileProcessor::new(&engine, proc_opts);
    let summary = processor.decrypt_all(&slice_dir).unwrap();

    assert_eq!(summary.files_processed, 1);
    fs::remove_file(tilesets_dir.join("futuristic_a3.rpgmvp")).unwrap();
    assert!(tilesets_dir.join("futuristic_a3.png").is_file());

    // 2. Fix casing on decrypted plaintext assets
    let report = execute_fix_casing(&www, false, true).unwrap();
    assert_eq!(report.files_renamed, 1);
    // Target was renamed to canonical Futuristic_A3.png
    let canonical_png = tilesets_dir.join("Futuristic_A3.png");
    assert!(canonical_png.is_file());
    assert!(!canonical_png.is_symlink());
    // Symlink alias futuristic_a3.png was created for compatibility
    let alias_png = tilesets_dir.join("futuristic_a3.png");
    assert!(alias_png.is_symlink());

    // Verify valid PNG magic header
    let header = fs::read(&canonical_png).unwrap();
    assert_eq!(&header[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn test_update_libs_embedded() {
    let temp_dir = tempdir().unwrap();
    let game_dir = temp_dir.path().join("mz_game");
    let js_libs = game_dir.join("js").join("libs");
    fs::create_dir_all(&js_libs).unwrap();
    fs::write(game_dir.join("index.html"), "<html></html>").unwrap();

    // Write a dummy older pixi v5
    fs::write(js_libs.join("pixi.js"), "// pixi.js - v5.0.0").unwrap();

    run_update_libs(&game_dir, UpdateMode::PixiOnly, false).unwrap();

    let updated = fs::read_to_string(js_libs.join("pixi.js")).unwrap();
    assert!(updated.contains("v5.3.12"));
}

#[test]
fn test_utf8_bom_json_parsing() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();
    let data_dir = root.join("data");
    fs::create_dir_all(&data_dir).unwrap();

    // Write a Map001.json with a UTF-8 BOM prefix (\xEF\xBB\xBF)
    let mut bom_content = vec![0xEF, 0xBB, 0xBF];
    bom_content.extend_from_slice(br#"{"battleback1Name":"ForestBack","events":[{"pages":[{"image":{"characterName":"HeroBOM"},"list":[]}]}]}"#);
    fs::write(data_dir.join("Map001.json"), bom_content).unwrap();

    let refs = rpgmaker_core::casing::parser::extract_all_references(root);
    let bb = refs
        .get("img/battlebacks1")
        .expect("battlebacks1 should be found despite BOM");
    assert!(bb.contains_key("ForestBack"));
    let chars = refs
        .get("img/characters")
        .expect("characters should be found despite BOM");
    assert!(chars.contains_key("HeroBOM"));
}

#[test]
fn test_nested_plugin_json_and_script_commands() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();
    let data_dir = root.join("data");
    let js_dir = root.join("js");
    fs::create_dir_all(&data_dir).unwrap();
    fs::create_dir_all(&js_dir).unwrap();

    // Map JSON with Script commands (355/655), MZ Plugin Command (357), and autonomous moveRoute (code 41)
    let map_json = serde_json::json!({
        "events": [{
            "pages": [
                {
                    "moveRoute": {
                        "list": [
                            {"code": 41, "parameters": ["AutoMoveChar"]}
                        ]
                    },
                    "list": [
                        {"code": 355, "parameters": ["$gameScreen.showPicture(1, 'ScriptPic1', 0, 0, 0, 100, 100, 255, 0);"]},
                        {"code": 655, "parameters": ["ImageManager.loadCharacter('ScriptChar1');"]},
                        {"code": 655, "parameters": ["AudioManager.playBgm({name: 'ScriptBgm1', volume: 90});"]},
                        {"code": 357, "parameters": ["MyPlugin", "cmd", "desc", {
                            "pictureName": "MzPluginPic",
                            "bgmTrack": "MzPluginBgm"
                        }]}
                    ]
                }
            ]
        }]
    });
    fs::write(data_dir.join("Map002.json"), map_json.to_string()).unwrap();

    // plugins.js with multi-level nested serialized JSON strings
    let plugins_js = r#"
var $plugins = [
    {
        "status": true,
        "description": "",
        "parameters": {
            "Config": "{\"pictureList\":[\"NestedPic1\",\"NestedPic2\"],\"SubObj\":{\"bgmName\":\"NestedBgm\"}}"
        }
    }
];
"#;
    fs::write(js_dir.join("plugins.js"), plugins_js).unwrap();

    let refs = rpgmaker_core::casing::parser::extract_all_references(root);

    // Verify script references
    let pics = refs
        .get("img/pictures")
        .expect("img/pictures references found");
    assert!(pics.contains_key("ScriptPic1"));
    assert!(pics.contains_key("MzPluginPic"));
    assert!(pics.contains_key("NestedPic1"));
    assert!(pics.contains_key("NestedPic2"));

    let chars = refs
        .get("img/characters")
        .expect("img/characters references found");
    assert!(chars.contains_key("ScriptChar1"));
    assert!(chars.contains_key("AutoMoveChar"));

    let bgms = refs.get("audio/bgm").expect("audio/bgm references found");
    assert!(bgms.contains_key("ScriptBgm1"));
    assert!(bgms.contains_key("MzPluginBgm"));
    assert!(bgms.contains_key("NestedBgm"));
}

#[test]
fn test_symlink_retargeting_during_decryption() {
    let temp_dir = tempdir().unwrap();
    let game_dir = temp_dir.path().join("game");
    let img_dir = game_dir.join("img").join("pictures");
    fs::create_dir_all(&img_dir).unwrap();

    // Create a dummy encrypted file (16-byte fake header + PNG payload)
    let enc_path = img_dir.join("Target.png_");
    let mut fake_enc = vec![
        0x52, 0x50, 0x47, 0x4D, 0x56, 0x00, 0x00, 0x00, 0x00, 0x03, 0x01, 0x00, 0x00, 0x00, 0x00,
        0x00,
    ];
    fake_enc.extend_from_slice(b"fake encrypted payload");
    fs::write(&enc_path, &fake_enc).unwrap();

    // Create a symlink alias target.png_ -> Target.png_
    let link_path = img_dir.join("target.png_");
    #[cfg(unix)]
    std::os::unix::fs::symlink("Target.png_", &link_path).unwrap();

    // Simulate decryption in-place: Target.png created
    fs::write(img_dir.join("Target.png"), b"decrypted png data").unwrap();

    // Retarget symlinks as executed in execute_decrypt
    for entry in walkdir::WalkDir::new(&game_dir)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if entry.path_is_symlink() {
            let p = entry.path();
            if let Ok(target) = fs::read_link(p) {
                if let Some(target_ext) = target.extension().and_then(|s| s.to_str()) {
                    if rpgmaker_core::crypto::is_encrypted_extension(target_ext) {
                        let new_target = rpgmaker_core::crypto::decrypted_path(&target);
                        let new_link_path =
                            if let Some(link_ext) = p.extension().and_then(|s| s.to_str()) {
                                if rpgmaker_core::crypto::is_encrypted_extension(link_ext) {
                                    rpgmaker_core::crypto::decrypted_path(p)
                                } else {
                                    p.to_path_buf()
                                }
                            } else {
                                p.to_path_buf()
                            };

                        let _ = fs::remove_file(p);
                        #[cfg(unix)]
                        std::os::unix::fs::symlink(&new_target, &new_link_path).unwrap();
                    }
                }
            }
        }
    }

    // Purge original encrypted files
    let _ = fs::remove_file(&enc_path);

    // Verify retargeted symlink target.png -> Target.png
    let retargeted = img_dir.join("target.png");
    assert!(
        retargeted.is_symlink(),
        "Symlink should be renamed to target.png and preserved"
    );
    #[cfg(unix)]
    {
        let dest = fs::read_link(&retargeted).unwrap();
        assert_eq!(dest, std::path::Path::new("Target.png"));
    }
}

#[test]
fn test_casing_duplicate_deduplication() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();
    let data_dir = root.join("data");
    let pic_dir = root.join("img").join("pictures");
    fs::create_dir_all(&data_dir).unwrap();
    fs::create_dir_all(&pic_dir).unwrap();

    // Reference calls for "ItemIcon"
    let map = serde_json::json!({
        "events": [{
            "pages": [{
                "list": [
                    {"code": 231, "parameters": [1, "ItemIcon"]}
                ]
            }]
        }]
    });
    fs::write(data_dir.join("Map001.json"), map.to_string()).unwrap();

    // On disk, two files differing only in casing exist: itemicon.png and ItemIcon.png
    // with identical byte contents!
    let content = b"Identical PNG bytes";
    fs::write(pic_dir.join("itemicon.png"), content).unwrap();
    fs::write(pic_dir.join("ItemIcon.png"), content).unwrap();

    // Run casing fixer
    let report = execute_fix_casing(root, false, true).unwrap();
    assert_eq!(report.symlinks_created, 1);

    // itemicon.png should now be a symlink to ItemIcon.png
    assert!(pic_dir.join("ItemIcon.png").is_file());
    assert!(!pic_dir.join("ItemIcon.png").is_symlink());
    assert!(pic_dir.join("itemicon.png").is_symlink());
}

#[test]
fn test_developer_junk_cleaning() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();
    let chars = root.join("img").join("characters");
    fs::create_dir_all(&chars).unwrap();

    fs::write(chars.join("hero.clip"), "csp junk").unwrap();
    fs::write(chars.join("hero.bak"), "bak junk").unwrap();
    fs::write(chars.join("map.tmx"), "tmx junk").unwrap();
    fs::write(root.join("translation.trans"), "trans junk").unwrap();

    // CSFCHUNK extensionless file
    let mut csf_bytes = b"CSFCHUNK".to_vec();
    csf_bytes.extend_from_slice(b"some raw chunk bytes");
    fs::write(chars.join("Hero_0"), csf_bytes).unwrap();

    // Legitimate asset
    fs::write(chars.join("hero.png"), "valid png").unwrap();

    let junk = rpgmaker_core::setup::cleaner::clean_developer_junk(root, false).unwrap();
    assert_eq!(junk.len(), 5);

    assert!(!chars.join("hero.clip").exists());
    assert!(!chars.join("hero.bak").exists());
    assert!(!chars.join("map.tmx").exists());
    assert!(!root.join("translation.trans").exists());
    assert!(!chars.join("Hero_0").exists());
    assert!(chars.join("hero.png").is_file());
}

#[test]
fn test_visible_backup_directory() {
    let temp_dir = tempdir().unwrap();
    let root = temp_dir.path();
    let data_dir = root.join("data");
    let js_dir = root.join("js");
    fs::create_dir_all(&data_dir).unwrap();
    fs::create_dir_all(&js_dir).unwrap();

    fs::write(data_dir.join("System.json"), r#"{"gameTitle":"Test Game"}"#).unwrap();
    fs::write(js_dir.join("main.js"), "console.log('test');").unwrap();

    // 1. Resolve backup dir: verify visible folder 'backup' is used, not '.backup'
    let backup_dir = rpgmaker_core::core::backup::resolve_backup_dir(root, "libs", None);
    assert_eq!(backup_dir, root.join("backup").join("libs"));
    assert!(!root.join(".backup").exists());

    // 2. Execute safe backup
    rpgmaker_core::core::backup::safe_backup(&js_dir, &backup_dir).unwrap();

    // 3. Verify backup is created inside visible 'backup/libs'
    assert!(root.join("backup").is_dir());
    assert!(backup_dir.is_dir());
    assert!(backup_dir.join("main.js").is_file());
    assert_eq!(
        fs::read_to_string(backup_dir.join("main.js")).unwrap(),
        "console.log('test');"
    );
    assert!(!root.join(".backup").exists());

    // 4. Verify that running safe_backup on root does not recursively copy 'backup' into 'backup'
    let full_backup_dir = rpgmaker_core::core::backup::resolve_backup_dir(root, "full", None);
    rpgmaker_core::core::backup::safe_backup(root, &full_backup_dir).unwrap();
    assert!(full_backup_dir.join("data").join("System.json").is_file());
    assert!(!full_backup_dir.join("backup").exists());
}
