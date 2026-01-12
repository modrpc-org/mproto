use genco::prelude::*;

use self::{common::lazy_type_requires_lifetime, rust_enum::rust_enum, rust_struct::rust_struct};
use crate::{
    ast,
    codegen::{CodegenCx, ResolvedType},
};

pub use package::{rust_module_gen, rust_package_gen};

mod common;
mod package;
mod rust_enum;
mod rust_struct;

pub fn rust_type_def(cx: &CodegenCx, type_def: &ast::TypeDef) -> rust::Tokens {
    let cx = &cx.with_type_params(&type_def.params);

    match &type_def.body {
        ast::TypeBody::Struct(struct_def) => {
            rust_struct(cx, &type_def.name, &type_def.params, struct_def)
        }
        ast::TypeBody::Enum(enum_def) => rust_enum(cx, &type_def.name, &type_def.params, enum_def),
    }
}

pub fn rust_type_tokens(cx: &CodegenCx, ty: &ast::Type) -> rust::Tokens {
    match ty {
        ast::Type::Primitive(ast::PrimitiveType::Void) => quote! { () },
        ast::Type::Primitive(ast::PrimitiveType::U8) => quote! { u8 },
        ast::Type::Primitive(ast::PrimitiveType::U16) => quote! { u16 },
        ast::Type::Primitive(ast::PrimitiveType::U32) => quote! { u32 },
        ast::Type::Primitive(ast::PrimitiveType::U64) => quote! { u64 },
        ast::Type::Primitive(ast::PrimitiveType::U128) => quote! { u128 },
        ast::Type::Primitive(ast::PrimitiveType::I8) => quote! { i8 },
        ast::Type::Primitive(ast::PrimitiveType::I16) => quote! { i16 },
        ast::Type::Primitive(ast::PrimitiveType::I32) => quote! { i32 },
        ast::Type::Primitive(ast::PrimitiveType::I64) => quote! { i64 },
        ast::Type::Primitive(ast::PrimitiveType::I128) => quote! { i128 },
        ast::Type::Primitive(ast::PrimitiveType::F32) => quote! { f32 },
        ast::Type::Primitive(ast::PrimitiveType::F64) => quote! { f64 },
        ast::Type::Primitive(ast::PrimitiveType::Bool) => quote! { bool },
        ast::Type::Primitive(ast::PrimitiveType::String) => quote! { String },
        ast::Type::Primitive(ast::PrimitiveType::Box(inner_ty)) => quote! {
            Box<$(rust_type_tokens(cx, inner_ty))>
        },
        ast::Type::Primitive(ast::PrimitiveType::List(item_ty)) => quote! {
            Vec<$(rust_type_tokens(cx, item_ty))>
        },
        ast::Type::Primitive(ast::PrimitiveType::Option(item_ty)) => quote! {
            Option<$(rust_type_tokens(cx, item_ty))>
        },
        ast::Type::Primitive(ast::PrimitiveType::Result(ok_ty, err_ty)) => quote! {
            Result<$(rust_type_tokens(cx, ok_ty)), $(rust_type_tokens(cx, err_ty))>
        },
        ast::Type::Defined { ident, args } => match cx.resolve_type(ident) {
            Some(ResolvedType::Defined(_)) => {
                let args_tokens = rust_type_arg_list(cx, args, None);
                quote! { $(cx.rust_import_qualified(ident))$args_tokens }
            }
            Some(ResolvedType::UnboundParam) => {
                quote! { $(&ident.name) }
            }
            Some(ResolvedType::BoundParam { value, .. }) => rust_type_tokens(cx, value),
            None => {
                panic!("rust_type_tokens failed to resolve type: {:?}", ident);
            }
        },
    }
}

pub fn rust_type_lazy_tokens(cx: &CodegenCx, ty: &ast::Type) -> rust::Tokens {
    match ty {
        ast::Type::Primitive(ast::PrimitiveType::Void) => quote! { () },
        ast::Type::Primitive(ast::PrimitiveType::U8) => quote! { u8 },
        ast::Type::Primitive(ast::PrimitiveType::U16) => quote! { u16 },
        ast::Type::Primitive(ast::PrimitiveType::U32) => quote! { u32 },
        ast::Type::Primitive(ast::PrimitiveType::U64) => quote! { u64 },
        ast::Type::Primitive(ast::PrimitiveType::U128) => quote! { u128 },
        ast::Type::Primitive(ast::PrimitiveType::I8) => quote! { i8 },
        ast::Type::Primitive(ast::PrimitiveType::I16) => quote! { i16 },
        ast::Type::Primitive(ast::PrimitiveType::I32) => quote! { i32 },
        ast::Type::Primitive(ast::PrimitiveType::I64) => quote! { i64 },
        ast::Type::Primitive(ast::PrimitiveType::I128) => quote! { i128 },
        ast::Type::Primitive(ast::PrimitiveType::F32) => quote! { f32 },
        ast::Type::Primitive(ast::PrimitiveType::F64) => quote! { f64 },
        ast::Type::Primitive(ast::PrimitiveType::Bool) => quote! { bool },
        ast::Type::Primitive(ast::PrimitiveType::String) => quote! { &'a str },
        ast::Type::Primitive(ast::PrimitiveType::Box(inner_ty)) => quote! {
            $(rust_type_tokens(cx, inner_ty))
        },
        ast::Type::Primitive(ast::PrimitiveType::List(item_ty)) => quote! {
            $(rust::import("mproto", "ListLazy").qualified())<'a, $(rust_type_tokens(cx, item_ty))>
        },
        ast::Type::Primitive(ast::PrimitiveType::Option(item_ty)) => quote! {
            Option<$(rust_type_lazy_tokens(cx, item_ty))>
        },
        ast::Type::Primitive(ast::PrimitiveType::Result(ok_ty, err_ty)) => quote! {
            Result<$(rust_type_lazy_tokens(cx, ok_ty)), $(rust_type_lazy_tokens(cx, err_ty))>
        },
        ast::Type::Defined { ident, args } => match cx.resolve_type(ident) {
            Some(ResolvedType::Defined(_)) => {
                let maybe_lifetime = if lazy_type_requires_lifetime(cx.db, ty) {
                    Some(quote! { 'a })
                } else {
                    None
                };
                let args_tokens = rust_type_arg_list(cx, args, maybe_lifetime);
                let ref_ident = ast::QualifiedIdentifier {
                    name: format!("{}Lazy", ident.name),
                    module: ident.module.clone(),
                };
                quote! { $(cx.rust_import_qualified(&ref_ident))$(args_tokens) }
            }
            Some(ResolvedType::UnboundParam) => {
                quote! { $(&ident.name)::Lazy<'a> }
            }
            Some(ResolvedType::BoundParam { value, .. }) => rust_type_lazy_tokens(cx, value),
            None => {
                panic!("rust_type_lazy_tokens failed to resolve type: {:?}", ident);
            }
        },
    }
}

pub fn rust_type_arg_list(
    cx: &CodegenCx,
    args: &[ast::Type],
    lifetimes: Option<rust::Tokens>,
) -> rust::Tokens {
    if args.len() == 0 {
        if let Some(lifetimes) = lifetimes {
            quote! { <$lifetimes> }
        } else {
            quote! {}
        }
    } else {
        let lifetimes = lifetimes
            .map(|l| quote! { $l,$(" ") })
            .unwrap_or(rust::Tokens::new());

        let mut args_items: rust::Tokens = quote! {
            $(lifetimes)$(rust_type_tokens(cx, &args[0]))
        };
        for arg in &args[1..] {
            args_items = quote! { $args_items, $(rust_type_tokens(cx, arg)) };
        }

        quote! { <$args_items> }
    }
}

pub fn rust_type_param_list(
    params: &[String],
    lifetimes: Option<rust::Tokens>,
    impl_trait: Option<rust::Tokens>,
) -> rust::Tokens {
    if params.len() == 0 {
        if let Some(lifetimes) = lifetimes {
            quote! { <$lifetimes> }
        } else {
            Tokens::new()
        }
    } else {
        let lifetimes = lifetimes
            .map(|l| quote! { $l,$(" ") })
            .unwrap_or(Tokens::new());
        let impl_trait = impl_trait.map(|i| quote! { : $i }).unwrap_or(Tokens::new());

        let mut tokens = quote! { <$(lifetimes)$(&params[0])$(&impl_trait) };

        for param in &params[1..] {
            tokens = quote! { $tokens, $(param)$(&impl_trait) };
        }

        tokens = quote! { $tokens> };

        tokens
    }
}

pub fn rust_type_default_value(cx: &CodegenCx, ty: &ast::Type) -> rust::Tokens {
    match &ty {
        ast::Type::Primitive(ast::PrimitiveType::Void) => quote! { () },
        ast::Type::Primitive(ast::PrimitiveType::U8) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::U16) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::U32) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::U64) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::U128) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::I8) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::I16) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::I32) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::I64) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::I128) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::F32) => quote! { 0.0 },
        ast::Type::Primitive(ast::PrimitiveType::F64) => quote! { 0.0 },
        ast::Type::Primitive(ast::PrimitiveType::Bool) => quote! { false },
        ast::Type::Primitive(ast::PrimitiveType::String) => quote! { 0 },
        ast::Type::Primitive(ast::PrimitiveType::Box(inner_ty)) => quote! {
            Box::new($(rust_type_default_value(cx, inner_ty)))
        },
        ast::Type::Primitive(ast::PrimitiveType::List(_)) => quote! { [] },
        ast::Type::Primitive(ast::PrimitiveType::Option(_)) => quote! { None },
        ast::Type::Primitive(ast::PrimitiveType::Result(ok_ty, _)) => quote! {
            Ok($(rust_type_default_value(cx, ok_ty)))
        },
        ast::Type::Defined { ident, .. } => {
            if let Some(type_def) = cx.db.lookup_type_def(ident) {
                match &type_def.body {
                    ast::TypeBody::Struct(s) => rust_struct_default_value(cx, &ident, s),
                    ast::TypeBody::Enum(e) => rust_enum_default_value(cx, &ident, e),
                }
            } else {
                quote! { todo!() }
            }
        }
    }
}

pub fn rust_struct_default_value(
    cx: &CodegenCx,
    ident: &ast::QualifiedIdentifier,
    s: &ast::Struct,
) -> rust::Tokens {
    let mut params = rust::Tokens::new();
    for field in &s.fields {
        quote_in! { params =>
            $(&field.name): $(rust_type_default_value(cx, &field.ty)),
        };
    }

    quote! {
        $(cx.rust_import_qualified(ident)) {
            $params
        }
    }
}

pub fn rust_enum_default_value(
    cx: &CodegenCx,
    ident: &ast::QualifiedIdentifier,
    e: &ast::Enum,
) -> rust::Tokens {
    let (variant_name, variant) = &e.variants[0];
    let enum_import = cx.rust_import_qualified(ident);

    match variant {
        ast::EnumVariant::Empty => {
            quote! {
                $(enum_import)::$(variant_name)
            }
        }
        ast::EnumVariant::NamedFields { fields } => {
            let mut params = rust::Tokens::new();
            for field in fields {
                quote_in! { params =>
                    $(&field.name): $(rust_type_default_value(cx, &field.ty)),
                };
            }

            quote! {
                $(enum_import)::$(variant_name) {
                    $params
                }
            }
        }
    }
}
