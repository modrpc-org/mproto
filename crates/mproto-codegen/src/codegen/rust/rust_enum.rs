use genco::prelude::*;

use crate::{
    ast,
    codegen::{
        enum_base_len, enum_variant_base_len,
        rust::{
            common::{
                enum_contains_float, enum_requires_heap, lazy_enum_requires_lifetime,
                rust_named_fields_constructor, rust_named_fields_decode, rust_named_fields_encode,
                rust_named_fields_lazy, rust_named_fields_owned, rust_named_fields_scratch_len,
            },
            rust_type_param_list,
        },
        CodegenCx, MprotoRust,
    },
};

fn rust_owned_enum_variant(cx: &CodegenCx, name: &str, variant: &ast::EnumVariant) -> rust::Tokens {
    match variant {
        ast::EnumVariant::Empty => {
            quote! {
                $name,
            }
        }
        ast::EnumVariant::NamedFields { fields } => {
            quote! {
                $name {
                    $(rust_named_fields_owned(cx, fields, false))
                },
            }
        }
    }
}

fn rust_lazy_enum_variant(cx: &CodegenCx, name: &str, variant: &ast::EnumVariant) -> rust::Tokens {
    match variant {
        ast::EnumVariant::Empty => {
            quote! {
                $name,
            }
        }
        ast::EnumVariant::NamedFields { fields } => {
            quote! {
                $name {
                    $(rust_named_fields_lazy(cx, fields))
                },
            }
        }
    }
}

fn rust_enum_variants_scratch_len(name: &str, e: &ast::Enum) -> rust::Tokens {
    let mut variants_scratch_len_tokens = rust::Tokens::new();
    for (variant_name, variant) in e.variants.iter() {
        match variant {
            ast::EnumVariant::Empty => {
                variants_scratch_len_tokens = quote! {
                    $variants_scratch_len_tokens
                    $(name)::$(variant_name) => 0,
                };
            }
            ast::EnumVariant::NamedFields { fields } => {
                let mut pattern_fields = quote! { $(fields.get(0).map(|f| &f.name)) };
                for field in &fields[1..] {
                    pattern_fields = quote! { $pattern_fields, $(&field.name) };
                }

                variants_scratch_len_tokens = quote! {
                    $variants_scratch_len_tokens
                    $(name)::$(variant_name) { $pattern_fields } => {
                        $(rust_named_fields_scratch_len(fields, quote! { }))
                    }
                };
            }
        }
    }

    variants_scratch_len_tokens
}

fn rust_enum_variants_encode(cx: &CodegenCx, name: &str, e: &ast::Enum) -> rust::Tokens {
    let mut variants_encode_tokens = rust::Tokens::new();
    for (i, (variant_name, variant)) in e.variants.iter().enumerate() {
        match variant {
            ast::EnumVariant::Empty => {
                variants_encode_tokens = quote! {
                    $variants_encode_tokens
                    $(name)::$(variant_name) => {
                        cursor.base(1)[0] = $i;
                        cursor.base(Self::BASE_LEN - 1).fill(0);
                    }
                };
            }
            ast::EnumVariant::NamedFields { fields } => {
                let variant_base_len = enum_variant_base_len::<MprotoRust>(cx, variant).as_tokens();
                let mut pattern_fields = quote! { $(fields.get(0).map(|f| &f.name)) };
                for field in &fields[1..] {
                    pattern_fields = quote! { $pattern_fields, $(&field.name) };
                }

                variants_encode_tokens = quote! {
                    $variants_encode_tokens
                    $(name)::$(variant_name) { $pattern_fields } => {
                        cursor.base(1)[0] = $i;
                        $(rust_named_fields_encode(fields, quote! { }))
                        cursor.base(Self::BASE_LEN - 1 - ($variant_base_len)).fill(0);
                    }
                };
            }
        }
    }

    variants_encode_tokens
}

/// Generate code for a match statement over a mproto enum. The supplied function produces Rust
/// tokens for a given enum variant.
#[allow(unused)] // used to use this, I still think it might be useful later.
fn rust_enum_match_variant(
    name: &str,
    e: &ast::Enum,
    mut inner: impl FnMut(usize, &str, &ast::EnumVariant) -> rust::Tokens,
) -> rust::Tokens {
    let mut out_tokens = rust::Tokens::new();
    for (i, (variant_name, variant)) in e.variants.iter().enumerate() {
        match variant {
            ast::EnumVariant::Empty => {
                out_tokens = quote! {
                    $out_tokens
                    $(name)::$(variant_name) => {
                        $(inner(i, variant_name, variant))
                    }
                };
            }
            ast::EnumVariant::NamedFields { fields } => {
                let mut pattern_fields = quote! { $(fields.get(0).map(|f| &f.name)) };
                for field in &fields[1..] {
                    pattern_fields = quote! { $pattern_fields, $(&field.name) };
                }

                out_tokens = quote! {
                    $out_tokens
                    $(name)::$(variant_name) { $pattern_fields } => {
                        $(inner(i, variant_name, variant))
                    }
                };
            }
        }
    }

    out_tokens
}

fn rust_lazy_enum_std_trait_impls(
    cx: &CodegenCx,
    name: &str,
    type_params: &[String],
    e: &ast::Enum,
) -> rust::Tokens {
    let owned_trait = &rust::import("mproto", "Owned");

    let lazy_enum_maybe_lifetime = if lazy_enum_requires_lifetime(cx.db, e) {
        Some(quote! { 'a })
    } else {
        None
    };

    let mut partial_eq_match_body = rust::Tokens::new();
    for (variant_name, variant) in &e.variants {
        match variant {
            ast::EnumVariant::Empty => {
                partial_eq_match_body = quote! {
                    $partial_eq_match_body
                    ($(name)Lazy::$(variant_name), $(name)Lazy::$(variant_name)) => true,
                };
            }
            ast::EnumVariant::NamedFields { fields } => {
                let mut self_pattern_fields =
                    quote! { $(&fields[0].name): self_$(&fields[0].name) };
                for field in &fields[1..] {
                    self_pattern_fields =
                        quote! { $self_pattern_fields, $(&field.name): self_$(&field.name) };
                }
                let mut other_pattern_fields =
                    quote! { $(&fields[0].name): other_$(&fields[0].name) };
                for field in &fields[1..] {
                    other_pattern_fields =
                        quote! { $other_pattern_fields, $(&field.name): other_$(&field.name) };
                }

                let mut compare_fields = quote! {
                    self_$(&fields[0].name) == other_$(&fields[0].name)
                };
                for field in &fields[1..] {
                    compare_fields = quote! {
                        $compare_fields
                            && self_$(&field.name) == other_$(&field.name)
                    };
                }

                partial_eq_match_body = quote! {
                    $partial_eq_match_body
                    (
                        $(name)Lazy::$(variant_name) {
                            $self_pattern_fields
                        },
                        $(name)Lazy::$(variant_name) {
                            $other_pattern_fields
                        },
                    ) => {
                        $compare_fields
                    }
                };
            }
        }
    }

    quote! {
        impl$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), Some(quote! { $owned_trait }))
        ) Copy for $(name)Lazy$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None)
        ) { }

        impl$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), Some(quote! { $owned_trait }))
        ) core::fmt::Debug for $(name)Lazy$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None)
        ) {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct($("\"")$(name)Lazy$("\""))
                    .finish()
            }
        }

        impl$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), Some(quote! { $owned_trait }))
        ) PartialEq for $(name)Lazy$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None)
        ) {
            fn eq(&self, other: &Self) -> bool {
                match (self, other) {
                    $partial_eq_match_body
                    #[allow(unreachable_patterns)]
                    _ => false,
                }
            }
        }
    }
}

pub fn rust_enum(
    cx: &CodegenCx,
    name: &str,
    type_params: &[String],
    e: &ast::Enum,
) -> rust::Tokens {
    let encode_cursor = &rust::import("mproto", "EncodeCursor");
    let decode_cursor = &rust::import("mproto", "DecodeCursor");
    let decode_error = &rust::import("mproto", "DecodeError");
    let decode_result = &rust::import("mproto", "DecodeResult");

    let base_len_trait = &rust::import("mproto", "BaseLen");
    let encode_trait = &rust::import("mproto", "Encode");
    let decode_trait = &rust::import("mproto", "Decode");
    let owned_trait = &rust::import("mproto", "Owned");
    let lazy_trait = &rust::import("mproto", "Lazy");
    let compat_trait = &rust::import("mproto", "Compatible");
    let try_from_trait = &rust::import("core::convert", "TryFrom");

    let lazy_enum_maybe_lifetime = if lazy_enum_requires_lifetime(cx.db, e) {
        Some(quote! { 'a })
    } else {
        None
    };

    let owned_type_param_tokens = &rust_type_param_list(type_params, None, None);
    let buf_type_param_tokens = &rust_type_param_list(
        type_params,
        lazy_enum_maybe_lifetime.clone(),
        Some(quote! { $owned_trait }),
    );
    let buf_type_arg_tokens =
        &rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None);

    let encode_impl_type_param_decl_tokens =
        rust_type_param_list(type_params, None, Some(quote! { $encode_trait }));
    let encode_impl_type_param_use_tokens = &rust_type_param_list(type_params, None, None);

    let decode_impl_type_param_decl_tokens = rust_type_param_list(
        type_params,
        Some(quote! { 'a }),
        Some(quote! { $decode_trait<'a> }),
    );
    let decode_owned_impl_type_param_use_tokens = rust_type_param_list(type_params, None, None);
    let decode_lazy_impl_type_param_use_tokens =
        rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None);

    let owned_cfg: rust::Tokens = if cx.is_package && enum_requires_heap(cx.db, e) {
        quote! { #[cfg(any(feature = "std", feature = "alloc"))] }
    } else {
        quote! {}
    };

    let mut owned_derive_impls: rust::Tokens = quote! { Clone, Debug, PartialEq, PartialOrd };
    if !enum_requires_heap(cx.db, e) {
        owned_derive_impls = quote! { Copy, $owned_derive_impls };
    }
    if !enum_contains_float(cx.db, e) {
        owned_derive_impls = quote! { $owned_derive_impls, Eq, Ord, Hash };
    }

    let mut owned_variant_tokens = rust::Tokens::new();
    for (variant_name, variant) in e.variants.iter() {
        owned_variant_tokens = quote! {
            $owned_variant_tokens
            $(rust_owned_enum_variant(cx, variant_name, variant))
        };
    }

    let mut buf_variant_tokens = rust::Tokens::new();
    for (variant_name, variant) in e.variants.iter() {
        buf_variant_tokens = quote! {
            $buf_variant_tokens
            $(rust_lazy_enum_variant(cx, variant_name, variant))
        };
    }

    let variants_scratch_len_tokens = rust_enum_variants_scratch_len(name, e);
    let variants_encode_tokens = rust_enum_variants_encode(cx, name, e);

    let mut variants_decode_tokens = rust::Tokens::new();
    for (i, (variant_name, variant)) in e.variants.iter().enumerate() {
        let variant_base_len = enum_variant_base_len::<MprotoRust>(cx, variant).as_tokens();
        let variant_decode: rust::Tokens = match variant {
            ast::EnumVariant::Empty => {
                quote! {
                    cursor.advance(Self::BASE_LEN - 1);
                    Ok($(name)::$(variant_name))
                }
            }
            ast::EnumVariant::NamedFields { fields } => {
                quote! {
                    $(rust_named_fields_decode(fields))
                    cursor.advance(Self::BASE_LEN - 1 - ($variant_base_len));
                    Ok($(name)::$(variant_name) {
                        $(rust_named_fields_constructor(fields))
                    })
                }
            }
        };

        variants_decode_tokens = quote! {
            $variants_decode_tokens
            $i => {
                $variant_decode
            }
        };
    }

    let mut variants_decode_lazy_tokens = rust::Tokens::new();
    for (i, (variant_name, variant)) in e.variants.iter().enumerate() {
        let variant_base_len = enum_variant_base_len::<MprotoRust>(cx, variant).as_tokens();
        let variant_decode: rust::Tokens = match variant {
            ast::EnumVariant::Empty => {
                quote! {
                    cursor.advance(Self::BASE_LEN - 1);
                    Ok($(name)Lazy::$(variant_name))
                }
            }
            ast::EnumVariant::NamedFields { fields } => {
                quote! {
                    $(rust_named_fields_decode(fields))
                    cursor.advance(Self::BASE_LEN - 1 - ($variant_base_len));
                    Ok($(name)Lazy::$(variant_name) {
                        $(rust_named_fields_constructor(fields))
                    })
                }
            }
        };

        variants_decode_lazy_tokens = quote! {
            $variants_decode_lazy_tokens
            $i => {
                $variant_decode
            }
        };
    }

    let mut variants_lazy_to_owned_tokens = rust::Tokens::new();
    for (variant_name, variant) in &e.variants {
        let variant_try_from: rust::Tokens = match variant {
            ast::EnumVariant::Empty => quote! {
                $(name)Lazy::$(variant_name) => Ok($(name)::$(variant_name)),
            },
            ast::EnumVariant::NamedFields { fields } => {
                let mut pattern_fields = rust::Tokens::new();
                for field in fields {
                    quote_in! { pattern_fields => $(&field.name), };
                }
                quote! {
                    $(name)Lazy::$(variant_name) { $pattern_fields } => {
                        Ok($(name)::$(variant_name) {
                            $(
                                fields.iter().fold(rust::Tokens::new(), |t, field| quote! {
                                    $t
                                    $(&field.name): $owned_trait::lazy_to_owned($(&field.name))?,
                                })
                            )
                        })
                    }
                }
            }
        };

        variants_lazy_to_owned_tokens = quote! {
            $variants_lazy_to_owned_tokens
            $variant_try_from
        };
    }

    let owned_impl = {
        quote! {
            impl$(
                rust_type_param_list(type_params, None, Some(quote! { $owned_trait }))
            ) $owned_trait for $(name)$(owned_type_param_tokens) {
                type Lazy<'a> = $(name)Lazy$(&decode_lazy_impl_type_param_use_tokens);

                fn lazy_to_owned(lazy: Self::Lazy<'_>) -> DecodeResult<Self> {
                    $try_from_trait::try_from(lazy)
                }
            }

            impl$(
                rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
            ) $lazy_trait<'a> for $(name)Lazy$(&decode_lazy_impl_type_param_use_tokens) {
                type Owned = $(name)$(owned_type_param_tokens);
            }
        }
    };

    quote! {
        $(&owned_cfg)
        #[derive($owned_derive_impls)]
        pub enum $(name)$(owned_type_param_tokens) {
            $owned_variant_tokens
        }

        #[derive(Clone)]
        pub enum $(name)Lazy$(buf_type_param_tokens) {
            $buf_variant_tokens
        }

        $(&owned_cfg)
        impl$(buf_type_param_tokens) $compat_trait<$(name)Lazy$(buf_type_arg_tokens)> for $(name)$(owned_type_param_tokens) { }
        $(&owned_cfg)
        impl$(buf_type_param_tokens) $compat_trait<$(name)$(owned_type_param_tokens)> for $(name)Lazy$(buf_type_arg_tokens) { }

        $(&owned_cfg)
        $owned_impl

        $(&owned_cfg)
        impl$(
            rust_type_param_list(type_params, None, Some(quote! { $base_len_trait }))
        ) $base_len_trait for $(name)$(
            rust_type_param_list(type_params, None, None)
        ) {
            const BASE_LEN: usize = $(enum_base_len::<MprotoRust>(cx, e).as_tokens());
        }

        impl$(&encode_impl_type_param_decl_tokens) $encode_trait for $(name)$(encode_impl_type_param_use_tokens) {
            fn scratch_len(&self) -> usize {
                match self {
                    $variants_scratch_len_tokens
                }
            }

            fn encode(&self, cursor: &mut $encode_cursor) {
                match self {
                    $variants_encode_tokens
                }
            }
        }

        $(&owned_cfg)
        impl$(&decode_impl_type_param_decl_tokens) $decode_trait<'a> for $(name)$(&decode_owned_impl_type_param_use_tokens) {
            fn decode(cursor: &$decode_cursor<'a>) -> $decode_result<Self> {
                let variant = cursor.base(1)[0];
                match variant {
                    $variants_decode_tokens
                    _ => { Err($decode_error) }
                }
            }
        }

        impl$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), Some(quote! { $owned_trait }))
        ) $base_len_trait for $(name)Lazy$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None)
        ) {
            const BASE_LEN: usize = $(enum_base_len::<MprotoRust>(cx, e).as_tokens());
        }

        impl$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), Some(quote! { $owned_trait }))
        ) $encode_trait for $(name)Lazy$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None)
        ) {
            fn scratch_len(&self) -> usize {
                match self {
                    $(rust_enum_variants_scratch_len(&format!("{name}Lazy"), e))
                }
            }

            fn encode(&self, cursor: &mut $encode_cursor) {
                match self {
                    $(rust_enum_variants_encode(cx, &format!("{name}Lazy"), e))
                }
            }
        }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
        ) $decode_trait<'a> for $(name)Lazy$(&decode_lazy_impl_type_param_use_tokens) {
            fn decode(cursor: &$decode_cursor<'a>) -> $decode_result<Self> {
                let variant = cursor.base(1)[0];
                match variant {
                    $variants_decode_lazy_tokens
                    _ => { Err($decode_error) }
                }
            }
        }

        $(&owned_cfg)
        impl$(
            rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), Some(quote! { $owned_trait }))
        ) $try_from_trait<$(name)Lazy$(rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None))> for $(name)$(
            rust_type_param_list(type_params, None, None)
        ) {
            type Error = $decode_error;

            fn try_from(other: $(name)Lazy$(rust_type_param_list(type_params, lazy_enum_maybe_lifetime.clone(), None))) -> Result<Self, Self::Error> {
                match other {
                    $variants_lazy_to_owned_tokens
                }
            }
        }

        $(rust_lazy_enum_std_trait_impls(cx, name, type_params, e))
    }
}
