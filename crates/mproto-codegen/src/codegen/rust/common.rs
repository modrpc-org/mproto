use std::collections::HashSet;

use genco::prelude::*;

use crate::{
    ast::{
        Enum, EnumVariant, NamedField, PrimitiveType, QualifiedIdentifier, Struct, Type, TypeBody,
    },
    codegen::{
        name_util::camel_to_snake_case,
        rust::{rust_type_lazy_tokens, rust_type_tokens},
        CodegenCx,
    },
    Database,
};

pub fn type_requires_heap(db: &Database, ty: &Type) -> bool {
    match ty {
        Type::Primitive(PrimitiveType::Void) => false,
        Type::Primitive(PrimitiveType::U8) => false,
        Type::Primitive(PrimitiveType::U16) => false,
        Type::Primitive(PrimitiveType::U32) => false,
        Type::Primitive(PrimitiveType::U64) => false,
        Type::Primitive(PrimitiveType::U128) => false,
        Type::Primitive(PrimitiveType::I8) => false,
        Type::Primitive(PrimitiveType::I16) => false,
        Type::Primitive(PrimitiveType::I32) => false,
        Type::Primitive(PrimitiveType::I64) => false,
        Type::Primitive(PrimitiveType::I128) => false,
        Type::Primitive(PrimitiveType::F32) => false,
        Type::Primitive(PrimitiveType::Bool) => false,
        Type::Primitive(PrimitiveType::String) => true,
        Type::Primitive(PrimitiveType::Box(_)) => true,
        Type::Primitive(PrimitiveType::List(_)) => true,
        Type::Primitive(PrimitiveType::Option(item_ty)) => type_requires_heap(db, item_ty),
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => {
            type_requires_heap(db, ok_ty) || type_requires_heap(db, err_ty)
        }
        Type::Defined { ident, .. } => {
            if let Some(type_def) = db.lookup_type_def(ident) {
                match type_def.body {
                    TypeBody::Struct(ref s) => struct_requires_heap(db, s),
                    TypeBody::Enum(ref e) => enum_requires_heap(db, e),
                }
            } else if ident.module.is_none() {
                // Must be a generic type
                // TODO we should still verify it's a valid type name
                false
            } else {
                panic!("type_requires_heap failed to lookup typedef '{:?}'", ident);
            }
        }
    }
}

pub fn struct_requires_heap(db: &Database, s: &Struct) -> bool {
    for field in &s.fields {
        if type_requires_heap(db, &field.ty) {
            return true;
        }
    }

    false
}

pub fn enum_requires_heap(db: &Database, e: &Enum) -> bool {
    for &(_, ref variant) in &e.variants {
        match *variant {
            EnumVariant::Empty => {}
            EnumVariant::NamedFields { ref fields } => {
                for field in fields {
                    if type_requires_heap(db, &field.ty) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn struct_contains_float(db: &Database, s: &Struct) -> bool {
    TypeWalker::new().walk_struct(db, s, &mut |leaf_ty| match leaf_ty {
        PrimitiveType::F32 => true,
        _ => false,
    })
}

pub fn enum_contains_float(db: &Database, e: &Enum) -> bool {
    TypeWalker::new().walk_enum(db, e, &mut |leaf_ty| match leaf_ty {
        PrimitiveType::F32 => true,
        _ => false,
    })
}

pub struct TypeWalker {
    seen: HashSet<QualifiedIdentifier>,
}

impl TypeWalker {
    pub fn new() -> Self {
        Self {
            seen: HashSet::new(),
        }
    }

    /// Stops the walk and returns early if any of the leaf visits returns true.
    ///
    /// Returns true if any of the leaf visitors triggered an early return.
    pub fn walk_type(
        &mut self,
        db: &Database,
        ty: &Type,
        visit_leaf: &mut impl FnMut(&PrimitiveType) -> bool,
    ) -> bool {
        match ty {
            Type::Primitive(PrimitiveType::Box(inner_ty)) => {
                self.walk_type(db, inner_ty, visit_leaf)
            }
            Type::Primitive(PrimitiveType::List(item_ty)) => {
                self.walk_type(db, item_ty, visit_leaf)
            }
            Type::Primitive(PrimitiveType::Option(inner_ty)) => {
                self.walk_type(db, inner_ty, visit_leaf)
            }
            Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => {
                self.walk_type(db, ok_ty, visit_leaf) || self.walk_type(db, err_ty, visit_leaf)
            }
            Type::Primitive(leaf) => visit_leaf(leaf),
            Type::Defined { ident, .. } => {
                if !self.seen.insert(ident.clone()) {
                    // Don't recurse into already-seen types.
                    return false;
                }

                if let Some(type_def) = db.lookup_type_def(ident) {
                    match type_def.body {
                        TypeBody::Struct(ref s) => self.walk_struct(db, s, visit_leaf),
                        TypeBody::Enum(ref e) => self.walk_enum(db, e, visit_leaf),
                    }
                } else if ident.module.is_none() {
                    // Must be a generic type
                    // TODO we should still verify it's a valid type name
                    false
                } else {
                    panic!("type_requires_heap failed to lookup typedef '{:?}'", ident);
                }
            }
        }
    }

    fn walk_struct(
        &mut self,
        db: &Database,
        s: &Struct,
        visit_leaf: &mut impl FnMut(&PrimitiveType) -> bool,
    ) -> bool {
        for field in &s.fields {
            if self.walk_type(db, &field.ty, visit_leaf) {
                return true;
            }
        }

        false
    }

    fn walk_enum(
        &mut self,
        db: &Database,
        e: &Enum,
        visit_leaf: &mut impl FnMut(&PrimitiveType) -> bool,
    ) -> bool {
        for &(_, ref variant) in &e.variants {
            match *variant {
                EnumVariant::Empty => {}
                EnumVariant::NamedFields { ref fields } => {
                    for field in fields {
                        if self.walk_type(db, &field.ty, visit_leaf) {
                            return true;
                        }
                    }
                }
            }
        }

        false
    }
}

pub fn lazy_type_requires_lifetime(db: &Database, ty: &Type) -> bool {
    match &ty {
        Type::Primitive(PrimitiveType::Void) => false,
        Type::Primitive(PrimitiveType::U8) => false,
        Type::Primitive(PrimitiveType::U16) => false,
        Type::Primitive(PrimitiveType::U32) => false,
        Type::Primitive(PrimitiveType::U64) => false,
        Type::Primitive(PrimitiveType::U128) => false,
        Type::Primitive(PrimitiveType::I8) => false,
        Type::Primitive(PrimitiveType::I16) => false,
        Type::Primitive(PrimitiveType::I32) => false,
        Type::Primitive(PrimitiveType::I64) => false,
        Type::Primitive(PrimitiveType::I128) => false,
        Type::Primitive(PrimitiveType::F32) => false,
        Type::Primitive(PrimitiveType::Bool) => false,
        Type::Primitive(PrimitiveType::String) => true,
        Type::Primitive(PrimitiveType::Box(_)) => true,
        Type::Primitive(PrimitiveType::List(_)) => true,
        Type::Primitive(PrimitiveType::Option(item_ty)) => lazy_type_requires_lifetime(db, item_ty),
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => {
            lazy_type_requires_lifetime(db, ok_ty) || lazy_type_requires_lifetime(db, err_ty)
        }
        Type::Defined { ident, .. } => {
            if let Some(type_def) = db.lookup_type_def(ident) {
                match &type_def.body {
                    TypeBody::Struct(_) => true,
                    TypeBody::Enum(e) => lazy_enum_requires_lifetime(db, e),
                }
            } else if ident.module.is_none() {
                // Must be a generic type
                // TODO we should still verify it's a valid type name
                true
            } else {
                panic!(
                    "buf_type_requires_lifetime failed to lookup typedef '{:?}'",
                    ident
                );
            }
        }
    }
}

pub fn lazy_enum_requires_lifetime(db: &Database, e: &Enum) -> bool {
    for &(_, ref variant) in &e.variants {
        match variant {
            EnumVariant::Empty => {}
            EnumVariant::NamedFields { fields } => {
                for field in fields {
                    if lazy_type_requires_lifetime(db, &field.ty) {
                        return true;
                    }
                }
            }
        }
    }

    false
}

pub fn rust_lazy_field_decode(field: &NamedField, field_offset: rust::Tokens) -> rust::Tokens {
    let decode_trait = &rust::import("mproto", "Decode");
    let decode_cursor = &rust::import("mproto", "DecodeCursor");

    if let Type::Primitive(PrimitiveType::Box(_)) = &field.ty {
        // Special handling for boxed types
        quote! {
            $decode_cursor::at_offset(self.buffer, self.offset + $field_offset)
                .inner_in_scratch(|cursor| $decode_trait::decode(cursor))
        }
    } else {
        quote! {
            $decode_trait::decode(&$decode_cursor::at_offset(self.buffer, self.offset + $field_offset))
        }
    }
}

pub fn rust_named_fields_owned(
    cx: &CodegenCx,
    fields: &[NamedField],
    make_pub: bool,
) -> rust::Tokens {
    let mut owned_field_tokens = rust::Tokens::new();
    for field in fields {
        if make_pub {
            owned_field_tokens = quote! {
                $owned_field_tokens
                pub $(&field.name): $(rust_type_tokens(cx, &field.ty)),
            };
        } else {
            owned_field_tokens = quote! {
                $owned_field_tokens
                $(&field.name): $(rust_type_tokens(cx, &field.ty)),
            };
        }
    }

    owned_field_tokens
}

pub fn rust_named_fields_lazy(cx: &CodegenCx, fields: &[NamedField]) -> rust::Tokens {
    let box_lazy = &rust::import("mproto", "BoxLazy");

    let mut ref_field_tokens = rust::Tokens::new();
    for field in fields {
        if let Type::Primitive(PrimitiveType::Box(_)) = &field.ty {
            // special handling for boxed types
            ref_field_tokens = quote! {
                $ref_field_tokens
                $(&field.name): $box_lazy<'a, $(rust_type_lazy_tokens(cx, &field.ty))>,
            };
        } else {
            ref_field_tokens = quote! {
                $ref_field_tokens
                $(&field.name): $(rust_type_lazy_tokens(cx, &field.ty)),
            };
        }
    }

    ref_field_tokens
}

pub fn rust_named_fields_lazy_phantom(type_params: &[String]) -> rust::Tokens {
    let mut phantom_field_tokens = rust::Tokens::new();
    for type_param in type_params {
        phantom_field_tokens = quote! {
            $phantom_field_tokens
            _$(camel_to_snake_case(type_param)): core::marker::PhantomData<$type_param>,
        };
    }

    phantom_field_tokens
}

pub fn rust_named_fields_lazy_phantom_constructor(type_params: &[String]) -> rust::Tokens {
    let mut phantom_field_tokens = rust::Tokens::new();
    for type_param in type_params {
        phantom_field_tokens = quote! {
            $phantom_field_tokens
            _$(camel_to_snake_case(type_param)): core::marker::PhantomData,
        };
    }

    phantom_field_tokens
}

pub fn rust_named_fields_scratch_len(
    fields: &[NamedField],
    field_prefix: rust::Tokens,
) -> rust::Tokens {
    if fields.len() > 0 {
        let mut fields_scratch_len_tokens = rust::Tokens::new();
        for (i, field) in fields.iter().enumerate() {
            quote_in! { fields_scratch_len_tokens =>
                $(&field_prefix)$(&field.name).scratch_len()
            };
            if i < fields.len() - 1 {
                quote_in! { fields_scratch_len_tokens => $(" + ") };
            }
        }

        fields_scratch_len_tokens
    } else {
        quote! { 0 }
    }
}

pub fn rust_named_fields_encode(fields: &[NamedField], field_prefix: rust::Tokens) -> rust::Tokens {
    let mut encode_owned_tokens = quote! {};

    for field in fields {
        encode_owned_tokens = quote! {
            $encode_owned_tokens
            $(&field_prefix)$(&field.name).encode(cursor);
        };
    }
    encode_owned_tokens
}

pub fn rust_named_fields_decode(fields: &[NamedField]) -> rust::Tokens {
    let decode_trait = &rust::import("mproto", "Decode");

    let mut decode_owned_tokens = quote! {};

    for field in fields {
        decode_owned_tokens = quote! {
            $decode_owned_tokens
            let $(&field.name) = $decode_trait::decode(cursor)?;
        };
    }

    decode_owned_tokens
}

pub fn rust_named_fields_constructor(fields: &[NamedField]) -> rust::Tokens {
    let mut constructor_tokens = rust::Tokens::new();
    for field in fields {
        constructor_tokens = quote! {
            $constructor_tokens
            $(&field.name),
        };
    }

    constructor_tokens
}
