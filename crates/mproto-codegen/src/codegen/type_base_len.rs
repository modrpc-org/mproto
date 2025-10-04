use genco::prelude::*;

use crate::{
    ast::{Enum, EnumVariant, PrimitiveType, Struct, Type, TypeBody},
    codegen::{CodegenCx, MprotoLang, ResolvedType},
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypeBaseLen<L: MprotoLang> {
    constant: usize,
    tokens: Tokens<L::GencoLang>,
}

impl<L: MprotoLang> TypeBaseLen<L> {
    pub fn constant(constant: usize) -> Self {
        Self {
            constant,
            tokens: Tokens::new(),
        }
    }

    pub fn tokens(tokens: Tokens<L::GencoLang>) -> Self {
        Self {
            constant: 0,
            tokens,
        }
    }

    pub fn merge(self, other: Self) -> Self {
        let constant = self.constant + other.constant;
        let tokens = if self.tokens.is_empty() {
            other.tokens
        } else if other.tokens.is_empty() {
            self.tokens
        } else {
            quote! { $(self.tokens) + $(other.tokens) }
        };

        Self { constant, tokens }
    }

    pub fn as_tokens(&self) -> Tokens<L::GencoLang> {
        if self.tokens.is_empty() {
            quote! { $(self.constant) }
        } else if self.constant == 0 {
            self.tokens.clone()
        } else {
            quote! { $(self.constant) + $(&self.tokens) }
        }
    }
}

pub fn type_base_len<L: MprotoLang>(cx: &CodegenCx, ty: &Type) -> TypeBaseLen<L> {
    match ty {
        Type::Primitive(PrimitiveType::Void)    => TypeBaseLen::constant(0),
        Type::Primitive(PrimitiveType::U8)      => TypeBaseLen::constant(1),
        Type::Primitive(PrimitiveType::U16)     => TypeBaseLen::constant(2),
        Type::Primitive(PrimitiveType::U32)     => TypeBaseLen::constant(4),
        Type::Primitive(PrimitiveType::U64)     => TypeBaseLen::constant(8),
        Type::Primitive(PrimitiveType::U128)    => TypeBaseLen::constant(16),
        Type::Primitive(PrimitiveType::I8)      => TypeBaseLen::constant(1),
        Type::Primitive(PrimitiveType::I16)     => TypeBaseLen::constant(2),
        Type::Primitive(PrimitiveType::I32)     => TypeBaseLen::constant(4),
        Type::Primitive(PrimitiveType::I64)     => TypeBaseLen::constant(8),
        Type::Primitive(PrimitiveType::I128)    => TypeBaseLen::constant(16),
        Type::Primitive(PrimitiveType::F32)     => TypeBaseLen::constant(4),
        Type::Primitive(PrimitiveType::Bool)    => TypeBaseLen::constant(1),
        Type::Primitive(PrimitiveType::String)  => TypeBaseLen::constant(8),
        Type::Primitive(PrimitiveType::Box(_))  => TypeBaseLen::constant(4),
        Type::Primitive(PrimitiveType::List(_)) => TypeBaseLen::constant(8),
        Type::Primitive(PrimitiveType::Option(item_ty)) => {
            TypeBaseLen::constant(1).merge(type_base_len(cx, item_ty))
        },
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => {
            TypeBaseLen::constant(1).merge(
                TypeBaseLen::tokens(quote! {
                    $(L::const_fn_max())($(type_base_len::<L>(cx, ok_ty).as_tokens()), $(type_base_len::<L>(cx, err_ty).as_tokens()))
                })
            )
        }
        Type::Defined { ident, args } => {
            match cx.resolve_type(ident) {
                Some(ResolvedType::Defined(type_def)) => {
                    let inner_cx = cx.with_type_args(&type_def.params, &args);
                    match type_def.body {
                        TypeBody::Struct(ref s) => struct_base_len(&inner_cx, s),
                        TypeBody::Enum(ref e) => enum_base_len(&inner_cx, e),
                    }
                }
                Some(ResolvedType::UnboundParam) => {
                    TypeBaseLen::tokens(L::type_param_base_len(&ident.name))
                }
                Some(ResolvedType::BoundParam { value, binding_cx }) => {
                    type_base_len(&cx.with_type_param_bindings(binding_cx), value)
                }
                None => {
                    panic!("type_base_len failed to resolve type: {:?}", ident);
                }
            }
        }
    }
}

pub fn struct_base_len<L: MprotoLang>(cx: &CodegenCx, s: &Struct) -> TypeBaseLen<L> {
    let mut base_len = TypeBaseLen::constant(0);

    for field in &s.fields {
        base_len = base_len.merge(type_base_len(cx, &field.ty));
    }

    base_len
}

pub fn enum_base_len<L: MprotoLang>(cx: &CodegenCx, e: &Enum) -> TypeBaseLen<L> {
    let mut base_len = TypeBaseLen::constant(0);

    for &(_, ref variant) in &e.variants {
        let variant_base_len = enum_variant_base_len::<L>(cx, variant);

        base_len = TypeBaseLen::tokens(quote! {
            $(L::const_fn_max())($(base_len.as_tokens()), $(variant_base_len.as_tokens()))
        });
    }

    // 1 extra byte for the enum tag
    TypeBaseLen::constant(1).merge(base_len)
}

pub fn enum_variant_base_len<L: MprotoLang>(
    cx: &CodegenCx,
    variant: &EnumVariant,
) -> TypeBaseLen<L> {
    let mut variant_base_len = TypeBaseLen::<L>::constant(0);

    match *variant {
        EnumVariant::Empty => {}
        EnumVariant::NamedFields { ref fields } => {
            for field in fields {
                variant_base_len = variant_base_len.merge(type_base_len(cx, &field.ty));
            }
        }
    }

    variant_base_len
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{PrimitiveType, QualifiedIdentifier, Type},
        codegen::MprotoRust,
        Database, Module,
    };

    use super::*;

    #[test]
    fn test_type_base_len_nested_type_param() {
        let s = "struct Foo<T> { foo: Bar<T> }\nstruct Bar<T> { foo: Baz<T>, x: u8 }\nstruct Baz<T> { foo: T, y: u32 }\n";

        let (_, type_defs) = crate::parse::root(s).unwrap();

        let local_module = Module::from_type_defs(type_defs.into());
        let db = Database::new(local_module);

        let foo_base_len = super::type_base_len::<MprotoRust>(
            &CodegenCx::new_with_type_params(&db, None, false, &["X"]),
            &Type::Defined {
                ident: QualifiedIdentifier::local("Foo"),
                args: vec![Type::Primitive(PrimitiveType::U64)],
            },
        );

        assert_eq!(foo_base_len, TypeBaseLen::constant(8 + 1 + 4),);
    }
}
