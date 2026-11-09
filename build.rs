#[path = "build-support/design_tokens.rs"]
mod design_tokens;

fn main() {
    println!("cargo:rerun-if-changed=resources/design-tokens.json");
    println!("cargo:rerun-if-changed=build-support/design_tokens.rs");
    let source =
        std::fs::read_to_string("resources/design-tokens.json").expect("design tokens file");
    let generated = design_tokens::generate(&source).expect("invalid design tokens");
    std::fs::write(
        std::path::Path::new(&std::env::var_os("OUT_DIR").expect("OUT_DIR"))
            .join("design_tokens.rs"),
        generated,
    )
    .expect("write design tokens");
    println!("cargo:rerun-if-changed=resources/aetheris.rc");
    println!("cargo:rerun-if-changed=resources/aetheris.ico");
    println!("cargo:rerun-if-changed=resources/aetheris.manifest");
    println!("cargo:rerun-if-changed=resources/companion.rc");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // GPUI already embeds the application's manifest. The independent
        // companion and update helper need Common Controls v6 for task dialogs.
        embed_resource::compile_for(
            "resources/companion.rc",
            ["aetheris-companion", "aetheris-update-helper"],
            embed_resource::ParamsIncludeDirs(["resources"]),
        )
        .manifest_required()
        .expect("impossible d’intégrer les ressources du compagnon Windows");
        if std::env::var_os("CARGO_FEATURE_GUI").is_some() {
            embed_resource::compile_for(
                "resources/aetheris.rc",
                ["aetheris"],
                embed_resource::ParamsIncludeDirs(["resources"]),
            )
            .manifest_required()
            .expect("impossible d’intégrer l’icône Windows d’Aetheris");
        }
    }
}
