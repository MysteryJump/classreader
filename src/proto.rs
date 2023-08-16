use crate::{
    component::{
        Annotation, AnnotationKind, Class, Component, ComponentKind, Field, Interface, Method,
        Module, PrimTy, Ty,
    },
    signature::{
        self, ArrayTypeSignature, BaseType, ClassTypeSignature, FieldSignature,
        ReferenceTypeSignature, TypeArgument, TypeParameter, TypeSignature, WildcardIndicator,
    },
};

pub mod component {
    include!(concat!(
        env!("OUT_DIR"),
        "/classreader_rs.proto.component.v1.rs"
    ));
}

impl From<&Vec<Component>> for component::ComponentList {
    fn from(value: &Vec<Component>) -> Self {
        Self {
            components: value.iter().map(|x| x.into()).collect::<Vec<_>>(),
        }
    }
}

impl From<&Component> for component::Component {
    fn from(value: &Component) -> Self {
        Self {
            component_kind: Some((&value.kind).into()),
            class_file_name: value.class_file_name.clone(),
        }
    }
}

impl From<&ComponentKind> for component::component::ComponentKind {
    fn from(value: &ComponentKind) -> Self {
        match value {
            ComponentKind::Class(c) => component::component::ComponentKind::Class(c.into()),
            ComponentKind::Interface(i) => component::component::ComponentKind::Interface(i.into()),
            ComponentKind::Module(m) => component::component::ComponentKind::Module(m.into()),
        }
    }
}

impl From<&Class> for component::Class {
    fn from(value: &Class) -> Self {
        let superclass = if let Some(sig) = &value.signature {
            let super_class: component::ClassType = (&sig.superclass_signature).into();
            if super_class.package_specifier == "java.lang" && super_class.identifier == "Object" {
                None
            } else {
                Some(super_class)
            }
        } else {
            value.super_class.clone().map(|x| {
                let x_split = x.split('.').collect::<Vec<_>>();
                let package_specifier = if x_split.len() != 1 {
                    x_split[..x_split.len() - 1].join(".")
                } else {
                    String::new()
                };
                let identifier = x_split[x_split.len() - 1].to_string();
                component::ClassType {
                    package_specifier,
                    identifier,
                    type_arguments: Vec::new(),
                    class_type_signature_suffixes: Vec::new(),
                }
            })
        };

        let interfaces = if let Some(sig) = &value.signature {
            sig.superinterface_signatures
                .clone()
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>()
        } else {
            value
                .interfaces
                .iter()
                .map(|x| {
                    let x_split = x.split('.').collect::<Vec<_>>();
                    let package_specifier = if x_split.len() != 1 {
                        x_split[..x_split.len() - 1].join(".")
                    } else {
                        String::new()
                    };
                    let identifier = x_split[x_split.len() - 1].to_string();
                    component::ClassType {
                        package_specifier,
                        identifier,
                        type_arguments: Vec::new(),
                        class_type_signature_suffixes: Vec::new(),
                    }
                })
                .collect::<Vec<_>>()
        };

        let type_parameters = if let Some(sig) = &value.signature {
            sig.type_parameters
                .clone()
                .unwrap_or(Vec::new())
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Self {
            qualified_name: value.qualified_name.clone(),
            superclass,
            interface_types: interfaces,
            type_parameters,
            fields: value.fields.iter().map(|x| x.into()).collect::<Vec<_>>(),
            methods: value.methods.iter().map(|x| x.into()).collect::<Vec<_>>(),
            is_enum: value.is_enum,
            is_abstract: value.is_abstract,
            annotations: value
                .annotations
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>(),
        }
    }
}

impl From<&Interface> for component::Interface {
    fn from(value: &Interface) -> Self {
        let interfaces = if let Some(sig) = &value.signature {
            sig.superinterface_signatures
                .clone()
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>()
        } else {
            value
                .interfaces
                .iter()
                .map(|x| {
                    let x_split = x.split('.').collect::<Vec<_>>();
                    let package_specifier = if x_split.len() != 1 {
                        x_split[..x_split.len() - 1].join(".")
                    } else {
                        String::new()
                    };
                    let identifier = x_split[x_split.len() - 1].to_string();
                    component::ClassType {
                        package_specifier,
                        identifier,
                        type_arguments: Vec::new(),
                        class_type_signature_suffixes: Vec::new(),
                    }
                })
                .collect::<Vec<_>>()
        };

        let type_parameters = if let Some(sig) = &value.signature {
            sig.type_parameters
                .clone()
                .unwrap_or(Vec::new())
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>()
        } else {
            Vec::new()
        };

        Self {
            qualified_name: value.qualified_name.clone(),
            interface_types: interfaces,
            type_parameters,
            fields: value.fields.iter().map(|x| x.into()).collect::<Vec<_>>(),
            methods: value.methods.iter().map(|x| x.into()).collect::<Vec<_>>(),
            is_annotation: value.is_annotation,
            annotations: value
                .annotations
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>(),
        }
    }
}

impl From<&Module> for component::Module {
    fn from(value: &Module) -> Self {
        Self {
            name: value.name.clone(),
            version: value.version.clone(),
        }
    }
}

impl From<&Method> for component::Method {
    fn from(value: &Method) -> Self {
        let method_kind = match &value.name {
            name if name == "<init>" => 1,
            name if name == "<clinit>" => 2,
            _ => 0,
        };

        if let Some(sig) = value.signature.as_ref() {
            let parameter_types = value
                .param_tys
                .iter()
                .zip(sig.parameters.iter())
                .map(|(ty, ty_sig)| convert_method_param_ty_to_proto_ty(Some(ty), ty_sig))
                .collect::<Vec<_>>();

            let type_parameters = sig
                .type_parameters
                .clone()
                .unwrap_or(Vec::new())
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>();

            Self {
                name: value.name.clone(),
                parameter_types,
                return_type: match &sig.result {
                    signature::Result::VoidDescriptor => Some(component::Type {
                        type_kind: Some(component::r#type::TypeKind::PrimitiveType(
                            component::PrimitiveType {
                                primitive_type_kind: 8,
                            },
                        )),
                    }),
                    signature::Result::JavaTypeSignature(ty) => {
                        Some(convert_method_param_ty_to_proto_ty(None, ty))
                    }
                },
                modifiers: value.modifiers.clone(),
                method_kind,
                type_parameters,
                is_static: value.is_static,
                annotations: value
                    .annotations
                    .iter()
                    .map(|x| x.into())
                    .collect::<Vec<_>>(),
            }
        } else {
            Self {
                name: value.name.clone(),
                parameter_types: value.param_tys.iter().map(|x| x.into()).collect::<Vec<_>>(),
                return_type: Some((&value.ret_ty).into()),
                modifiers: value.modifiers.clone(),
                method_kind,
                type_parameters: Vec::new(),
                is_static: value.is_static,
                annotations: value
                    .annotations
                    .iter()
                    .map(|x| x.into())
                    .collect::<Vec<_>>(),
            }
        }
    }
}

impl From<&Field> for component::Field {
    fn from(value: &Field) -> Self {
        Self {
            name: value.name.clone(),
            r#type: Some(convert_field_ty_to_proto_ty(&value.ty, &value.signature)),
            modifiers: value.modifiers.clone(),
            is_static: value.is_static,
            annotations: value
                .annotations
                .iter()
                .map(|x| x.into())
                .collect::<Vec<_>>(),
        }
    }
}

impl From<&PrimTy> for component::PrimitiveType {
    fn from(value: &PrimTy) -> Self {
        let val = match value {
            PrimTy::Byte => 0,
            PrimTy::Short => 1,
            PrimTy::Int => 2,
            PrimTy::Long => 3,
            PrimTy::Float => 4,
            PrimTy::Double => 5,
            PrimTy::Boolean => 6,
            PrimTy::Char => 7,
            PrimTy::Void => 8,
        };

        Self {
            primitive_type_kind: val,
        }
    }
}

impl From<&BaseType> for component::PrimitiveType {
    fn from(value: &BaseType) -> Self {
        Self {
            primitive_type_kind: match value {
                BaseType::Byte => 0,
                BaseType::Char => 7,
                BaseType::Double => 5,
                BaseType::Float => 4,
                BaseType::Int => 2,
                BaseType::Long => 3,
                BaseType::Short => 1,
                BaseType::Boolean => 6,
            },
        }
    }
}

impl From<&Ty> for component::r#type::TypeKind {
    fn from(value: &Ty) -> Self {
        match value {
            Ty::Prim(p) => component::r#type::TypeKind::PrimitiveType(p.into()),
            Ty::Reference(r) => {
                component::r#type::TypeKind::ReferenceType(Box::new(component::ReferenceType {
                    reference_type_kind: Some(
                        component::reference_type::ReferenceTypeKind::ClassType(
                            component::ClassType {
                                package_specifier: if let Some(s) = &r.package_name {
                                    s.replace('/', ".")
                                } else {
                                    String::new()
                                },
                                identifier: r.name.clone(),
                                type_arguments: Vec::new(),
                                class_type_signature_suffixes: Vec::new(),
                            },
                        ),
                    ),
                }))
            }
            Ty::Array(inner, dim) => {
                let inner: component::r#type::TypeKind = inner.as_ref().into();
                if let component::r#type::TypeKind::ReferenceType(b) = &inner {
                    if let Some(component::reference_type::ReferenceTypeKind::ArrayType(a)) =
                        &b.reference_type_kind
                    {
                        return component::r#type::TypeKind::ReferenceType(Box::new(
                            component::ReferenceType {
                                reference_type_kind: Some(
                                    component::reference_type::ReferenceTypeKind::ArrayType(
                                        Box::new(component::ArrayType {
                                            inner_type: Some(a.inner_type.clone().unwrap()),
                                            dimension: *dim as i32 + a.dimension,
                                        }),
                                    ),
                                ),
                            },
                        ));
                    }
                }

                component::r#type::TypeKind::ReferenceType(Box::new(component::ReferenceType {
                    reference_type_kind: Some(
                        component::reference_type::ReferenceTypeKind::ArrayType(Box::new(
                            component::ArrayType {
                                inner_type: Some(Box::new(component::Type {
                                    type_kind: Some(inner),
                                })),
                                dimension: *dim as i32,
                            },
                        )),
                    ),
                }))
            }
            Ty::Void => component::r#type::TypeKind::PrimitiveType(component::PrimitiveType {
                primitive_type_kind: 8,
            }),
            Ty::TyVar(_) => unreachable!(),
        }
    }
}

impl From<&Ty> for component::Type {
    fn from(value: &Ty) -> Self {
        Self {
            type_kind: Some(value.into()),
        }
    }
}

impl From<&ClassTypeSignature> for component::ClassType {
    fn from(value: &ClassTypeSignature) -> Self {
        let package_specifier = if let Some(s) = &value.package_specifier {
            s.replace('/', ".")
        } else {
            String::new()
        };

        let identifier = value.simple_class_type_signature.identifier.clone();
        let type_arguments = value
            .simple_class_type_signature
            .type_arguments
            .clone()
            .unwrap_or_default()
            .iter()
            .map(|x| x.into())
            .collect::<Vec<_>>();

        Self {
            package_specifier,
            identifier,
            type_arguments,
            class_type_signature_suffixes: Vec::new(),
        }
    }
}

impl From<&ReferenceTypeSignature> for component::ReferenceType {
    fn from(value: &ReferenceTypeSignature) -> Self {
        match &value {
            ReferenceTypeSignature::TypeVariable(v) => component::ReferenceType {
                reference_type_kind: Some(
                    component::reference_type::ReferenceTypeKind::TypeVariable(
                        v.identifier.clone(),
                    ),
                ),
            },
            ReferenceTypeSignature::Class(c) => component::ReferenceType {
                reference_type_kind: Some(component::reference_type::ReferenceTypeKind::ClassType(
                    c.into(),
                )),
            },
            ReferenceTypeSignature::Array(a) => component::ReferenceType {
                reference_type_kind: Some(component::reference_type::ReferenceTypeKind::ArrayType(
                    Box::new(convert_array_type_signature_to_array_type(a, 0)),
                )),
            },
        }
    }
}

impl From<&ReferenceTypeSignature> for component::r#type::TypeKind {
    fn from(value: &ReferenceTypeSignature) -> Self {
        Self::ReferenceType(Box::new(value.into()))
    }
}

impl From<&FieldSignature> for component::r#type::TypeKind {
    fn from(value: &FieldSignature) -> Self {
        (&value.reference_type_signature).into()
    }
}

impl From<&TypeSignature> for component::r#type::TypeKind {
    fn from(value: &TypeSignature) -> Self {
        match value {
            TypeSignature::Base(b) => Self::PrimitiveType(b.into()),
            TypeSignature::Reference(r) => r.into(),
        }
    }
}

fn convert_array_type_signature_to_array_type(
    sig: &ArrayTypeSignature,
    depth: u32,
) -> component::ArrayType {
    if let TypeSignature::Reference(ReferenceTypeSignature::Array(a)) =
        sig.java_type_signature.as_ref()
    {
        convert_array_type_signature_to_array_type(a, depth + 1)
    } else {
        component::ArrayType {
            inner_type: Some(Box::new(component::Type {
                type_kind: Some(sig.java_type_signature.as_ref().into()),
            })),
            dimension: depth as i32,
        }
    }
}

impl From<&FieldSignature> for component::Type {
    fn from(value: &FieldSignature) -> Self {
        Self {
            type_kind: Some(value.into()),
        }
    }
}

impl From<&TypeArgument> for component::TypeArgument {
    fn from(value: &TypeArgument) -> Self {
        match value {
            TypeArgument::ReferenceType(wc, ref_ty) => component::TypeArgument {
                type_argument_kind: Some(
                    component::type_argument::TypeArgumentKind::ReferenceType(
                        component::TypeArgumentReferenceType {
                            wildcard_type: match wc {
                                Some(WildcardIndicator::Minus) => 2,
                                Some(WildcardIndicator::Plus) => 1,
                                None => 0,
                            },
                            reference_type: Some(ref_ty.into()),
                        },
                    ),
                ),
            },
            TypeArgument::Any => component::TypeArgument {
                type_argument_kind: Some(component::type_argument::TypeArgumentKind::Any(true)),
            },
        }
    }
}

impl From<&TypeParameter> for component::TypeParameter {
    fn from(value: &TypeParameter) -> Self {
        let mut type_bounds = Vec::new();
        if let Some(r_ts) = &value.class_bound {
            type_bounds.push(r_ts.into());
        }

        for r_ts in &value.interface_bounds {
            type_bounds.push(r_ts.into());
        }

        Self {
            identifier: value.identifier.clone(),
            type_bounds,
        }
    }
}

impl From<&ReferenceTypeSignature> for component::TypeBound {
    fn from(value: &ReferenceTypeSignature) -> Self {
        let ty_kind: component::r#type::TypeKind = value.into();
        match ty_kind {
            component::r#type::TypeKind::PrimitiveType(_) => unreachable!(),
            component::r#type::TypeKind::ReferenceType(re) => {
                let component::ReferenceType {
                    reference_type_kind: tk,
                } = re.as_ref();

                match tk.as_ref().unwrap() {
                    component::reference_type::ReferenceTypeKind::ClassType(ct) => {
                        component::TypeBound {
                            type_bound_kind: Some(component::type_bound::TypeBoundKind::ClassType(
                                ct.clone(),
                            )),
                        }
                    }
                    component::reference_type::ReferenceTypeKind::TypeVariable(ty) => {
                        component::TypeBound {
                            type_bound_kind: Some(
                                component::type_bound::TypeBoundKind::TypeVariable(ty.clone()),
                            ),
                        }
                    }
                    component::reference_type::ReferenceTypeKind::ArrayType(_) => unreachable!(),
                }
            }
        }
    }
}

impl From<&Annotation> for component::Annotation {
    fn from(value: &Annotation) -> Self {
        Self {
            annotation_type: Some((&value.ty).into()),
            annotation_kind: match &value.kind {
                AnnotationKind::RuntimeInvisible => 0,
                AnnotationKind::RuntimeVisible => 1,
                AnnotationKind::RuntimeInvisibleParameter => 2,
                AnnotationKind::RuntimeVisibleParameter => 3,
                AnnotationKind::RuntimeInvisibleType => 4,
                AnnotationKind::RuntimeVisibleType => 5,
            },
        }
    }
}

fn convert_field_ty_to_proto_ty(ty: &Ty, sig: &Option<FieldSignature>) -> component::Type {
    if let Some(sig) = sig {
        sig.into()
    } else {
        ty.into()
    }
}

fn convert_method_param_ty_to_proto_ty(_ty: Option<&Ty>, sig: &TypeSignature) -> component::Type {
    let tyk = sig.into();
    component::Type {
        type_kind: Some(tyk),
    }
}

impl From<Component> for component::Component {
    fn from(value: Component) -> Self {
        (&value).into()
    }
}
