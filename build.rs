fn main() {
    println!("cargo:rerun-if-changed=resources/aetheris.rc");
    println!("cargo:rerun-if-changed=resources/aetheris.ico");

    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var_os("CARGO_FEATURE_GUI").is_some()
    {
        embed_resource::compile(
            "resources/aetheris.rc",
            embed_resource::ParamsIncludeDirs(["resources"]),
        )
        .manifest_required()
        .expect("impossible d’intégrer l’icône Windows d’Aetheris");
    }
}
