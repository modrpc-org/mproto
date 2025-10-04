use genco::prelude::*;

pub struct EncoderCommon {
    pub type_param_list: js::Tokens,
    pub encoder_fields: js::Tokens,
    pub encoder_constructor: js::Tokens,
    pub lazy_constructor: js::Tokens,
    pub encoder_instance: js::Tokens,
    pub lazy_encoder_instance: js::Tokens,
}

impl EncoderCommon {
    pub fn new(name: &str, type_params: &[String]) -> Self {
        let encode_interface = &js::import("@modrpc-org/mproto", "Encoder");
        let decode_interface = &js::import("@modrpc-org/mproto", "Decoder");

        let ref type_param_list = if type_params.len() > 0 {
            let mut type_param_list = js::Tokens::new();
            quote_in! { type_param_list => $(&type_params[0]) };
            for type_param_name in &type_params[1..] {
                quote_in! { type_param_list => , $type_param_name };
            }

            quote! { <$type_param_list> }
        } else {
            quote! {}
        };

        let encoder_fields = if type_params.len() > 0 {
            let mut type_param_encoder_fields = js::Tokens::new();
            for type_param_name in type_params {
                type_param_encoder_fields = quote! {
                    $type_param_encoder_fields
                    $(type_param_name)Encoder: $encode_interface<$type_param_name> & $decode_interface<$type_param_name>;
                };
            }

            type_param_encoder_fields
        } else {
            quote! {}
        };

        let (encoder_constructor, lazy_constructor) = if type_params.len() > 0 {
            let mut type_param_encoders = js::Tokens::new();
            type_param_encoders = quote! {
                $type_param_encoders
                $(&type_params[0])Encoder: $encode_interface<$(&type_params[0])> & $decode_interface<$(&type_params[0])>,
            };
            for type_param_name in &type_params[1..] {
                type_param_encoders = quote! {
                    $type_param_encoders
                    $(type_param_name)Encoder: $encode_interface<$type_param_name> & $decode_interface<$type_param_name>,
                };
            }

            let mut type_param_encoder_fields = js::Tokens::new();
            for type_param_name in type_params {
                type_param_encoder_fields = quote! {
                    $type_param_encoder_fields
                    this.$(type_param_name)Encoder = $(type_param_name)Encoder;
                };
            }

            let type_param_encoders = &type_param_encoders;
            let type_param_encoder_fields = &type_param_encoder_fields;

            (
                quote! {
                    constructor(
                        $type_param_encoders
                    ) {
                        $type_param_encoder_fields
                    }
                },
                quote! {
                    constructor(
                        $type_param_encoders
                        buffer: DataView,
                        offset: number,
                    ) {
                        $type_param_encoder_fields
                        this._buffer = buffer;
                        this._offset = offset;
                    }
                },
            )
        } else {
            (
                quote! {},
                quote! {
                    constructor(
                        buffer: DataView,
                        offset: number,
                    ) {
                        this._buffer = buffer;
                        this._offset = offset;
                    }
                },
            )
        };

        let (encoder_instance, lazy_encoder_instance) = if type_params.len() > 0 {
            let mut param_type_param_encoders: js::Tokens = quote! {
                $(&type_params[0])Encoder: $encode_interface<$(&type_params[0])> & $decode_interface<$(&type_params[0])>,
            };
            for type_param_name in &type_params[1..] {
                param_type_param_encoders = quote! {
                    $param_type_param_encoders
                    $(type_param_name)Encoder: $encode_interface<$type_param_name> & $decode_interface<$type_param_name>,
                };
            }

            let mut type_param_encoders = js::Tokens::new();
            quote_in! { type_param_encoders => $(&type_params[0])Encoder };
            for type_param_name in &type_params[1..] {
                quote_in! { type_param_encoders => , $(type_param_name)Encoder };
            }

            let type_param_encoders = &type_param_encoders;
            let param_type_param_encoders = &param_type_param_encoders;

            (
                quote! {
                    export const Proto$name = $type_param_list(
                        $param_type_param_encoders
                    ) => new $(name)Encoder($type_param_encoders);
                },
                quote! {
                    export const Proto$(name)Lazy = $type_param_list(
                        $param_type_param_encoders
                    ) => new $(name)LazyEncoder($type_param_encoders);
                },
            )
        } else {
            (
                quote! { export const Proto$name = new $(name)Encoder(); },
                quote! { export const Proto$(name)Lazy = new $(name)LazyEncoder(); },
            )
        };

        Self {
            type_param_list: type_param_list.clone(),
            encoder_fields,
            encoder_constructor,
            lazy_constructor,
            encoder_instance,
            lazy_encoder_instance,
        }
    }
}
