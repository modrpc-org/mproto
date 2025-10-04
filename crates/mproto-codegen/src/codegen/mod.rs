use genco::prelude::*;

use crate::{ast::QualifiedIdentifier, Database};

pub use codegen_cx::{
    type_uses_param, type_uses_type_param, CodegenCx, ResolvedType, TypeParamBinding,
    TypeParamBindings,
};
pub(crate) use type_base_len::{
    enum_base_len, enum_variant_base_len, struct_base_len, type_base_len, TypeBaseLen,
};

mod codegen_cx;
pub mod js;
pub mod name_util;
pub mod rust;
mod type_base_len;

pub trait MprotoLang {
    type GencoLang: genco::lang::Lang;

    fn associated_constant(type_name: &str, constant: &str) -> Tokens<Self::GencoLang>;

    // mproto specific concepts
    fn type_param_base_len(type_name: &str) -> Tokens<Self::GencoLang>;
    fn const_fn_max() -> Tokens<Self::GencoLang>;
    fn import_qualified(
        db: &Database,
        local_def_source: Option<&str>,
        qualified_identifier: &QualifiedIdentifier,
    ) -> Tokens<Self::GencoLang>;
}

#[derive(Debug, Eq, PartialEq)]
pub enum MprotoJs {}
#[derive(Debug, Eq, PartialEq)]
pub enum MprotoRust {}

impl MprotoLang for MprotoJs {
    type GencoLang = genco::lang::JavaScript;

    fn associated_constant(type_name: &str, constant: &str) -> Tokens<Self::GencoLang> {
        quote! { $type_name.$constant }
    }

    fn type_param_base_len(type_name: &str) -> Tokens<Self::GencoLang> {
        quote! { this.$(type_name)Encoder.baseLength() }
    }

    fn const_fn_max() -> Tokens<Self::GencoLang> {
        quote! { Math.max }
    }

    fn import_qualified(
        db: &Database,
        local_def_source: Option<&str>,
        qualified_identifier: &QualifiedIdentifier,
    ) -> Tokens<Self::GencoLang> {
        if let Some(module) = &qualified_identifier.module {
            // Import from some other crate.
            let lib_suffix = db
                .lookup_module_lib_suffix(module)
                // TODO error handling
                .expect(&format!("module '{module}' not found"));
            quote! {
                $(genco::lang::js::import(
                    format!("{module}-{lib_suffix}").as_ref(),
                    &qualified_identifier.name,
                ))
            }
        } else if let Some(local_def_source) = local_def_source {
            // Import from a module in the same crate.
            quote! {
                $(genco::lang::js::import(
                    local_def_source,
                    &qualified_identifier.name,
                ))
            }
        } else {
            // No import required.
            quote! { $(&qualified_identifier.name) }
        }
    }
}

impl MprotoLang for MprotoRust {
    type GencoLang = genco::lang::Rust;

    fn associated_constant(type_name: &str, constant: &str) -> Tokens<Self::GencoLang> {
        quote! { $type_name::$constant }
    }

    fn type_param_base_len(type_name: &str) -> Tokens<Self::GencoLang> {
        Self::associated_constant(type_name, "BASE_LEN")
    }

    fn const_fn_max() -> Tokens<Self::GencoLang> {
        quote! { $(genco::lang::rust::import("mproto", "max")) }
    }

    fn import_qualified(
        db: &Database,
        local_def_source: Option<&str>,
        qualified_identifier: &QualifiedIdentifier,
    ) -> Tokens<Self::GencoLang> {
        if let Some(module) = &qualified_identifier.module {
            // Import from some other crate.
            let lib_suffix = db
                .lookup_module_lib_suffix(module)
                // TODO error handling
                .expect(&format!("module '{module}' not found"));
            quote! {
                $(
                    genco::lang::rust::import(
                        &format!("{module}_{lib_suffix}"),
                        &qualified_identifier.name,
                    )
                    .qualified()
                )
            }
        } else if let Some(local_def_source) = local_def_source {
            // Import from a module in the same crate.
            quote! {
                $(genco::lang::rust::import(
                    local_def_source,
                    &qualified_identifier.name,
                ))
            }
        } else {
            // No import required.
            quote! { $(&qualified_identifier.name) }
        }
    }
}
