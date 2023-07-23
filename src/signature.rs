/*
JavaTypeSignature:
    ReferenceTypeSignature
    BaseType

BaseType:
    (one of)
    B C D F I J S Z

ReferenceTypeSignature:
    ClassTypeSignature
    TypeVariableSignature
    ArrayTypeSignature
ClassTypeSignature:
    L [PackageSpecifier] SimpleClassTypeSignature {ClassTypeSignatureSuffix} ;
    PackageSpecifier:
    Identifier / {PackageSpecifier}
SimpleClassTypeSignature:
    Identifier [TypeArguments]
TypeArguments:
    < TypeArgument {TypeArgument} >
TypeArgument:
    [WildcardIndicator] ReferenceTypeSignature
    *
WildcardIndicator:
    +
    -
ClassTypeSignatureSuffix:
    . SimpleClassTypeSignature
TypeVariableSignature:
    T Identifier ;
ArrayTypeSignature:
    [ JavaTypeSignature

ClassSignature:
    [TypeParameters] SuperclassSignature {SuperinterfaceSignature}
TypeParameters:
    < TypeParameter {TypeParameter} >
TypeParameter:
    Identifier ClassBound {InterfaceBound}
ClassBound:
    : [ReferenceTypeSignature]
InterfaceBound:
    : ReferenceTypeSignature
SuperclassSignature:
    ClassTypeSignature
SuperinterfaceSignature:
    ClassTypeSignature

MethodSignature:
    [TypeParameters] ( {JavaTypeSignature} ) Result {ThrowsSignature}
Result:
    JavaTypeSignature
    VoidDescriptor
ThrowsSignature:
    ^ ClassTypeSignature
    ^ TypeVariableSignature
VoidDescriptor:
    V
FieldSignature:
    ReferenceTypeSignature
*/

use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete::{char, one_of},
    combinator::{opt, peek},
    multi::{many0, many1},
    sequence::pair,
    Err, IResult,
};

// JavaTypeSignature
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeSignature {
    Reference(ReferenceTypeSignature),
    Base(BaseType),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseType {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReferenceTypeSignature {
    TypeVariable(TypeVariableSignature),
    Class(ClassTypeSignature),
    Array(ArrayTypeSignature),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassTypeSignature {
    pub package_specifier: Option<String>,
    pub simple_class_type_signature: SimpleClassTypeSignature,
    pub class_type_signature_suffixes: Vec<SimpleClassTypeSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SimpleClassTypeSignature {
    pub identifier: String,
    pub type_arguments: Option<Vec<TypeArgument>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeArgument {
    ReferenceType(Option<WildcardIndicator>, ReferenceTypeSignature),
    Any,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WildcardIndicator {
    Plus,
    Minus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeVariableSignature {
    pub identifier: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayTypeSignature {
    pub java_type_signature: Box<TypeSignature>,
}

// ClassSignature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClassSignature {
    pub type_parameters: Option<Vec<TypeParameter>>,
    pub superclass_signature: ClassTypeSignature,
    pub superinterface_signatures: Vec<ClassTypeSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeParameter {
    pub identifier: String,
    pub class_bound: Option<ReferenceTypeSignature>,
    pub interface_bounds: Vec<ReferenceTypeSignature>,
}

// MethodSignature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSignature {
    pub type_parameters: Option<Vec<TypeParameter>>,
    pub parameters: Vec<TypeSignature>,
    pub result: Result,
    pub throws_signatures: Vec<ThrowsSignature>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Result {
    JavaTypeSignature(TypeSignature),
    VoidDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ThrowsSignature {
    ClassTypeSignature(ClassTypeSignature),
    TypeVariableSignature(TypeVariableSignature),
}

// FieldSignature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldSignature {
    pub reference_type_signature: ReferenceTypeSignature,
}

pub fn parse_type_signature(input: &str) -> IResult<&str, TypeSignature> {
    alt((parse_base_type, parse_reference_type_signature))(input)
}

fn parse_base_type(input: &str) -> IResult<&str, TypeSignature> {
    let (input, bt) = one_of("BCDFIJSZ")(input)?;
    let bt = match bt {
        'B' => BaseType::Byte,
        'C' => BaseType::Char,
        'D' => BaseType::Double,
        'F' => BaseType::Float,
        'I' => BaseType::Int,
        'J' => BaseType::Long,
        'S' => BaseType::Short,
        'Z' => BaseType::Boolean,
        _ => unreachable!(),
    };
    Ok((input, TypeSignature::Base(bt)))
}

fn parse_reference_type_signature(input: &str) -> IResult<&str, TypeSignature> {
    alt((
        parse_class_type_signature,
        parse_type_variable_signature,
        parse_array_type_signature,
    ))(input)
}

fn parse_class_type_signature(input: &str) -> IResult<&str, TypeSignature> {
    let (input, _) = char('L')(input)?;
    let (input, package_specifier) = parse_package_specifier(input)?;
    let (input, simple_class_type_signature) = parse_simple_class_type_signature(input)?;
    let (input, class_type_signature_suffixes) = parse_class_type_signature_suffixes(input)?;
    let (input, _) = char(';')(input)?;
    Ok((
        input,
        TypeSignature::Reference(ReferenceTypeSignature::Class(ClassTypeSignature {
            package_specifier,
            simple_class_type_signature,
            class_type_signature_suffixes,
        })),
    ))
}

fn parse_package_specifier(input: &str) -> IResult<&str, Option<String>> {
    let (input, package_specifier) = many0(pair(parse_identifier, tag("/")))(input)?;
    let package_specifier = package_specifier
        .into_iter()
        .map(|(ident, _)| ident)
        .collect::<Vec<_>>()
        .join(".");
    let package_specifier = if package_specifier.is_empty() {
        None
    } else {
        Some(package_specifier)
    };
    Ok((input, package_specifier))
}

fn parse_identifier(input: &str) -> IResult<&str, String> {
    // input char is JavaLetter, it includes A-Z, a-z and ASCII dollar sign and underscore
    // if input char is not first char, it also includes 0-9
    // indeed, JavaLetter includes the large set for Japanese, Chinese, etc. But we ignore them and if this programe meets them, it will panic

    let (input, first_char) =
        one_of("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$")(input)?;
    let mut identifier = first_char.to_string();

    let mut input = input;
    while let Ok((i, c)) = one_of::<_, _, nom::error::Error<_>>(
        "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz_$0123456789",
    )(input)
    {
        identifier.push(c);
        input = i;
        if input.is_empty() {
            break;
        }
    }

    Ok((input, identifier))
}

fn parse_simple_class_type_signature(input: &str) -> IResult<&str, SimpleClassTypeSignature> {
    let (input, identifier) = parse_identifier(input)?;

    let (input, type_arguments) = parse_type_arguments(input)?;
    Ok((
        input,
        SimpleClassTypeSignature {
            identifier,
            type_arguments,
        },
    ))
}

fn parse_type_arguments(input: &str) -> IResult<&str, Option<Vec<TypeArgument>>> {
    let Ok((input, _)) = char::<_, nom::error::Error<_>>('<')(input) else {
        return Ok((input, None));
    };
    let (input, type_arguments) = many1(parse_type_argument)(input)?;
    let (input, _) = char('>')(input)?;

    Ok((input, Some(type_arguments)))
}

fn parse_type_argument(input: &str) -> IResult<&str, TypeArgument> {
    alt((parse_any_type_argument, parse_reference_type_argument))(input)
}

fn parse_any_type_argument(input: &str) -> IResult<&str, TypeArgument> {
    let (input, _) = char('*')(input)?;
    Ok((input, TypeArgument::Any))
}

fn parse_reference_type_argument(input: &str) -> IResult<&str, TypeArgument> {
    let (input, wildcard_indicator) = opt(one_of("+-"))(input)?;
    let wildcard_indicator = match wildcard_indicator {
        Some('+') => Some(WildcardIndicator::Plus),
        Some('-') => Some(WildcardIndicator::Minus),
        Some(_) => unreachable!(),
        None => None,
    };
    let (input, TypeSignature::Reference(reference_type_signature)) =
        parse_reference_type_signature(input)?
    else {
        unreachable!()
    };
    Ok((
        input,
        TypeArgument::ReferenceType(wildcard_indicator, reference_type_signature),
    ))
}

fn parse_class_type_signature_suffixes(
    input: &str,
) -> IResult<&str, Vec<SimpleClassTypeSignature>> {
    let (input, class_type_signature_suffixes) =
        many0(pair(char('.'), parse_simple_class_type_signature))(input)?;
    let class_type_signature_suffixes = class_type_signature_suffixes
        .into_iter()
        .map(|(_, sig)| sig)
        .collect::<Vec<_>>();
    Ok((input, class_type_signature_suffixes))
}

fn parse_type_variable_signature(input: &str) -> IResult<&str, TypeSignature> {
    let (input, _) = char('T')(input)?;
    let (input, identifier) = parse_identifier(input)?;
    let (input, _) = char(';')(input)?;
    Ok((
        input,
        TypeSignature::Reference(ReferenceTypeSignature::TypeVariable(
            TypeVariableSignature { identifier },
        )),
    ))
}

fn parse_array_type_signature(input: &str) -> IResult<&str, TypeSignature> {
    let (input, _) = char('[')(input)?;
    let (input, java_type_signature) = parse_type_signature(input)?;
    Ok((
        input,
        TypeSignature::Reference(ReferenceTypeSignature::Array(ArrayTypeSignature {
            java_type_signature: Box::new(java_type_signature),
        })),
    ))
}

pub fn parse_class_signature(input: &str) -> IResult<&str, ClassSignature> {
    let (input, type_parameters) = parse_type_parameters(input)?;
    let (input, superclass_signature) = parse_superclass_signature(input)?;
    let (input, superinterface_signatures) = many0(parse_superinterface_signature)(input)?;
    Ok((
        input,
        ClassSignature {
            type_parameters,
            superclass_signature,
            superinterface_signatures,
        },
    ))
}

fn parse_type_parameters(input: &str) -> IResult<&str, Option<Vec<TypeParameter>>> {
    let Ok((input, _)) = char::<_, nom::error::Error<_>>('<')(input) else {
        return Ok((input, None));
    };
    let (input, type_parameters) = many1(parse_type_parameter)(input)?;
    let (input, _) = char('>')(input)?;
    Ok((input, Some(type_parameters)))
}

fn parse_type_parameter(input: &str) -> IResult<&str, TypeParameter> {
    let (input, identifier) = parse_identifier(input)?;
    let (input, class_bound) = parse_class_bound(input)?;
    let (input, interface_bounds) = many0(parse_interface_bound)(input)?;
    Ok((
        input,
        TypeParameter {
            identifier,
            class_bound,
            interface_bounds,
        },
    ))
}

fn parse_class_bound(input: &str) -> IResult<&str, Option<ReferenceTypeSignature>> {
    let (input, _) = char(':')(input)?;
    if let Ok((input, _)) = peek(one_of::<_, _, nom::error::Error<_>>(":>"))(input) {
        return Ok((input, None));
    }

    let (input, TypeSignature::Reference(reference_type_signature)) =
        parse_reference_type_signature(input)?
    else {
        unreachable!()
    };
    Ok((input, Some(reference_type_signature)))
}

fn parse_interface_bound(input: &str) -> IResult<&str, ReferenceTypeSignature> {
    let Ok((input, _)) = char::<_, nom::error::Error<_>>(':')(input) else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };
    let (input, TypeSignature::Reference(reference_type_signature)) =
        parse_reference_type_signature(input)?
    else {
        unreachable!()
    };
    Ok((input, reference_type_signature))
}

fn parse_superclass_signature(input: &str) -> IResult<&str, ClassTypeSignature> {
    let Ok((input, TypeSignature::Reference(ReferenceTypeSignature::Class(class_type_signature)))) =
        parse_class_type_signature(input)
    else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };
    Ok((input, class_type_signature))
}

fn parse_superinterface_signature(input: &str) -> IResult<&str, ClassTypeSignature> {
    let Ok((input, TypeSignature::Reference(ReferenceTypeSignature::Class(class_type_signature)))) =
        parse_class_type_signature(input)
    else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };
    Ok((input, class_type_signature))
}

pub fn parse_method_signature(input: &str) -> IResult<&str, MethodSignature> {
    let (input, type_parameters) = parse_type_parameters(input)?;
    let (input, _) = char('(')(input)?;
    let (input, parameters) = many0(parse_type_signature)(input)?;
    let (input, _) = char(')')(input)?;
    let (input, result) = parse_result(input)?;
    let (input, throws_signatures) = many0(parse_throws_signature)(input)?;
    Ok((
        input,
        MethodSignature {
            type_parameters,
            parameters,
            result,
            throws_signatures,
        },
    ))
}

fn parse_result(input: &str) -> IResult<&str, Result> {
    if tag::<_, _, nom::error::Error<_>>("V")(input).is_ok() {
        Ok((input, Result::VoidDescriptor))
    } else {
        parse_type_signature(input).map(|(i, ts)| (i, Result::JavaTypeSignature(ts)))
    }
}

fn parse_throws_signature(input: &str) -> IResult<&str, ThrowsSignature> {
    let Ok((input, _)) = char::<_, nom::error::Error<_>>('^')(input) else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };
    if let Ok((input, TypeSignature::Reference(ReferenceTypeSignature::Class(signature)))) =
        parse_reference_type_signature(input)
    {
        Ok((input, ThrowsSignature::ClassTypeSignature(signature)))
    } else if let Ok((
        input,
        TypeSignature::Reference(ReferenceTypeSignature::TypeVariable(signature)),
    )) = parse_type_variable_signature(input)
    {
        Ok((input, ThrowsSignature::TypeVariableSignature(signature)))
    } else {
        Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )))
    }
}

pub fn parse_field_signature(input: &str) -> IResult<&str, FieldSignature> {
    let Ok((input, TypeSignature::Reference(reference_type_signature))) =
        parse_reference_type_signature(input)
    else {
        return Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Char,
        )));
    };

    Ok((
        input,
        FieldSignature {
            reference_type_signature,
        },
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_java_type_signature() {
        let test_cases = [
            ("B", TypeSignature::Base(BaseType::Byte)),
            (
                "Ljava/lang/String;",
                TypeSignature::Reference(ReferenceTypeSignature::Class(ClassTypeSignature {
                    package_specifier: Some("java.lang".to_string()),
                    simple_class_type_signature: SimpleClassTypeSignature {
                        identifier: "String".to_string(),
                        type_arguments: None,
                    },
                    class_type_signature_suffixes: vec![],
                })),
            ),
            (
                "Ljava/util/List<Ljava/lang/String;>;",
                TypeSignature::Reference(ReferenceTypeSignature::Class(ClassTypeSignature {
                    package_specifier: Some("java.util".to_string()),
                    simple_class_type_signature: SimpleClassTypeSignature {
                        identifier: "List".to_string(),
                        type_arguments: Some(vec![TypeArgument::ReferenceType(
                            None,
                            ReferenceTypeSignature::Class(ClassTypeSignature {
                                package_specifier: Some("java.lang".to_string()),
                                simple_class_type_signature: SimpleClassTypeSignature {
                                    identifier: "String".to_string(),
                                    type_arguments: None,
                                },
                                class_type_signature_suffixes: vec![],
                            }),
                        )]),
                    },
                    class_type_signature_suffixes: vec![],
                })),
            ),
        ];

        for (input, expect) in test_cases.iter() {
            let (_, actual) = parse_type_signature(input).unwrap();
            assert_eq!(actual, *expect);
        }
    } // input, expect

    #[test]
    fn test_class_signature() {
        let test_cases = [(
            "<T:Ljava/lang/Object;>Ljava/lang/Object;",
            ClassSignature {
                type_parameters: Some(vec![TypeParameter {
                    identifier: "T".to_string(),
                    class_bound: Some(ReferenceTypeSignature::Class(ClassTypeSignature {
                        package_specifier: Some("java.lang".to_string()),
                        simple_class_type_signature: SimpleClassTypeSignature {
                            identifier: "Object".to_string(),
                            type_arguments: None,
                        },
                        class_type_signature_suffixes: vec![],
                    })),
                    interface_bounds: vec![],
                }]),
                superclass_signature: ClassTypeSignature {
                    package_specifier: Some("java.lang".to_string()),
                    simple_class_type_signature: SimpleClassTypeSignature {
                        identifier: "Object".to_string(),
                        type_arguments: None,
                    },
                    class_type_signature_suffixes: vec![],
                },
                superinterface_signatures: vec![],
            },
        )];

        for (input, expect) in test_cases.iter() {
            let (_, actual) = parse_class_signature(input).unwrap();
            assert_eq!(actual, *expect);
        }
    }

    #[test]
    fn test_method_signature() {
        let test_cases = [
            (
                "<T:Ljava/lang/Object;>(Ljava/lang/Object;)TT;",
                MethodSignature {
                    type_parameters: Some(vec![TypeParameter {
                        identifier: "T".to_string(),
                        class_bound: Some(ReferenceTypeSignature::Class(ClassTypeSignature {
                            package_specifier: Some("java.lang".to_string()),
                            simple_class_type_signature: SimpleClassTypeSignature {
                                identifier: "Object".to_string(),
                                type_arguments: None,
                            },
                            class_type_signature_suffixes: vec![],
                        })),
                        interface_bounds: vec![],
                    }]),
                    parameters: vec![TypeSignature::Reference(ReferenceTypeSignature::Class(
                        ClassTypeSignature {
                            package_specifier: Some("java.lang".to_string()),
                            simple_class_type_signature: SimpleClassTypeSignature {
                                identifier: "Object".to_string(),
                                type_arguments: None,
                            },
                            class_type_signature_suffixes: vec![],
                        },
                    ))],
                    result: Result::JavaTypeSignature(TypeSignature::Reference(
                        ReferenceTypeSignature::TypeVariable(TypeVariableSignature {
                            identifier: "T".to_string(),
                        }),
                    )),
                    throws_signatures: vec![],
                },
            ),
            (
                "(Ljava/lang/Object;)Ljava/lang/Object;",
                MethodSignature {
                    type_parameters: None,
                    parameters: vec![TypeSignature::Reference(ReferenceTypeSignature::Class(
                        ClassTypeSignature {
                            package_specifier: Some("java.lang".to_string()),
                            simple_class_type_signature: SimpleClassTypeSignature {
                                identifier: "Object".to_string(),
                                type_arguments: None,
                            },
                            class_type_signature_suffixes: vec![],
                        },
                    ))],
                    result: Result::JavaTypeSignature(TypeSignature::Reference(
                        ReferenceTypeSignature::Class(ClassTypeSignature {
                            package_specifier: Some("java.lang".to_string()),
                            simple_class_type_signature: SimpleClassTypeSignature {
                                identifier: "Object".to_string(),
                                type_arguments: None,
                            },
                            class_type_signature_suffixes: vec![],
                        }),
                    )),
                    throws_signatures: vec![],
                },
            ),
        ];

        for (input, expect) in test_cases.iter() {
            let (_, actual) = parse_method_signature(input).unwrap();
            assert_eq!(actual, *expect);
        }
    }
}
