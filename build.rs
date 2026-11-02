fn main() {
    println!("cargo:rerun-if-changed=resources/aetheris.rc");
    println!("cargo:rerun-if-changed=resources/aetheris.ico");
    println!("cargo:rerun-if-changed=resources/aetheris.manifest");
    println!("cargo:rerun-if-changed=resources/companion.rc");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        // GPUI already embeds the application's manifest. The independent
        // companion needs its own Common Controls v6 manifest for task dialogs.
        embed_resource::compile_for(
            "resources/companion.rc",
            ["aetheris-companion"],
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
