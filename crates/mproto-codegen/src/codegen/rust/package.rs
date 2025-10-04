use std::io::Write;
use std::path::Path;

use genco::prelude::*;

use crate::{ast::TypeDef, codegen::CodegenCx, Database, Module};

const CARGO_TOML: &'static str = include_str!("templates/cargo.toml");

pub fn rust_package_gen(
    root_dir: impl AsRef<Path>,
    pkg_name: &str,
    type_defs: &[TypeDef],
) -> std::io::Result<()> {
    let local_module = Module::from_type_defs(type_defs.into());
    let db = Database::new(local_module);

    let pkg_root = root_dir.as_ref().join(pkg_name).join("rust");
    let src_dir = pkg_root.join("src");

    std::fs::create_dir_all(&pkg_root)?;
    std::fs::create_dir_all(&src_dir)?;

    // Write Cargo.toml
    let mut cargo_toml_file = std::fs::File::create(pkg_root.join("Cargo.toml"))?;
    cargo_toml_file.write_all(CARGO_TOML.replace("PKG_NAME", pkg_name).as_bytes())?;

    // Write lib.rs
    rust_module_gen(&db, src_dir.join("lib.rs"), type_defs, true)?;

    Ok(())
}

pub fn rust_module_gen(
    db: &Database,
    path: impl AsRef<Path>,
    type_defs: &[TypeDef],
    is_crate: bool,
) -> std::io::Result<()> {
    // Write lib.rs
    let fmt = genco::fmt::Config::from_lang::<genco::lang::Rust>()
        .with_indentation(genco::fmt::Indentation::Space(4));
    let config = genco::lang::rust::Config::default();
    let mut lib_rs_file = std::fs::File::create(path)?;

    if is_crate {
        lib_rs_file.write_all(b"#![cfg_attr(not(feature = \"std\"), no_std)]\n\n")?;
        lib_rs_file.write_all(
            b"#[cfg(all(not(feature = \"std\"), feature = \"alloc\"))]\nextern crate alloc;\n\n",
        )?;
    }

    let mut w = genco::fmt::IoWriter::new(lib_rs_file);
    let mut tokens = genco::lang::rust::Tokens::new();

    let codegen_cx = CodegenCx::new(db, None, is_crate);

    for type_def in type_defs {
        let type_tokens = crate::codegen::rust::rust_type_def(&codegen_cx, type_def);
        tokens = quote! {
            $tokens

            $type_tokens
        };
    }

    tokens
        .format_file(&mut w.as_formatter(&fmt), &config)
        .expect("format rust file");

    Ok(())
}
