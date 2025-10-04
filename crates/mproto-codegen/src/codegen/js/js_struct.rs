use genco::prelude::*;

use crate::{
    ast::{NamedField, QualifiedIdentifier, Struct, Type},
    codegen::{
        js::{
            common::{js_named_fields_decode, js_named_fields_encode, js_named_fields_scratch_len},
            encoder_common::EncoderCommon,
            js_encoder_type_args, js_type_lazy_encoder, js_type_lazy_tokens, js_type_tokens,
        },
        struct_base_len, type_base_len, CodegenCx, MprotoJs, TypeBaseLen,
    },
};

pub fn js_struct(cx: &CodegenCx, name: &str, type_params: &[String], s: &Struct) -> js::Tokens {
    let encode_cursor = &js::import("@modrpc-org/mproto", "EncodeCursor");
    let decode_cursor = &js::import("@modrpc-org/mproto", "DecodeCursor");

    let encode_interface = &js::import("@modrpc-org/mproto", "Encoder");
    let decode_interface = &js::import("@modrpc-org/mproto", "Decoder");

    let type_args: Vec<Type> = type_params
        .iter()
        .map(|type_param| Type::Defined {
            ident: QualifiedIdentifier {
                name: type_param.clone(),
                module: None,
            },
            args: Vec::new(),
        })
        .collect();

    let EncoderCommon {
        ref type_param_list,
        ref encoder_fields,
        ref encoder_constructor,
        lazy_constructor,
        encoder_instance,
        lazy_encoder_instance,
    } = EncoderCommon::new(name, type_params);

    let ref full_type_name: js::Tokens = quote! { $(name)$(type_param_list) };
    let ref full_lazy_type_name: js::Tokens = quote! { $(name)Lazy$(type_param_list) };

    let mut owned_field_tokens = js::Tokens::new();
    for field in &s.fields {
        owned_field_tokens = quote! {
            $owned_field_tokens
            $(&field.name): $(js_type_tokens(cx, &field.ty));
        };
    }

    let fields_scratch_len_tokens = js_named_fields_scratch_len(cx, &s.fields);

    let mut lazy_method_tokens = js::Tokens::new();
    let mut field_offset = TypeBaseLen::<MprotoJs>::constant(0);
    for field in &s.fields {
        lazy_method_tokens = quote! {
            $lazy_method_tokens

            $(js_lazy_decoder_method(cx, field, field_offset.as_tokens()))
        };

        field_offset = field_offset.merge(type_base_len(cx, &field.ty));
    }

    let encode_owned_tokens = js_named_fields_encode(cx, &s.fields);
    let decode_owned_tokens = js_named_fields_decode(cx, &s.fields);

    let mut decode_owned_construct = js::Tokens::new();
    for field in &s.fields {
        decode_owned_construct = quote! {
            $decode_owned_construct
            $(&field.name): $(&field.name),
        };
    }

    let tokens: js::Tokens = quote! {
        export interface $full_type_name {
            $owned_field_tokens
        }

        export class $(name)Encoder$(type_param_list) implements $encode_interface<$full_type_name>, $decode_interface<$full_type_name> {
            $encoder_fields

            $encoder_constructor

            baseLength = () => $(struct_base_len::<MprotoJs>(cx, s).as_tokens());

            scratchLength(value: $full_type_name): number {
                return $fields_scratch_len_tokens 0;
            }

            encode(cursor: $encode_cursor, value: $full_type_name) {
                $encode_owned_tokens
            }

            decode(cursor: $decode_cursor): $full_type_name {
                $decode_owned_tokens

                return {
                    $decode_owned_construct
                }
            }
        }

        export class $(name)Lazy$(type_param_list) {
            $encoder_fields
            private _buffer: DataView;
            private _offset: number;

            $lazy_constructor

            $lazy_method_tokens
        }

        export class $(name)LazyEncoder$(type_param_list) implements $decode_interface<$full_lazy_type_name> {
            $encoder_fields

            $encoder_constructor

            baseLength = () => $(struct_base_len::<MprotoJs>(cx, s).as_tokens());

            decode(cursor: $decode_cursor): $full_lazy_type_name {
                let offset = cursor.base(this.baseLength());
                $(if type_params.len() == 0 {
                    return new $(name)Lazy(cursor.buffer, offset);
                } else {
                    return new $(name)Lazy($(js_encoder_type_args(cx, &type_args, js_type_lazy_encoder)), cursor.buffer, offset);
                })
            }
        }

        $encoder_instance
        $lazy_encoder_instance
    };

    tokens
}

pub fn js_lazy_decoder_method(
    cx: &CodegenCx,
    field: &NamedField,
    field_offset: js::Tokens,
) -> js::Tokens {
    let decode_cursor = &js::import("@modrpc-org/mproto", "DecodeCursor");
    let decoder = js_type_lazy_encoder(cx, &field.ty);

    quote! {
        public $(&field.name)(): $(js_type_lazy_tokens(cx, &field.ty)) {
            return $(decoder).decode(new $decode_cursor(this._buffer, this._offset + $field_offset));
        }
    }
}
