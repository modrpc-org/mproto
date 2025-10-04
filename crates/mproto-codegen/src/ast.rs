#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub struct TypeDefId(pub(crate) usize);

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Struct {
    pub fields: Vec<NamedField>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct NamedField {
    pub name: String,
    pub ty: Type,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct Enum {
    pub variants: Vec<(String, EnumVariant)>,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum EnumVariant {
    Empty,
    NamedFields { fields: Vec<NamedField> },
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum PrimitiveType {
    Void,
    U8,
    U16,
    U32,
    U64,
    U128,
    I8,
    I16,
    I32,
    I64,
    I128,
    Bool,
    F32,
    String,
    Box(Box<Type>),
    List(Box<Type>),
    Option(Box<Type>),
    Result(Box<Type>, Box<Type>),
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum Type {
    Primitive(PrimitiveType),
    Defined {
        ident: QualifiedIdentifier,
        args: Vec<Type>,
    },
}

impl Type {
    pub fn local(name: impl Into<String>) -> Self {
        Self::Defined {
            ident: QualifiedIdentifier::local(name),
            args: Vec::new(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub enum TypeBody {
    Struct(Struct),
    Enum(Enum),
}

impl TypeBody {
    pub fn is_struct(&self) -> bool {
        if let Self::Struct(_) = self {
            true
        } else {
            false
        }
    }

    pub fn is_enum(&self) -> bool {
        if let Self::Enum(_) = self {
            true
        } else {
            false
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct TypeDef {
    pub name: String,
    pub params: Vec<String>,
    pub body: TypeBody,
}

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
pub struct QualifiedIdentifier {
    pub name: String,
    pub module: Option<String>,
}

impl QualifiedIdentifier {
    pub fn local(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            module: None,
        }
    }
}
