use genco::prelude::*;

use self::{js_enum::js_enum, js_struct::js_struct};
use crate::{
    ast::{PrimitiveType, QualifiedIdentifier, Type, TypeBody, TypeDef},
    codegen::{CodegenCx, ResolvedType},
};

pub use package::{js_module_gen, js_package_gen};

pub(crate) mod common;
pub(crate) mod encoder_common;
mod js_enum;
mod js_struct;
mod package;

pub fn js_type_def(cx: &CodegenCx, type_def: &TypeDef) -> js::Tokens {
    let cx = &cx.with_type_params(&type_def.params);

    match &type_def.body {
        TypeBody::Struct(struct_def) => js_struct(cx, &type_def.name, &type_def.params, struct_def),
        TypeBody::Enum(enum_def) => js_enum(cx, &type_def.name, &type_def.params, enum_def),
    }
}

pub fn js_type_tokens(cx: &CodegenCx, ty: &Type) -> js::Tokens {
    match ty {
        Type::Primitive(PrimitiveType::Void) => quote! { void },
        Type::Primitive(PrimitiveType::U8) => quote! { number },
        Type::Primitive(PrimitiveType::U16) => quote! { number },
        Type::Primitive(PrimitiveType::U32) => quote! { number },
        Type::Primitive(PrimitiveType::U64) => quote! { bigint },
        Type::Primitive(PrimitiveType::U128) => quote! { bigint },
        Type::Primitive(PrimitiveType::I8) => quote! { number },
        Type::Primitive(PrimitiveType::I16) => quote! { number },
        Type::Primitive(PrimitiveType::I32) => quote! { number },
        Type::Primitive(PrimitiveType::I64) => quote! { bigint },
        Type::Primitive(PrimitiveType::I128) => quote! { bigint },
        Type::Primitive(PrimitiveType::F32) => quote! { number },
        Type::Primitive(PrimitiveType::F64) => quote! { number },
        Type::Primitive(PrimitiveType::Bool) => quote! { boolean },
        Type::Primitive(PrimitiveType::String) => quote! { string },
        Type::Primitive(PrimitiveType::Box(inner_ty)) => js_type_tokens(cx, inner_ty),
        Type::Primitive(PrimitiveType::List(item_ty)) => quote! {
            $(js_type_tokens(cx, item_ty))[]
        },
        Type::Primitive(PrimitiveType::Option(item_ty)) => quote! {
            $(js_type_tokens(cx, item_ty)) | null
        },
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "Result"))<$(js_type_tokens(cx, ok_ty)), $(js_type_tokens(cx, err_ty))>
        },
        Type::Defined { ident, args } => match cx.resolve_type(ident) {
            Some(ResolvedType::Defined(_)) => {
                let args = js_type_args(cx, args, js_type_tokens);
                let import = cx.js_import_qualified(ident);
                quote! { $(import)$(args) }
            }
            Some(ResolvedType::UnboundParam) => {
                quote! { $(&ident.name) }
            }
            Some(ResolvedType::BoundParam { value, .. }) => js_type_tokens(cx, value),
            None => {
                panic!("js_type_tokens failed to resolve type: {:?}", ident);
            }
        },
    }
}

pub fn js_type_lazy_tokens(cx: &CodegenCx, ty: &Type) -> js::Tokens {
    match ty {
        Type::Primitive(PrimitiveType::Box(inner_ty)) => js_type_lazy_tokens(cx, &inner_ty),
        Type::Primitive(PrimitiveType::List(item_ty)) => {
            let list_lazy = js::import("@modrpc-org/mproto", "ListLazy");
            quote! { $list_lazy<$(js_type_lazy_tokens(cx, &item_ty))> }
        }
        Type::Primitive(PrimitiveType::Option(inner_ty)) => {
            let option = js::import("@modrpc-org/mproto", "Option");
            quote! { $option<$(js_type_lazy_tokens(cx, &inner_ty))> }
        }
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "Result"))<$(js_type_lazy_tokens(cx, ok_ty)), $(js_type_lazy_tokens(cx, err_ty))>
        },
        Type::Primitive(_) => js_type_tokens(cx, ty),
        Type::Defined { ident, args } => {
            match cx.resolve_type(ident) {
                Some(ResolvedType::Defined(type_def)) => {
                    if type_def.body.is_enum() {
                        // Lazy decoders aren't generated for enum types yet.
                        let args = js_type_args(cx, args, js_type_tokens);
                        let import = cx.js_import_qualified(&QualifiedIdentifier {
                            name: format!("{}", ident.name),
                            module: ident.module.clone(),
                        });
                        quote! { $(import)$(args) }
                    } else {
                        let args = js_type_args(cx, args, js_type_tokens);
                        let import = cx.js_import_qualified(&QualifiedIdentifier {
                            name: format!("{}Lazy", ident.name),
                            module: ident.module.clone(),
                        });
                        quote! { $(import)$(args) }
                    }
                }
                Some(ResolvedType::UnboundParam) => {
                    quote! { $(&ident.name) }
                }
                Some(ResolvedType::BoundParam { value, .. }) => js_type_lazy_tokens(cx, value),
                None => {
                    panic!("js_type_lazy_tokens failed to resolve type: {:?}", ident);
                }
            }
        }
    }
}

pub fn js_type_args(
    cx: &CodegenCx,
    args: &[Type],
    mut gen_tokens_fn: impl FnMut(&CodegenCx, &Type) -> js::Tokens,
) -> js::Tokens {
    if args.len() > 0 {
        let arg_tokens = gen_tokens_fn(cx, &args[0]);
        let mut args_items: js::Tokens = quote! { $arg_tokens };
        for arg in &args[1..] {
            let arg_tokens = gen_tokens_fn(cx, arg);
            args_items = quote! { $args_items, $arg_tokens };
        }

        quote! { <$args_items> }
    } else {
        quote! {}
    }
}

pub fn js_type_param_list(params: &[String]) -> js::Tokens {
    if params.len() == 0 {
        Tokens::new()
    } else {
        let mut tokens = quote! { <$(&params[0]) };

        for param in &params[1..] {
            tokens = quote! { $tokens, $param };
        }

        tokens = quote! { $tokens> };

        tokens
    }
}

pub fn js_encoder_type_args(
    cx: &CodegenCx,
    args: &[Type],
    mut gen_tokens_fn: impl FnMut(&CodegenCx, &Type) -> js::Tokens,
) -> js::Tokens {
    if args.len() > 0 {
        let arg_tokens = gen_tokens_fn(cx, &args[0]);
        let mut args_items: js::Tokens = quote! { $arg_tokens };
        for arg in &args[1..] {
            let arg_tokens = gen_tokens_fn(cx, arg);
            args_items = quote! { $args_items, $arg_tokens };
        }

        quote! { $args_items }
    } else {
        quote! {}
    }
}

pub fn js_encoder_type_args_enclosed(
    cx: &CodegenCx,
    args: &[Type],
    gen_tokens_fn: impl FnMut(&CodegenCx, &Type) -> js::Tokens,
) -> js::Tokens {
    if args.len() > 0 {
        quote! { ($(js_encoder_type_args(cx, args, gen_tokens_fn))) }
    } else {
        quote! {}
    }
}

pub fn js_type_encoder(cx: &CodegenCx, ty: &Type) -> js::Tokens {
    match ty {
        Type::Primitive(PrimitiveType::Void) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoVoid")) }
        }
        Type::Primitive(PrimitiveType::U8) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoUint8")) }
        }
        Type::Primitive(PrimitiveType::U16) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoUint16")) }
        }
        Type::Primitive(PrimitiveType::U32) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoUint32")) }
        }
        Type::Primitive(PrimitiveType::U64) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoUint64")) }
        }
        Type::Primitive(PrimitiveType::U128) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoUint128")) }
        }
        Type::Primitive(PrimitiveType::I8) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoInt8")) }
        }
        Type::Primitive(PrimitiveType::I16) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoInt16")) }
        }
        Type::Primitive(PrimitiveType::I32) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoInt32")) }
        }
        Type::Primitive(PrimitiveType::I64) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoInt64")) }
        }
        Type::Primitive(PrimitiveType::I128) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoInt128")) }
        }
        Type::Primitive(PrimitiveType::String) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoString")) }
        }
        Type::Primitive(PrimitiveType::F32) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoFloat32")) }
        }
        Type::Primitive(PrimitiveType::F64) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoFloat64")) }
        }
        Type::Primitive(PrimitiveType::Bool) => {
            quote! { $(js::import("@modrpc-org/mproto", "ProtoBool")) }
        }
        Type::Primitive(PrimitiveType::Box(inner_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoBox"))($(js_type_encoder(cx, inner_ty)))
        },
        Type::Primitive(PrimitiveType::List(item_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoList"))($(js_type_encoder(cx, item_ty)))
        },
        Type::Primitive(PrimitiveType::Option(inner_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoOption"))($(js_type_encoder(cx, inner_ty)))
        },
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoResult"))($(js_type_encoder(cx, ok_ty)), $(js_type_encoder(cx, err_ty)))
        },
        Type::Defined { ident, args } => match cx.resolve_type(ident) {
            Some(ResolvedType::Defined(_)) => {
                let args = js_encoder_type_args_enclosed(cx, args, js_type_encoder);
                let import = cx.js_import_qualified(&QualifiedIdentifier {
                    name: format!("Proto{}", ident.name),
                    module: ident.module.clone(),
                });
                quote! { $(import)$(args) }
            }
            Some(ResolvedType::UnboundParam) => {
                quote! { this.$(&ident.name)Encoder }
            }
            Some(ResolvedType::BoundParam { value, .. }) => js_type_encoder(cx, value),
            None => {
                panic!("js_type_encoder failed to resolve type: {:?}", ident);
            }
        },
    }
}

pub fn js_type_lazy_encoder(cx: &CodegenCx, ty: &Type) -> js::Tokens {
    match ty {
        Type::Primitive(PrimitiveType::Box(item_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoBoxLazy"))($(js_type_lazy_encoder(cx, item_ty)))
        },
        Type::Primitive(PrimitiveType::List(item_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoListLazy"))($(js_type_lazy_encoder(cx, item_ty)))
        },
        Type::Primitive(PrimitiveType::Option(inner_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoOptionLazy"))($(js_type_lazy_encoder(cx, inner_ty)))
        },
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => quote! {
            $(js::import("@modrpc-org/mproto", "ProtoResultLazy"))($(js_type_lazy_encoder(cx, ok_ty)), $(js_type_lazy_encoder(cx, err_ty)))
        },
        Type::Primitive(_) => js_type_encoder(cx, ty),
        Type::Defined { ident, args } => {
            match cx.resolve_type(ident) {
                Some(ResolvedType::Defined(type_def)) => {
                    if type_def.body.is_enum() {
                        // Lazy decoders aren't generated for enum types yet.
                        let args = js_encoder_type_args_enclosed(cx, args, js_type_encoder);
                        let import = cx.js_import_qualified(&QualifiedIdentifier {
                            name: format!("Proto{}", ident.name),
                            module: ident.module.clone(),
                        });
                        quote! { $(import)$(args) }
                    } else {
                        let args = js_encoder_type_args_enclosed(cx, args, js_type_encoder);
                        let import = cx.js_import_qualified(&QualifiedIdentifier {
                            name: format!("Proto{}Lazy", ident.name),
                            module: ident.module.clone(),
                        });
                        quote! { $(import)$(args) }
                    }
                }
                Some(ResolvedType::UnboundParam) => {
                    quote! { this.$(&ident.name)Encoder }
                }
                Some(ResolvedType::BoundParam { value, .. }) => js_type_lazy_encoder(cx, value),
                None => {
                    panic!("js_type_lazy_encoder failed to resolve type: {:?}", ident);
                }
            }
        }
    }
}
