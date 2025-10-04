use genco::prelude::*;

use crate::{
    ast::NamedField,
    codegen::{js::js_type_encoder, CodegenCx},
};

pub fn js_named_fields_encode(cx: &CodegenCx, fields: &[NamedField]) -> js::Tokens {
    let mut encode_owned_tokens = quote! {};

    for field in fields {
        encode_owned_tokens = quote! {
            $encode_owned_tokens
            $(js_type_encoder(cx, &field.ty)).encode(cursor, value.$(&field.name));
        };
    }

    encode_owned_tokens
}

pub fn js_named_fields_decode(cx: &CodegenCx, fields: &[NamedField]) -> js::Tokens {
    let mut decode_owned_tokens = quote! {};

    for field in fields {
        decode_owned_tokens = quote! {
            $decode_owned_tokens
            let $(&field.name) = $(js_type_encoder(cx, &field.ty)).decode(cursor);
        };
    }

    decode_owned_tokens
}

pub fn js_named_fields_scratch_len(cx: &CodegenCx, fields: &[NamedField]) -> js::Tokens {
    let mut fields_scratch_len_tokens = js::Tokens::new();
    for field in fields {
        quote_in! { fields_scratch_len_tokens =>
            $(js_type_encoder(cx, &field.ty)).scratchLength(value.$(&field.name)) +$(" ")
        };
    }

    fields_scratch_len_tokens
}
