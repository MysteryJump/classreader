#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDescriptor {
    pub descriptor: String,
    pub ty: FieldTy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FieldTy {
    Base(BaseTy),
    Obj(ObjTy),
    Array(ArrayTy),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaseTy {
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
    Boolean,
    Void,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObjTy {
    pub class_name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArrayTy {
    pub ty: Box<FieldTy>,
    pub dims: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodDescriptor {
    pub descriptor: String,
    pub param_descs: Vec<FieldDescriptor>,
    pub ret_desc: ReturnDescriptor,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReturnDescriptor {
    TyDesc(FieldDescriptor),
    Void,
}

pub fn parse_field_descriptor(descriptor: &str) -> FieldDescriptor {
    let mut chars = descriptor.chars();
    let mut ty = FieldTy::Base(BaseTy::Void);
    let mut dims = 0;
    let mut descriptor_len = 0;

    loop {
        descriptor_len += 1;
        match chars.next() {
            Some('B') => ty = FieldTy::Base(BaseTy::Byte),
            Some('C') => ty = FieldTy::Base(BaseTy::Char),
            Some('D') => ty = FieldTy::Base(BaseTy::Double),
            Some('F') => ty = FieldTy::Base(BaseTy::Float),
            Some('I') => ty = FieldTy::Base(BaseTy::Int),
            Some('J') => ty = FieldTy::Base(BaseTy::Long),
            Some('S') => ty = FieldTy::Base(BaseTy::Short),
            Some('Z') => ty = FieldTy::Base(BaseTy::Boolean),
            Some('V') => ty = FieldTy::Base(BaseTy::Void),
            Some('L') => {
                let mut class_name = String::new();
                loop {
                    descriptor_len += 1;
                    match chars.next() {
                        Some(';') => break,
                        Some(c) => class_name.push(c),
                        None => panic!("Invalid descriptor: {}", descriptor),
                    }
                }
                ty = FieldTy::Obj(ObjTy { class_name });
            }
            Some('[') => {
                dims += 1;
                continue;
            }
            Some(_) => panic!("Invalid descriptor: {}", descriptor),
            None => break,
        }
        break;
    }

    if dims > 0 {
        ty = FieldTy::Array(ArrayTy {
            ty: Box::new(ty),
            dims,
        });
    }

    FieldDescriptor {
        descriptor: descriptor[..descriptor_len].to_string(),
        ty: match ty {
            FieldTy::Base(ty) => FieldTy::Base(ty),
            FieldTy::Obj(ty) => FieldTy::Obj(ty),
            FieldTy::Array(ty) => FieldTy::Array(ty),
        },
    }
}

pub fn parse_method_descriptor(descriptor: &str) -> MethodDescriptor {
    let mut idx = 0usize;
    let mut param_descs = Vec::new();

    if &descriptor[..=idx] != "(" {
        panic!("Invalid descriptor: {}", descriptor);
    }

    idx += 1;

    loop {
        match descriptor.chars().nth(idx) {
            Some(')') => {
                idx += 1;
                break;
            }
            Some(_) => {
                let desc = parse_field_descriptor(&descriptor[idx..]);
                idx += desc.descriptor.len();
                param_descs.push(desc);
            }
            None => panic!("Invalid descriptor: {}", descriptor),
        }
    }

    let ret_desc = match descriptor.chars().nth(idx) {
        Some('V') => ReturnDescriptor::Void,
        Some(_) => ReturnDescriptor::TyDesc(parse_field_descriptor(&descriptor[idx..])),
        None => panic!("Invalid descriptor: {}", descriptor),
    };

    MethodDescriptor {
        descriptor: descriptor.to_string(),
        param_descs,
        ret_desc,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_field_descriptor() {
        let test_cases = ["I", "Ljava/lang/Object;", "[[[D"];
        let expected = [
            FieldDescriptor {
                descriptor: "I".to_string(),
                ty: FieldTy::Base(BaseTy::Int),
            },
            FieldDescriptor {
                descriptor: "Ljava/lang/Object;".to_string(),
                ty: FieldTy::Obj(ObjTy {
                    class_name: "java/lang/Object".to_string(),
                }),
            },
            FieldDescriptor {
                descriptor: "[[[D".to_string(),
                ty: FieldTy::Array(ArrayTy {
                    ty: Box::new(FieldTy::Base(BaseTy::Double)),
                    dims: 3,
                }),
            },
        ];

        for (i, test_case) in test_cases.iter().enumerate() {
            let actual = parse_field_descriptor(test_case);
            assert_eq!(actual.descriptor, expected[i].descriptor);
            assert_eq!(actual.ty, expected[i].ty);
        }
    }

    #[test]
    fn test_method_descriptor() {
        let test_cases = ["(IDLjava/lang/Thread;)Ljava/lang/Object;"];
        let expected = [MethodDescriptor {
            descriptor: "(IDLjava/lang/Thread;)Ljava/lang/Object;".to_string(),
            param_descs: vec![
                FieldDescriptor {
                    descriptor: "I".to_string(),
                    ty: FieldTy::Base(BaseTy::Int),
                },
                FieldDescriptor {
                    descriptor: "D".to_string(),
                    ty: FieldTy::Base(BaseTy::Double),
                },
                FieldDescriptor {
                    descriptor: "Ljava/lang/Thread;".to_string(),
                    ty: FieldTy::Obj(ObjTy {
                        class_name: "java/lang/Thread".to_string(),
                    }),
                },
            ],
            ret_desc: ReturnDescriptor::TyDesc(FieldDescriptor {
                descriptor: "Ljava/lang/Object;".to_string(),
                ty: FieldTy::Obj(ObjTy {
                    class_name: "java/lang/Object".to_string(),
                }),
            }),
        }];

        for (i, test_case) in test_cases.iter().enumerate() {
            let actual = parse_method_descriptor(test_case);
            assert_eq!(actual.descriptor, expected[i].descriptor);
            assert_eq!(actual.param_descs, expected[i].param_descs);
            assert_eq!(actual.ret_desc, expected[i].ret_desc);
        }
    }
}
