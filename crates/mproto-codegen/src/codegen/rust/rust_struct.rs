use genco::prelude::*;

use crate::{
    ast,
    codegen::{
        name_util::snake_to_upper_camel_case,
        rust::{
            common::{
                rust_lazy_field_decode, rust_named_fields_constructor, rust_named_fields_decode,
                rust_named_fields_encode, rust_named_fields_lazy_phantom,
                rust_named_fields_lazy_phantom_constructor, rust_named_fields_owned,
                rust_named_fields_scratch_len, struct_contains_float, struct_requires_heap,
            },
            rust_type_lazy_tokens, rust_type_param_list, rust_type_tokens,
        },
        struct_base_len, type_base_len, type_uses_type_param, CodegenCx, MprotoLang, MprotoRust,
        TypeBaseLen,
    },
};

pub fn rust_struct(
    cx: &CodegenCx,
    name: &str,
    type_params: &[String],
    s: &ast::Struct,
) -> rust::Tokens {
    let encode_cursor = &rust::import("mproto", "EncodeCursor");
    let decode_cursor = &rust::import("mproto", "DecodeCursor");
    let decode_error = &rust::import("mproto", "DecodeError");
    let decode_result = &rust::import("mproto", "DecodeResult");

    let base_len_trait = &rust::import("mproto", "BaseLen");
    let encode_trait = &rust::import("mproto", "Encode");
    let decode_trait = &rust::import("mproto", "Decode");
    let compat_trait = &rust::import("mproto", "Compatible");
    let owned_trait = &rust::import("mproto", "Owned");
    let lazy_trait = &rust::import("mproto", "Lazy");
    let try_from_trait = &rust::import("core::convert", "TryFrom");

    let owned_field_tokens = rust_named_fields_owned(cx, &s.fields, true);
    let fields_scratch_len_tokens = rust_named_fields_scratch_len(&s.fields, quote! { self. });
    let encode_owned_tokens = rust_named_fields_encode(&s.fields, quote! { self. });
    let decode_owned_tokens = rust_named_fields_decode(&s.fields);

    let owned_type_param_tokens = &rust_type_param_list(type_params, None, None);
    let buf_type_param_tokens = &rust_type_param_list(type_params, Some(quote! { 'a }), None);
    let lazy_compat_impl_param_tokens = &rust_type_param_list(
        type_params,
        Some(quote! { 'a }),
        Some(quote! { $owned_trait }),
    );

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
        rust_type_param_list(type_params, Some(quote! { 'a }), None);

    let mut buf_method_tokens = rust::Tokens::new();
    let mut field_offset = TypeBaseLen::<MprotoRust>::constant(0);
    for field in &s.fields {
        buf_method_tokens = quote! {
            $buf_method_tokens

            $(rust_lazy_decoder_method(cx, field, field_offset.as_tokens()))
        };

        field_offset = field_offset.merge(type_base_len(cx, &field.ty));
    }

    let owned_cfg: rust::Tokens = if cx.is_package && struct_requires_heap(cx.db, s) {
        quote! { #[cfg(any(feature = "std", feature = "alloc"))] }
    } else {
        quote! {}
    };

    let mut owned_derive_impls: rust::Tokens = quote! { Clone, Debug, PartialEq, PartialOrd };
    if !struct_requires_heap(cx.db, s) {
        owned_derive_impls = quote! { Copy, $owned_derive_impls };
    }
    if !struct_contains_float(cx.db, s) {
        owned_derive_impls = quote! { $owned_derive_impls, Eq, Ord, Hash };
    }
    if s.fields.is_empty() {
        owned_derive_impls = quote! { $owned_derive_impls, Default };
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

    let mut generic_fields = RustGenericNamedFields::new(type_params, true);
    let (generic_struct_base_len, generic_fields_tokens) =
        generic_fields.add_fields(cx, "", &s.fields);

    let encode_cursor_param = if !s.fields.is_empty() {
        &quote! { cursor: &mut $encode_cursor }
    } else {
        &quote! { _: &mut $encode_cursor }
    };
    let decode_cursor_param = if !s.fields.is_empty() {
        &quote! { cursor: &$decode_cursor<'a> }
    } else {
        &quote! { _: &$decode_cursor<'a> }
    };

    quote! {
        $(register(encode_trait))
        $(register(decode_trait))

        $(&owned_cfg)
        #[derive($owned_derive_impls)]
        pub struct $(name)$(owned_type_param_tokens) {
            $owned_field_tokens
        }

        pub struct $(name)Lazy$(buf_type_param_tokens) {
            buffer: &'a [u8],
            offset: usize,
            $(rust_named_fields_lazy_phantom(type_params))
        }

        pub struct $(name)Gen<
            $(&generic_fields.type_params)
        > {
            $generic_fields_tokens
        }

        impl<
            $(&generic_fields.compat_impl_type_params)
        > $compat_trait<$(name)$(owned_type_param_tokens)> for $(name)Gen<$(&generic_fields.type_args)> { }
        impl<
            $(&generic_fields.compat_impl_type_params)
        > $compat_trait<$(name)Gen<$(&generic_fields.type_args)>> for $(name)$(owned_type_param_tokens) { }

        impl<
            $(&generic_fields.type_params)
        > $base_len_trait for $(name)Gen<$(&generic_fields.type_args)> {
            const BASE_LEN: usize = $(generic_struct_base_len.as_tokens());
        }

        impl<
            $(&generic_fields.type_params)
        > $encode_trait for $(name)Gen<$(&generic_fields.type_args)> {
            fn scratch_len(&self) -> usize {
                $(rust_named_fields_scratch_len(&s.fields, quote! { self. }))
            }

            fn encode(&self, $encode_cursor_param) {
                $(rust_named_fields_encode(&s.fields, quote! { self. }))
            }
        }

        $(&owned_cfg)
        $owned_impl

        $(&owned_cfg)
        impl$(lazy_compat_impl_param_tokens) $compat_trait<$(name)Lazy$(buf_type_param_tokens)> for $(name)$(owned_type_param_tokens) { }
        $(&owned_cfg)
        impl$(lazy_compat_impl_param_tokens) $compat_trait<$(name)$(owned_type_param_tokens)> for $(name)Lazy$(buf_type_param_tokens) { }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
        ) $(name)Lazy$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) {
            $buf_method_tokens
        }

        $(&owned_cfg)
        impl$(
            rust_type_param_list(type_params, None, Some(quote! { $base_len_trait }))
        ) $base_len_trait for $(name)$(
            rust_type_param_list(type_params, None, None)
        ) {
            const BASE_LEN: usize = $(struct_base_len::<MprotoRust>(cx, s).as_tokens());
        }

        impl$(encode_impl_type_param_decl_tokens) $encode_trait for $(name)$(encode_impl_type_param_use_tokens) {
            fn scratch_len(&self) -> usize {
                $(&fields_scratch_len_tokens)
            }

            fn encode(&self, $encode_cursor_param) {
                $(&encode_owned_tokens)
            }
        }

        $(&owned_cfg)
        impl$(&decode_impl_type_param_decl_tokens) $decode_trait<'a> for $(name)$(&decode_owned_impl_type_param_use_tokens) {
            fn decode($decode_cursor_param) -> $decode_result<Self> {
                $decode_owned_tokens

                Ok($name {
                    $(rust_named_fields_constructor(&s.fields))
                })
            }
        }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
        ) $base_len_trait for $(name)Lazy$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) {
            const BASE_LEN: usize = $(struct_base_len::<MprotoRust>(cx, s).as_tokens());
        }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
        ) $encode_trait for $(name)Lazy$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) {
            fn scratch_len(&self) -> usize {
                $(rust_named_fields_lazy_scratch_len(cx, &s.fields))
            }

            fn encode(&self, $encode_cursor_param) {
                $(rust_named_fields_lazy_encode(cx, &s.fields))
            }
        }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
        ) $decode_trait<'a> for $(name)Lazy$(&decode_lazy_impl_type_param_use_tokens) {
            fn decode(cursor: &$decode_cursor<'a>) -> $decode_result<Self> {
                let offset = cursor.offset();
                cursor.advance(Self::BASE_LEN);
                Ok($(name)Lazy {
                    buffer: cursor.buffer(),
                    offset,
                    $(rust_named_fields_lazy_phantom_constructor(type_params))
                })
            }
        }

        $(&owned_cfg)
        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
        ) $try_from_trait<$(name)Lazy$(rust_type_param_list(type_params, Some(quote! { 'a }), None))> for $(name)$(
            rust_type_param_list(type_params, None, None)
        ) {
            type Error = $decode_error;

            fn try_from(other: $(name)Lazy$(rust_type_param_list(type_params, Some(quote! { 'a }), None))) -> Result<Self, Self::Error> {
                let cursor = $decode_cursor::at_offset(other.buffer, other.offset);
                $decode_trait::decode(&cursor)
            }
        }

        $(rust_lazy_struct_std_trait_impls(name, type_params, s))
    }
}

pub fn rust_lazy_decoder_method(
    cx: &CodegenCx,
    field: &ast::NamedField,
    field_offset: rust::Tokens,
) -> rust::Tokens {
    let decode_result = &rust::import("mproto", "DecodeResult");

    quote! {
        pub fn $(&field.name)(&self) -> $decode_result<$(rust_type_lazy_tokens(cx, &field.ty))> {
            $(rust_lazy_field_decode(field, field_offset))
        }
    }
}

// TODO unwrapping the decoded fields is not ideal. We could change scratch_len and encode method
// signatures to return a new `EncodeResult<()>` type but this would be a big change to the API.
// And most uses of these methods are infallible, so it would be an annoyance.
fn rust_named_fields_lazy_encode(cx: &CodegenCx, fields: &[ast::NamedField]) -> rust::Tokens {
    let mut out_tokens = quote! {};
    let mut field_offset = TypeBaseLen::<MprotoRust>::constant(0);

    for field in fields {
        let field_ty = rust_type_lazy_tokens(cx, &field.ty);

        out_tokens = quote! {
            $out_tokens
            let $(&field.name): $field_ty = $(
                rust_lazy_field_decode(field, field_offset.as_tokens())
            ).unwrap();
        };

        field_offset = field_offset.merge(type_base_len(cx, &field.ty));
    }

    out_tokens = quote! {
        $out_tokens
        $(rust_named_fields_encode(fields, quote! { }))
    };

    out_tokens
}

// TODO unwrapping the decoded fields is not ideal - see comment at rust_named_fields_lazy_encode
fn rust_named_fields_lazy_scratch_len(cx: &CodegenCx, fields: &[ast::NamedField]) -> rust::Tokens {
    let mut out_tokens = quote! {};
    let mut field_offset = TypeBaseLen::<MprotoRust>::constant(0);

    for field in fields {
        let field_ty = rust_type_lazy_tokens(cx, &field.ty);

        out_tokens = quote! {
            $out_tokens
            let $(&field.name): $field_ty = $(
                rust_lazy_field_decode(field, field_offset.as_tokens())
            ).unwrap();
        };

        field_offset = field_offset.merge(type_base_len(cx, &field.ty));
    }

    out_tokens = quote! {
        $out_tokens
        $(rust_named_fields_scratch_len(&fields, quote! { }))
    };

    out_tokens
}

struct RustGenericNamedFields {
    type_params: rust::Tokens,
    compat_impl_type_params: rust::Tokens,
    type_args: rust::Tokens,
    pub_fields: bool,
}

impl RustGenericNamedFields {
    pub fn new(struct_params: &[String], pub_fields: bool) -> Self {
        let mut compat_impl_type_params = if struct_params.is_empty() {
            quote! {}
        } else {
            quote! { $(&struct_params[0]): $(rust::import("mproto", "Owned")) }
        };
        if struct_params.len() > 1 {
            for param_name in &struct_params[1..] {
                compat_impl_type_params = quote! {
                    $compat_impl_type_params,
                    $param_name: $(rust::import("mproto", "Owned"))
                };
            }
        }

        Self {
            type_params: rust::Tokens::new(),
            compat_impl_type_params,
            type_args: rust::Tokens::new(),
            pub_fields,
        }
    }

    pub fn add_fields(
        &mut self,
        cx: &CodegenCx,
        param_name_prefix: &str,
        named_fields: &[ast::NamedField],
    ) -> (TypeBaseLen<MprotoRust>, rust::Tokens) {
        let maybe_pub = &if self.pub_fields {
            quote! { pub }
        } else {
            quote! {}
        };

        let mut fields = rust::Tokens::new();
        let mut base_len = TypeBaseLen::constant(0);
        for field in named_fields {
            let mut param_name = format!(
                "{param_name_prefix}{}",
                &snake_to_upper_camel_case(&field.name),
            );
            // Ensure that type parameter name doesn't conflict with an already defined type.
            // Types from imported modules are qualified in generated code, so we don't need to
            // worry about conflicting with types from other modules.
            while cx
                .db
                .lookup_type_def(&ast::QualifiedIdentifier::local(&param_name))
                .is_some()
            {
                param_name.insert(0, 'T');
            }
            let param_name = &param_name;

            // Add field's type bound for the struct definition
            let struct_field_bound = if type_uses_type_param(cx, &field.ty) {
                // By itself this bound is not strict enough - for example a struct like
                // ```
                // struct Foo<T> { x: Bar<T> }
                // ```
                // Ought to have a generated Rust *Gen type like
                // ```
                // struct FooGen<X: for<T> Compatible<Bar<T>>> { x: X }
                // ```
                // But that is not possible, so we'll rely on the Compatible blanket impls for
                // the struct to enforce that instance fields have compatible types.
                Some(quote! { $(rust::import("mproto", "Encode")) })
            } else {
                Self::generic_bound(cx, &field.ty)
            };
            if let Some(bound) = struct_field_bound {
                self.type_params = quote! {
                    $(self.type_params.clone())
                    $param_name: $bound,
                };

                if self.type_args.is_empty() {
                    self.type_args = quote! { $param_name };
                } else {
                    self.type_args.append(quote! { , $param_name });
                }

                fields = quote! {
                    $fields
                    $maybe_pub $(&field.name): $param_name,
                };

                base_len = base_len.merge(TypeBaseLen::tokens(MprotoRust::type_param_base_len(
                    param_name,
                )));
            } else {
                fields = quote! {
                    $fields
                    $maybe_pub $(&field.name): $(rust_type_tokens(cx, &field.ty)),
                };
                base_len = base_len.merge(type_base_len(cx, &field.ty));
            }

            // Add field's type bound for the struct's `Compatible` trait impls
            if let Some(bound) = &Self::generic_bound(cx, &field.ty) {
                if self.compat_impl_type_params.is_empty() {
                    self.compat_impl_type_params.append(quote! {
                        $param_name: $bound
                    });
                } else {
                    self.compat_impl_type_params.append(quote! {
                        ,
                        $param_name: $bound
                    });
                }
            }
        }

        (base_len, fields)
    }

    fn generic_bound(cx: &CodegenCx, ty: &ast::Type) -> Option<rust::Tokens> {
        match ty {
            ast::Type::Primitive(ast::PrimitiveType::String)
            | ast::Type::Primitive(ast::PrimitiveType::Box(_))
            | ast::Type::Primitive(ast::PrimitiveType::List(_))
            | ast::Type::Primitive(ast::PrimitiveType::Option(_))
            | ast::Type::Primitive(ast::PrimitiveType::Result(_, _))
            | ast::Type::Defined { .. } => Some(quote! {
                $(rust::import("mproto", "Encode")) + $(rust::import("mproto", "Compatible"))<$(rust_type_tokens(cx, ty))>
            }),
            _ => None,
        }
    }
}

fn rust_compare_named_fields(
    fields: &[ast::NamedField],
    left_prefix: rust::Tokens,
    right_prefix: rust::Tokens,
    left_suffix: rust::Tokens,
    right_suffix: rust::Tokens,
    comparison: rust::Tokens,
) -> rust::Tokens {
    if fields.is_empty() {
        return quote! { true };
    }

    let mut compare_fields = quote! {
        $(&left_prefix)$(&fields[0].name)$(&left_suffix) $(&comparison) $(&right_prefix)$(&fields[0].name)$(&right_suffix)
    };
    for field in &fields[1..] {
        compare_fields = quote! {
            $compare_fields
                && $(&left_prefix)$(&field.name)$(&left_suffix) $(&comparison) $(&right_prefix)$(&field.name)$(&right_suffix)
        };
    }

    compare_fields
}

fn rust_lazy_struct_std_trait_impls(
    name: &str,
    type_params: &[String],
    s: &ast::Struct,
) -> rust::Tokens {
    let owned_trait = &rust::import("mproto", "Owned");

    quote! {
        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) Copy for $(name)Lazy$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) { }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) Clone for $(name)Lazy$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) {
            fn clone(&self) -> Self {
                Self {
                    buffer: self.buffer,
                    offset: self.offset,
                    $(rust_named_fields_lazy_phantom_constructor(type_params))
                }
            }
        }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) core::fmt::Debug for $(name)Lazy$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) {
            fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
                f.debug_struct($("\"")$(name)Lazy$("\""))
                    .finish()
            }
        }

        impl$(
            rust_type_param_list(type_params, Some(quote! { 'a }), Some(quote! { $owned_trait }))
        ) PartialEq for $(name)Lazy$(
            rust_type_param_list(type_params, Some(quote! { 'a }), None)
        ) {
            fn eq(&self, $(if !s.fields.is_empty() { other } else { _ }): &Self) -> bool {
                $(rust_compare_named_fields(
                    &s.fields,
                    quote!{ self. },
                    quote!{ other. },
                    quote!{ ().unwrap() },
                    quote!{ ().unwrap() },
                    quote!{ == },
                ))
            }
        }
    }
}
