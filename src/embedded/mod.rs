pub static LAUNCHER_GAME_SH: &str = include_str!("../../assets/launcher/Game.sh");

pub static MV_PIXI_JS: &[u8] = include_bytes!("../../assets/mv/libs/pixi.js");
pub static MZ_PIXI_JS: &[u8] = include_bytes!("../../assets/mz/libs/pixi.js");

pub static MV_CORE_FILES: &[(&str, &[u8])] = &[
    ("rpg_core.js", include_bytes!("../../assets/mv/rpg_core.js")),
    (
        "rpg_managers.js",
        include_bytes!("../../assets/mv/rpg_managers.js"),
    ),
    (
        "rpg_objects.js",
        include_bytes!("../../assets/mv/rpg_objects.js"),
    ),
    (
        "rpg_scenes.js",
        include_bytes!("../../assets/mv/rpg_scenes.js"),
    ),
    (
        "rpg_sprites.js",
        include_bytes!("../../assets/mv/rpg_sprites.js"),
    ),
    (
        "rpg_windows.js",
        include_bytes!("../../assets/mv/rpg_windows.js"),
    ),
];
