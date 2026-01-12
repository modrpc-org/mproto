use nom::{
    branch::alt,
    bytes::complete::{tag, take_while1},
    character::complete::{char, multispace0},
    combinator::{cut, map, opt},
    error::{context, ParseError},
    multi::separated_list0,
    sequence::{preceded, separated_pair, terminated},
    IResult,
};

use crate::ast::{
    Enum, EnumVariant, NamedField, PrimitiveType, QualifiedIdentifier, Struct, Type, TypeBody,
    TypeDef,
};

fn identifier(i: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(i)
}

fn qualified_identifier(i: &str) -> IResult<&str, QualifiedIdentifier> {
    let (i, maybe_module): (_, Option<&str>) = opt(|i| {
        let (i, module) = identifier(i)?;
        let (i, _) = char('.')(i)?;
        Ok((i, module))
    })(i)?;

    let (i, name) = identifier(i)?;

    Ok((
        i,
        QualifiedIdentifier {
            name: name.to_string(),
            module: maybe_module.map(|x| x.to_string()),
        },
    ))
}

fn opt_trailing_comma<I, O, E: ParseError<I>>(
    f: impl FnMut(I) -> IResult<I, O, E>,
) -> impl FnMut(I) -> IResult<I, O, E>
where
    I: nom::Slice<std::ops::RangeFrom<usize>> + nom::InputIter + Clone,
    <I as nom::InputIter>::Item: nom::AsChar,
{
    terminated(f, opt(char(',')))
}

fn box_ty(i: &str) -> IResult<&str, PrimitiveType> {
    let (i, _) = tag("box")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("<")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, inner_ty) = ty(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(">")(i)?;

    Ok((i, PrimitiveType::Box(inner_ty.into())))
}

fn list_ty(i: &str) -> IResult<&str, PrimitiveType> {
    let (i, _) = char('[')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, ty) = ty(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = char(']')(i)?;

    Ok((i, PrimitiveType::List(ty.into())))
}

fn option_ty(i: &str) -> IResult<&str, PrimitiveType> {
    let (i, _) = tag("option")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("<")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, ty) = ty(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(">")(i)?;

    Ok((i, PrimitiveType::Option(ty.into())))
}

fn result_ty(i: &str) -> IResult<&str, PrimitiveType> {
    let (i, _) = tag("result")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag("<")(i)?;
    let (i, ok_ty) = ty(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = char(',')(i)?;
    let (i, _) = multispace0(i)?;
    let (i, err_ty) = ty(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = opt(char(','))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(">")(i)?;

    Ok((i, PrimitiveType::Result(ok_ty.into(), err_ty.into())))
}

fn builtin_ty(i: &str) -> IResult<&str, PrimitiveType> {
    alt((
        map(tag("void"), |_| PrimitiveType::Void),
        map(tag("u8"), |_| PrimitiveType::U8),
        map(tag("u16"), |_| PrimitiveType::U16),
        map(tag("u32"), |_| PrimitiveType::U32),
        map(tag("u64"), |_| PrimitiveType::U64),
        map(tag("u128"), |_| PrimitiveType::U128),
        map(tag("i8"), |_| PrimitiveType::I8),
        map(tag("i16"), |_| PrimitiveType::I16),
        map(tag("i32"), |_| PrimitiveType::I32),
        map(tag("i64"), |_| PrimitiveType::I64),
        map(tag("i128"), |_| PrimitiveType::I128),
        map(tag("f32"), |_| PrimitiveType::F32),
        map(tag("f64"), |_| PrimitiveType::F64),
        map(tag("bool"), |_| PrimitiveType::Bool),
        map(tag("string"), |_| PrimitiveType::String),
        box_ty,
        list_ty,
        option_ty,
        result_ty,
    ))(i)
}

pub fn type_params_list(i: &str) -> IResult<&str, Vec<String>> {
    let (i, _) = tag("<")(i)?;
    let (i, params) = separated_list0(
        preceded(multispace0, char(',')),
        preceded(multispace0, identifier),
    )(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = opt(char(','))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(">")(i)?;

    let params = params.into_iter().map(|p| p.into()).collect();

    Ok((i, params))
}

pub fn type_args_list(i: &str) -> IResult<&str, Vec<Type>> {
    let (i, _) = tag("<")(i)?;
    let (i, args) =
        separated_list0(preceded(multispace0, char(',')), preceded(multispace0, ty))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = opt(char(','))(i)?;
    let (i, _) = multispace0(i)?;
    let (i, _) = tag(">")(i)?;

    let args = args.into_iter().map(|p| p.into()).collect();

    Ok((i, args))
}

pub fn struct_def(i: &str) -> IResult<&str, TypeDef> {
    let (i, _) = tag("struct")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, name) = identifier(i)?;
    let (i, _) = multispace0(i)?;
    let (i, maybe_params) = opt(type_params_list)(i)?;
    let (i, _) = multispace0(i)?;
    let (i, fields) = named_fields(i)?;

    let params = maybe_params.unwrap_or(Vec::new());

    let type_def = TypeDef {
        name: name.into(),
        params,
        body: TypeBody::Struct(Struct { fields }),
    };

    Ok((i, type_def))
}

fn enum_def<'a>(i: &'a str) -> IResult<&'a str, TypeDef> {
    let (i, _) = tag("enum")(i)?;
    let (i, _) = multispace0(i)?;
    let (i, name) = identifier(i)?;
    let (i, _) = multispace0(i)?;
    let (i, maybe_params) = opt(type_params_list)(i)?;
    let (i, _) = multispace0(i)?;
    let (i, variants) = enum_variants(i)?;

    let params = maybe_params.unwrap_or(Vec::new());

    let type_def = TypeDef {
        name: name.into(),
        params,
        body: TypeBody::Enum(Enum { variants }),
    };

    Ok((i, type_def))
}

fn enum_variants<'a>(i: &'a str) -> IResult<&'a str, Vec<(String, EnumVariant)>> {
    context(
        "map",
        preceded(
            char('{'),
            cut(terminated(
                opt_trailing_comma(map(
                    separated_list0(
                        preceded(multispace0, char(',')),
                        preceded(multispace0, enum_variant),
                    ),
                    |tuple_vec| {
                        tuple_vec
                            .into_iter()
                            .map(|(name, variant)| (name.into(), variant))
                            .collect()
                    },
                )),
                preceded(multispace0, char('}')),
            )),
        ),
    )(i)
}

fn enum_variant<'a>(i: &'a str) -> IResult<&'a str, (&'a str, EnumVariant)> {
    alt((
        separated_pair(
            identifier,
            multispace0,
            map(named_fields, |fields| EnumVariant::NamedFields { fields }),
        ),
        map(identifier, |x| (x, EnumVariant::Empty)),
    ))(i)
}

pub fn defined_ty<'a>(i: &'a str) -> IResult<&'a str, Type> {
    let (i, ident) = qualified_identifier(i)?;
    let (i, _) = multispace0(i)?;
    let (i, maybe_args) = opt(type_args_list)(i)?;

    let args = maybe_args.unwrap_or(Vec::new());
    let defined_type = Type::Defined { ident, args };

    Ok((i, defined_type))
}

pub fn ty<'a>(i: &'a str) -> IResult<&'a str, Type> {
    alt((map(builtin_ty, |x| Type::Primitive(x)), defined_ty))(i)
}

pub fn type_def<'a>(i: &'a str) -> IResult<&'a str, TypeDef> {
    alt((struct_def, enum_def))(i)
}

fn named_field<'a>(i: &'a str) -> IResult<&'a str, (&'a str, Type)> {
    separated_pair(
        identifier,
        cut(preceded(multispace0, char(':'))),
        preceded(multispace0, ty),
    )(i)
}

fn named_fields<'a>(i: &'a str) -> IResult<&'a str, Vec<NamedField>> {
    context(
        "map",
        preceded(
            char('{'),
            cut(terminated(
                opt_trailing_comma(map(
                    separated_list0(
                        preceded(multispace0, char(',')),
                        preceded(multispace0, named_field),
                    ),
                    |tuple_vec| {
                        tuple_vec
                            .into_iter()
                            .map(|(k, v)| NamedField {
                                name: k.to_owned(),
                                ty: v,
                            })
                            .collect()
                    },
                )),
                preceded(multispace0, char('}')),
            )),
        ),
    )(i)
}

pub fn root<'a>(i: &'a str) -> IResult<&'a str, Vec<TypeDef>> {
    separated_list0(multispace0, type_def)(i)
}

pub fn parse_file(path: impl AsRef<std::path::Path>) -> Result<Vec<TypeDef>, String> {
    use std::io::Read;

    // Open input file
    let Ok(mut file) = std::fs::File::open(path.as_ref()) else {
        return Err(format!("Failed to open file '{}'", path.as_ref().display()));
    };

    // Load file to string
    let mut file_str = String::new();
    if let Err(e) = file.read_to_string(&mut file_str) {
        return Err(format!(
            "Failed to read file '{}': {}",
            path.as_ref().display(),
            e
        ));
    }

    // Remove comments
    let mut schema_str = file_str.lines()
        .map(|line| {
            if let Some(index) = line.find("//") {
                // Return the slice from the start of the line up to the comment marker
                &line[..index]
            } else {
                // If no comment is found, return the entire line
                line
            }
        })
        .collect::<Vec<&str>>()
        .join("\n");
    schema_str += "\n";

    let (_, type_defs) = root(&schema_str).map_err(|e| format!("mproto schema parse error: {e}"))?;

    Ok(type_defs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builtin_u8() {
        let data = "u8";
        let (_, parsed) = builtin_ty(data).unwrap();

        assert_eq!(parsed, PrimitiveType::U8);
    }

    #[test]
    fn test_list_u8() {
        let data = "[u8]";
        let (_, parsed) = list_ty(data).unwrap();

        assert_eq!(
            parsed,
            PrimitiveType::List(Box::new(Type::Primitive(PrimitiveType::U8))),
        );
    }

    #[test]
    fn test_box_u8() {
        let data = "box<u8>";
        let (_, parsed) = box_ty(data).unwrap();

        assert_eq!(
            parsed,
            PrimitiveType::Box(Box::new(Type::Primitive(PrimitiveType::U8))),
        );
    }

    #[test]
    fn test_option_u8() {
        let data = "option<u8>";
        let (_, parsed) = option_ty(data).unwrap();

        assert_eq!(
            parsed,
            PrimitiveType::Option(Box::new(Type::Primitive(PrimitiveType::U8))),
        );
    }

    #[test]
    fn test_result() {
        let data = "result<void, string>";
        let (_, parsed) = result_ty(data).unwrap();

        assert_eq!(
            parsed,
            PrimitiveType::Result(
                Box::new(Type::Primitive(PrimitiveType::Void)),
                Box::new(Type::Primitive(PrimitiveType::String)),
            ),
        );
    }

    #[test]
    fn test_struct_named_fields() {
        use PrimitiveType::*;

        let data = "struct Foo { bar : u32, baz : i8 }";
        let (_, parsed) = struct_def(data).unwrap();

        assert_eq!(
            parsed,
            TypeDef {
                name: "Foo".into(),
                params: vec![],
                body: TypeBody::Struct(Struct {
                    fields: vec![
                        NamedField {
                            name: "bar".into(),
                            ty: Type::Primitive(U32)
                        },
                        NamedField {
                            name: "baz".into(),
                            ty: Type::Primitive(I8)
                        },
                    ]
                }),
            }
        );
    }

    #[test]
    fn test_struct_type_param() {
        use PrimitiveType::*;

        let data = "struct Foo <T, F>  { bar : u32, baz : i8 }";
        let (_, parsed) = struct_def(data).unwrap();

        assert_eq!(
            parsed,
            TypeDef {
                name: "Foo".into(),
                params: vec!["T".into(), "F".into()],
                body: TypeBody::Struct(Struct {
                    fields: vec![
                        NamedField {
                            name: "bar".into(),
                            ty: Type::Primitive(U32)
                        },
                        NamedField {
                            name: "baz".into(),
                            ty: Type::Primitive(I8)
                        },
                    ]
                }),
            }
        );
    }

    #[test]
    fn test_defined_type() {
        use PrimitiveType::*;

        let data = "struct Foo { bar : bar_proto.Bar, baz : i8 }";
        let (_, parsed) = struct_def(data).unwrap();

        assert_eq!(
            parsed,
            TypeDef {
                name: "Foo".into(),
                params: vec![],
                body: TypeBody::Struct(Struct {
                    fields: vec![
                        NamedField {
                            name: "bar".into(),
                            ty: Type::Defined {
                                ident: QualifiedIdentifier {
                                    module: Some("bar_proto".into()),
                                    name: "Bar".into(),
                                },
                                args: vec![],
                            },
                        },
                        NamedField {
                            name: "baz".into(),
                            ty: Type::Primitive(I8)
                        },
                    ]
                }),
            }
        );
    }

    #[test]
    fn test_enum_named_fields() {
        use PrimitiveType::*;

        let data = "enum Foo { Bar { x: u32, y: u8 }, Baz { bip: i8 } }";
        let (_, parsed) = enum_def(data).unwrap();

        assert_eq!(
            parsed,
            TypeDef {
                name: "Foo".into(),
                params: vec![],
                body: TypeBody::Enum(Enum {
                    variants: vec![
                        (
                            "Bar".into(),
                            EnumVariant::NamedFields {
                                fields: vec![
                                    NamedField {
                                        name: "x".into(),
                                        ty: Type::Primitive(U32)
                                    },
                                    NamedField {
                                        name: "y".into(),
                                        ty: Type::Primitive(U8)
                                    },
                                ],
                            }
                        ),
                        (
                            "Baz".into(),
                            EnumVariant::NamedFields {
                                fields: vec![NamedField {
                                    name: "bip".into(),
                                    ty: Type::Primitive(I8)
                                },],
                            }
                        ),
                    ]
                }),
            }
        );
    }
}
