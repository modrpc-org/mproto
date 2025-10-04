use std::fs;
use std::io::Write;
use std::path::Path;

use genco::{self, quote_in};

use crate::{ast::TypeDef, codegen, Database, Module};

const PACKAGE_JSON: &'static str = include_str!("templates/package.json");
const TSCONFIG_JSON: &'static str = include_str!("templates/tsconfig.json");

pub fn js_package_gen(
    root_dir: impl AsRef<Path>,
    pkg_name: &str,
    type_defs: &[TypeDef],
) -> std::io::Result<()> {
    let local_module = Module::from_type_defs(type_defs.into());
    let db = Database::new(local_module);

    let pkg_root = root_dir.as_ref().join(pkg_name).join("typescript");
    let src_dir = pkg_root.join("src");

    fs::create_dir_all(&pkg_root)?;
    fs::create_dir_all(&src_dir)?;

    // Write package.json
    let mut package_json_file = fs::File::create(pkg_root.join("package.json"))?;
    package_json_file.write_all(PACKAGE_JSON.replace("PKG_NAME", pkg_name).as_bytes())?;

    // Write package.json
    let mut tsconfig_json_file = fs::File::create(pkg_root.join("tsconfig.json"))?;
    tsconfig_json_file.write_all(TSCONFIG_JSON.as_bytes())?;

    // Write index.ts
    let fmt = genco::fmt::Config::from_lang::<genco::lang::JavaScript>()
        .with_indentation(genco::fmt::Indentation::Space(4));
    let config = genco::lang::js::Config::default();
    let index_file = fs::File::create(src_dir.join("index.ts"))?;
    let mut w = genco::fmt::IoWriter::new(index_file);
    let mut tokens = genco::lang::js::Tokens::new();

    let codegen_cx = codegen::CodegenCx::new(&db, None, true);

    for type_def in type_defs {
        let struct_tokens = codegen::js::js_type_def(&codegen_cx, type_def);
        quote_in! { tokens => $struct_tokens$("\n\n") };
    }

    tokens
        .format_file(&mut w.as_formatter(&fmt), &config)
        .expect("format js struct");

    Ok(())
}

pub fn js_module_gen(path: impl AsRef<Path>, type_defs: &[TypeDef]) -> std::io::Result<()> {
    let local_module = Module::from_type_defs(type_defs.into());
    let db = Database::new(local_module);

    let fmt = genco::fmt::Config::from_lang::<genco::lang::Rust>()
        .with_indentation(genco::fmt::Indentation::Space(4));
    let config = genco::lang::js::Config::default();
    let proto_ts_file = fs::File::create(path)?;

    let mut w = genco::fmt::IoWriter::new(proto_ts_file);
    let mut tokens = genco::lang::js::Tokens::new();

    let codegen_cx = codegen::CodegenCx::new(&db, None, false);

    for type_def in type_defs {
        let type_tokens = codegen::js::js_type_def(&codegen_cx, type_def);
        quote_in! { tokens => $type_tokens$("\n\n") };
    }

    tokens
        .format_file(&mut w.as_formatter(&fmt), &config)
        .expect("format typescript file");

    Ok(())
}
