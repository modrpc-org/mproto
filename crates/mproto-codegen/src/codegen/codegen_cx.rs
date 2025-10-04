use crate::{
    ast::{PrimitiveType, QualifiedIdentifier, Type, TypeDef},
    codegen::{MprotoJs, MprotoLang, MprotoRust},
    Database,
};

pub struct CodegenCx<'a> {
    pub db: &'a Database,
    pub local_def_source: Option<&'a str>,
    pub is_package: bool,
    pub type_param_bindings: TypeParamBindings<'a>,
}

impl<'a> CodegenCx<'a> {
    pub fn new(db: &'a Database, local_def_source: Option<&'a str>, is_package: bool) -> Self {
        Self {
            db,
            local_def_source,
            is_package,
            type_param_bindings: TypeParamBindings::empty(),
        }
    }

    pub fn new_with_type_params(
        db: &'a Database,
        local_def_source: Option<&'a str>,
        is_package: bool,
        type_params: &'a [impl AsRef<str>],
    ) -> Self {
        Self {
            db,
            local_def_source,
            is_package,
            type_param_bindings: TypeParamBindings::from_type_params(type_params),
        }
    }

    pub fn rust_import_qualified(
        &self,
        qualified_identifier: &QualifiedIdentifier,
    ) -> genco::lang::rust::Tokens {
        MprotoRust::import_qualified(self.db, self.local_def_source, qualified_identifier)
    }

    pub fn js_import_qualified(
        &self,
        qualified_identifier: &QualifiedIdentifier,
    ) -> genco::lang::js::Tokens {
        MprotoJs::import_qualified(self.db, self.local_def_source, qualified_identifier)
    }

    pub fn with_type_param_bindings(&self, type_param_bindings: &TypeParamBindings<'a>) -> Self {
        Self {
            db: self.db,
            local_def_source: self.local_def_source,
            is_package: self.is_package,
            type_param_bindings: type_param_bindings.clone(),
        }
    }

    pub fn with_type_params(&self, type_params: &'a [impl AsRef<str>]) -> Self {
        Self {
            db: self.db,
            local_def_source: self.local_def_source,
            is_package: self.is_package,
            type_param_bindings: TypeParamBindings::from_type_params(type_params),
        }
    }

    pub fn with_type_args(
        &'a self,
        type_params: &'a [impl AsRef<str>],
        type_args: &'a [Type],
    ) -> Self {
        Self {
            db: self.db,
            local_def_source: self.local_def_source,
            is_package: self.is_package,
            type_param_bindings: TypeParamBindings::from_type_args(
                &self.type_param_bindings,
                type_params,
                type_args,
            ),
        }
    }

    pub fn resolve_type_param_binding(&self, type_name: &str) -> Option<TypeParamBinding<'a>> {
        self.type_param_bindings.resolve(type_name)
    }

    pub fn resolve_type(&self, ident: &QualifiedIdentifier) -> Option<ResolvedType<'a>> {
        if ident.module.is_none() {
            if let Some(type_param_binding) = self.resolve_type_param_binding(&ident.name) {
                return match type_param_binding {
                    TypeParamBinding::Unbound => Some(ResolvedType::UnboundParam),
                    TypeParamBinding::Bound { value, binding_cx } => {
                        Some(ResolvedType::BoundParam { value, binding_cx })
                    }
                };
            }
        }

        if let Some(type_def) = self.db.lookup_type_def(ident) {
            Some(ResolvedType::Defined(type_def))
        } else {
            None
        }
    }
}

pub enum ResolvedType<'a> {
    /// Defined type
    Defined(&'a TypeDef),
    /// Unbound type parameter
    UnboundParam,
    /// Bound type parameter
    BoundParam {
        value: &'a Type,
        binding_cx: &'a TypeParamBindings<'a>,
    },
}

#[derive(Copy, Clone)]
pub enum TypeParamBinding<'a> {
    Unbound,
    Bound {
        value: &'a Type,
        binding_cx: &'a TypeParamBindings<'a>,
    },
}

#[derive(Clone)]
pub struct TypeParamBindings<'a> {
    bindings: Vec<(&'a str, TypeParamBinding<'a>)>,
}

impl<'a> TypeParamBindings<'a> {
    pub fn empty() -> Self {
        Self {
            bindings: Vec::new(),
        }
    }

    pub fn from_type_params(type_params: &'a [impl AsRef<str>]) -> Self {
        Self {
            bindings: type_params
                .iter()
                .map(|x| (x.as_ref(), TypeParamBinding::Unbound))
                .collect(),
        }
    }

    pub fn from_type_args(
        parent_bindings: &'a TypeParamBindings<'a>,
        type_params: &'a [impl AsRef<str>],
        type_args: &'a [Type],
    ) -> Self {
        Self {
            bindings: type_params
                .iter()
                .map(|x| x.as_ref())
                .zip(type_args.iter().map(|type_arg| TypeParamBinding::Bound {
                    value: type_arg,
                    binding_cx: parent_bindings,
                }))
                .collect(),
        }
    }

    pub fn resolve(&self, type_name: &str) -> Option<TypeParamBinding<'a>> {
        for &(binding_name, binding) in &self.bindings {
            if binding_name == type_name {
                return Some(binding);
            }
        }

        None
    }
}

pub fn type_uses_param(cx: &CodegenCx, ty: &Type, param_name: &str) -> bool {
    match ty {
        Type::Primitive(PrimitiveType::Box(inner_ty)) => type_uses_param(cx, inner_ty, param_name),
        Type::Primitive(PrimitiveType::List(item_ty)) => type_uses_param(cx, item_ty, param_name),
        Type::Primitive(PrimitiveType::Option(inner_ty)) => {
            type_uses_param(cx, inner_ty, param_name)
        }
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => {
            type_uses_param(cx, ok_ty, param_name) || type_uses_param(cx, err_ty, param_name)
        }
        Type::Defined { ident, args } => match cx.resolve_type(ident) {
            Some(ResolvedType::Defined(_)) => {
                for arg in args {
                    if type_uses_param(cx, arg, param_name) {
                        return true;
                    }
                }

                false
            }
            Some(ResolvedType::UnboundParam) => ident == &QualifiedIdentifier::local(param_name),
            Some(ResolvedType::BoundParam { .. }) => false,
            None => {
                panic!("type_uses_type_param failed to resolve type: {:?}", ident);
            }
        },
        _ => false,
    }
}

pub fn type_uses_type_param(cx: &CodegenCx, ty: &Type) -> bool {
    match ty {
        Type::Primitive(PrimitiveType::Box(inner_ty)) => type_uses_type_param(cx, inner_ty),
        Type::Primitive(PrimitiveType::List(item_ty)) => type_uses_type_param(cx, item_ty),
        Type::Primitive(PrimitiveType::Option(inner_ty)) => type_uses_type_param(cx, inner_ty),
        Type::Primitive(PrimitiveType::Result(ok_ty, err_ty)) => {
            type_uses_type_param(cx, ok_ty) || type_uses_type_param(cx, err_ty)
        }
        Type::Defined { ident, args } => match cx.resolve_type(ident) {
            Some(ResolvedType::Defined(_)) => {
                for arg in args {
                    if type_uses_type_param(cx, arg) {
                        return true;
                    }
                }

                false
            }
            Some(ResolvedType::UnboundParam) => true,
            Some(ResolvedType::BoundParam { .. }) => false,
            None => {
                panic!("type_uses_type_param failed to resolve type: {:?}", ident);
            }
        },
        _ => false,
    }
}
