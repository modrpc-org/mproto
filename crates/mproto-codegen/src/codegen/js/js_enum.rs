use genco::prelude::*;

use crate::{
    ast::{Enum, EnumVariant},
    codegen::{
        enum_base_len, enum_variant_base_len,
        js::{
            common::{js_named_fields_decode, js_named_fields_encode, js_named_fields_scratch_len},
            encoder_common::EncoderCommon,
            js_type_tokens,
        },
        CodegenCx, MprotoJs,
    },
};

pub fn js_enum(cx: &CodegenCx, name: &str, type_params: &[String], e: &Enum) -> js::Tokens {
    let encode_cursor = &js::import("@modrpc-org/mproto", "EncodeCursor");
    let decode_cursor = &js::import("@modrpc-org/mproto", "DecodeCursor");
    let encode_interface = &js::import("@modrpc-org/mproto", "Encoder");
    let decode_interface = &js::import("@modrpc-org/mproto", "Decoder");

    let EncoderCommon {
        ref type_param_list,
        encoder_fields,
        encoder_constructor,
        encoder_instance,
        ..
    } = EncoderCommon::new(name, type_params);

    let ref full_type_name: js::Tokens = quote! { $(name)$(type_param_list) };

    let mut variants_scratch_len_tokens = js::Tokens::new();
    for (variant_name, variant) in &e.variants {
        variants_scratch_len_tokens = quote! {
            $variants_scratch_len_tokens
            if (value instanceof $(name).$(variant_name)) {
                $({
                    match variant {
                        EnumVariant::Empty => {
                            quote! {
                                return 0;
                            }
                        }
                        EnumVariant::NamedFields { fields } => {
                            quote! {
                                return $(js_named_fields_scratch_len(cx, fields))0;
                            }
                        }
                    }
                })
            }
        };
    }

    let mut variants_encode_tokens = js::Tokens::new();
    for (i, (variant_name, variant)) in e.variants.iter().enumerate() {
        let variant_base_len = enum_variant_base_len::<MprotoJs>(cx, variant).as_tokens();
        variants_encode_tokens = quote! {
            $variants_encode_tokens
            if (value instanceof $(name).$(variant_name)) {
                $({
                    match variant {
                        EnumVariant::Empty => {
                            quote! {
                                cursor.buffer.setUint8(cursor.base(1), $i);
                                cursor.base(this.baseLength() - 1);
                            }
                        }
                        EnumVariant::NamedFields { fields } => {
                            quote! {
                                cursor.buffer.setUint8(cursor.base(1), $i);
                                cursor.base(this.baseLength() - 1 - $variant_base_len);
                                $(js_named_fields_encode(cx, fields))
                            }
                        }
                    }
                })
            }
        };
    }

    let mut variants_decode_tokens: js::Tokens = quote! {
        let variant = cursor.buffer.getUint8(cursor.base(1));
    };
    for (i, (variant_name, variant)) in e.variants.iter().enumerate() {
        let variant_base_len = enum_variant_base_len::<MprotoJs>(cx, variant).as_tokens();
        variants_decode_tokens = quote! {
            $variants_decode_tokens
            if (variant == $i) {
                $({
                    match variant {
                        EnumVariant::Empty => {
                            quote! {
                                cursor.base(this.baseLength() - 1);
                                return new $name.$variant_name();
                            }
                        }
                        EnumVariant::NamedFields { fields } => {
                            let decode_fields = js_named_fields_decode(cx, fields);

                            let mut constructor_fields = js::Tokens::new();
                            quote_in! { constructor_fields => $(&fields[0].name) };
                            for field in fields[1..].iter() {
                                quote_in! { constructor_fields => , $(&field.name) };
                            }

                            quote! {
                                $decode_fields
                                cursor.base(this.baseLength() - 1 - $variant_base_len);
                                return new $name.$variant_name($constructor_fields);
                            }
                        }
                    }
                })
            }
        };
    }

    let tokens: js::Tokens = quote! {
        export namespace $name {
            $({
                let mut variant_tokens = quote! { };

                for (variant_name, variant) in &e.variants {
                    let mut variant_fields = quote! { };
                    let mut variant_constr_params = quote! { };
                    let mut variant_constr_set_fields = quote! { };

                    match variant {
                        EnumVariant::Empty => { }
                        EnumVariant::NamedFields { fields } => {
                            for field in fields {
                                variant_fields = quote! {
                                    $variant_fields
                                    public $(&field.name): $(js_type_tokens(cx, &field.ty));
                                };
                                variant_constr_params = quote! {
                                    $variant_constr_params
                                    $(&field.name): $(js_type_tokens(cx, &field.ty)),
                                };
                                variant_constr_set_fields = quote! {
                                    $variant_constr_set_fields
                                    this.$(&field.name) = $(&field.name);
                                };
                            }
                        }
                    }

                    variant_tokens = quote! {
                        $variant_tokens
                        export class $(variant_name)$(type_param_list) {
                            $variant_fields

                            constructor(
                                $variant_constr_params
                            ) {
                                $variant_constr_set_fields
                            }

                            toString(): string { return $("`")$variant_name ${JSON.stringify(this)}$("`"); }
                        }
                    };
                }

                variant_tokens
            });
        }

        export type $full_type_name = $({
            let mut variant_tokens = quote! {
                $name.$(&e.variants[0].0)$(type_param_list)
            };

            for (variant_name, _) in &e.variants[1..] {
                variant_tokens = quote! {
                    $variant_tokens
                    | $name.$(variant_name)$(type_param_list)
                };
            }

            variant_tokens
        });

        export class $(name)Encoder$(type_param_list) implements $encode_interface<$full_type_name>, $decode_interface<$full_type_name> {
            $encoder_fields

            $encoder_constructor

            baseLength = () => $(enum_base_len::<MprotoJs>(cx, e).as_tokens());

            scratchLength(value: $full_type_name): number {
                $variants_scratch_len_tokens
            }

            encode(cursor: $encode_cursor, value: $full_type_name) {
                $variants_encode_tokens
            }

            decode(cursor: $decode_cursor): $full_type_name {
                $variants_decode_tokens
            }
        }

        $encoder_instance
    };

    tokens
}
