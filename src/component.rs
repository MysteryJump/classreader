use crate::{
    class_file::{
        AccessFlags, Attribute, AttributeKind, ClassFile, ConstantPoolInfo, FieldAccessFlags,
        FieldInfo, MethodAccessFlags, MethodInfo,
    },
    descriptor::{
        parse_field_descriptor, parse_method_descriptor, BaseTy, FieldDescriptor, FieldTy,
        ReturnDescriptor,
    },
    signature::{
        parse_class_signature, parse_field_signature, parse_method_signature, BaseType,
        ClassSignature, FieldSignature, MethodSignature, ReferenceTypeSignature, TypeSignature,
    },
};

use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct Component {
    pub minor_version: u16,
    pub major_version: u16,
    pub kind: ComponentKind,
    pub class_file_name: String,
}

#[derive(Debug, Serialize)]
pub enum ComponentKind {
    Class(Class),
    Interface(Interface),
    Module(Module),
}

#[derive(Debug, Serialize)]
pub struct TyName {
    pub package_name: Option<String>,
    pub name: String,
}

#[derive(Debug, Serialize)]
pub enum Ty {
    Prim(PrimTy),
    Reference(TyName),
    TyVar(String),
    Array(Box<Ty>, usize),
    Void,
}

#[derive(Debug, Serialize)]
pub enum PrimTy {
    Byte = 0,
    Char = 7,
    Double = 5,
    Float = 4,
    Int = 2,
    Long = 3,
    Short = 1,
    Boolean = 6,
    Void = 8,
}

#[derive(Debug, Serialize)]
pub struct Class {
    pub qualified_name: String,
    pub super_class: Option<String>,
    pub interfaces: Vec<String>,
    pub signature: Option<ClassSignature>,
    pub methods: Vec<Method>,
    pub fields: Vec<Field>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Serialize)]
pub struct Method {
    pub name: String,
    pub signature: Option<MethodSignature>,
    pub modifiers: String,
    pub param_tys: Vec<Ty>,
    pub ret_ty: Ty,
    pub type_params: Vec<String>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Serialize)]
pub struct Field {
    pub name: String,
    pub ty: Ty,
    pub signature: Option<FieldSignature>,
    pub modifiers: String,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Serialize)]
pub struct Interface {
    pub is_annotation: bool,
    pub qualified_name: String,
    pub interfaces: Vec<String>,
    pub signature: Option<ClassSignature>,
    pub methods: Vec<Method>,
    pub fields: Vec<Field>,
    pub annotations: Vec<Annotation>,
}

#[derive(Debug, Serialize)]
pub struct Module {
    pub name: String,
    pub version: String,
}

#[derive(Debug, Serialize)]
pub struct Annotation {
    pub kind: AnnotationKind,
    pub ty: Ty,
}

#[allow(clippy::enum_variant_names)]
#[derive(Debug, Serialize)]
pub enum AnnotationKind {
    RuntimeInvisible,
    RuntimeVisible,
    RuntimeInvisibleParameter,
    RuntimeVisibleParameter,
    RuntimeInvisibleType,
    RuntimeVisibleType,
}

struct ComponentExtractor<'a, 'ctxt> {
    class_file: &'a ClassFile,
    context: &'ctxt ExtractorContext,
}

bitflags::bitflags! {
    #[derive(Debug, Serialize)]
    #[serde(transparent)]
    pub struct AccessModifier: u16 {
        const PRIVATE = 0x0001;
        const PROTECTED = 0x0002;
        const PUBLIC = 0x0004;
        const DEFAULT = 0x0008;
    }
}

pub struct ExtractorContext {
    pub target_access_modifiers: AccessModifier,
}

enum Kind {
    Class,
    Interface,
    AnnotationInterface,
}

impl<'a> ComponentExtractor<'a, '_> {
    fn extract_component(&self) -> Component {
        let class_file_name = self.get_source_file_name();
        let class_file = &self.class_file;

        let comp_kind = if class_file.access_flags.contains(AccessFlags::INTERFACE) {
            self.extract_class_component(
                if class_file.access_flags.contains(AccessFlags::ANNOTATION) {
                    Kind::AnnotationInterface
                } else {
                    Kind::Interface
                },
            )
        } else if class_file.access_flags.contains(AccessFlags::MODULE) {
            self.extract_module_component()
        } else {
            self.extract_class_component(Kind::Class)
        };

        Component {
            minor_version: class_file.minor_version,
            major_version: class_file.major_version,
            kind: comp_kind,
            class_file_name: class_file_name.unwrap().to_string(),
        }
    }

    fn extract_class_component(&self, kind: Kind) -> ComponentKind {
        let ConstantPoolInfo::Class { name_index } =
            &self.class_file.constant_pool[self.class_file.this_class as usize]
        else {
            panic!("Class file has no this_class indexing into constant pool")
        };
        let ConstantPoolInfo::Utf8 {
            bytes: _,
            length: _,
            utf8_str: qualified_name,
        } = &self.class_file.constant_pool[*name_index as usize]
        else {
            panic!("Class file has no name")
        };

        let super_class = if self.class_file.super_class == 0 {
            if qualified_name == "java/lang/Object" {
                None
            } else {
                panic!("Class file has no super_class")
            }
        } else {
            let ConstantPoolInfo::Class { name_index } =
                &self.class_file.constant_pool[self.class_file.super_class as usize]
            else {
                panic!("Class file has no super_class indexing into constant pool")
            };
            if *name_index != 0 {
                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: super_class,
                } = &self.class_file.constant_pool[*name_index as usize]
                else {
                    panic!("Class file has no name")
                };
                let super_class = super_class.replace('/', ".");
                if super_class == "java.lang.Object" {
                    None
                } else {
                    Some(super_class)
                }
            } else {
                None
            }
        };

        let signature = self.extract_class_signature();
        let class_sig = if let Some(signature) = signature {
            let (_, class_signature) = parse_class_signature(&signature).unwrap();
            Some(class_signature)
        } else {
            None
        };

        let interfaces = self
            .class_file
            .interfaces
            .iter()
            .map(|interface| {
                let ConstantPoolInfo::Class { name_index } =
                    &self.class_file.constant_pool[*interface as usize]
                else {
                    panic!("Class file has no interface indexing into constant pool")
                };
                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: interface,
                } = &self.class_file.constant_pool[*name_index as usize]
                else {
                    panic!("Class file has no name")
                };
                interface.replace('/', ".")
            })
            .collect();

        let methods = self
            .class_file
            .methods
            .iter()
            .filter_map(|x| self.extract_method_info(x))
            .collect::<Vec<_>>();

        let fields = self
            .class_file
            .fields
            .iter()
            .filter_map(|x| self.extract_field_info(x))
            .collect::<Vec<_>>();

        let annotations = self.extract_annotations(&self.class_file.attributes);

        match kind {
            Kind::Class => ComponentKind::Class(Class {
                qualified_name: qualified_name.replace('/', "."),
                super_class,
                interfaces,
                signature: class_sig,
                methods,
                fields,
                annotations,
            }),
            Kind::Interface | Kind::AnnotationInterface if super_class.is_some() => {
                panic!("Interface has super class: {super_class:?}")
            }
            Kind::Interface => ComponentKind::Interface(Interface {
                is_annotation: false,
                qualified_name: qualified_name.replace('/', "."),
                interfaces,
                signature: class_sig,
                methods,
                fields,
                annotations,
            }),
            Kind::AnnotationInterface => ComponentKind::Interface(Interface {
                is_annotation: true,
                qualified_name: qualified_name.replace('/', "."),
                interfaces,
                signature: class_sig,
                methods,
                fields,
                annotations,
            }),
        }
    }

    fn extract_module_component(&self) -> ComponentKind {
        for attr in &self.class_file.attributes {
            if let AttributeKind::Module {
                module_name_index,
                module_version_index,
                ..
            } = attr.kind
            {
                let ConstantPoolInfo::Module { name_index, .. } =
                    &self.class_file.constant_pool[module_name_index as usize]
                else {
                    panic!("Class file has no this_class indexing into constant pool")
                };

                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: module_name,
                } = &self.class_file.constant_pool[*name_index as usize]
                else {
                    panic!("Class file has no module name")
                };

                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: module_version,
                } = &self.class_file.constant_pool[module_version_index as usize]
                else {
                    panic!("Class file has no module version")
                };

                return ComponentKind::Module(Module {
                    name: module_name.to_string(),
                    version: module_version.to_string(),
                });
            }
        }

        panic!("Class file has no module attribute")
    }

    fn extract_class_signature(&self) -> Option<String> {
        for attr in &self.class_file.attributes {
            if let AttributeKind::Signature { signature_index } = attr.kind {
                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: signature,
                } = &self.class_file.constant_pool[signature_index as usize]
                else {
                    panic!("Class file has no signature")
                };
                return Some(signature.to_string());
            }
        }
        None
    }

    fn extract_method_signature(&self, method_info: &MethodInfo) -> Option<MethodSignature> {
        for attr in &method_info.attributes {
            if let AttributeKind::Signature { signature_index } = attr.kind {
                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: signature,
                } = &self.class_file.constant_pool[signature_index as usize]
                else {
                    panic!(
                        "Method `{}` in Class file has no signature",
                        method_info.get_name(&self.class_file.constant_pool)
                    )
                };

                let (_, signature) = parse_method_signature(signature).unwrap();
                return Some(signature);
            }
        }
        None
    }

    fn extract_field_signature(&self, field_info: &FieldInfo) -> Option<FieldSignature> {
        for attr in &field_info.attributes {
            if let AttributeKind::Signature { signature_index } = attr.kind {
                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: signature,
                } = &self.class_file.constant_pool[signature_index as usize]
                else {
                    panic!(
                        "Field `{}` in Class file has no signature",
                        field_info.get_name(&self.class_file.constant_pool)
                    )
                };

                let (_, signature) = parse_field_signature(signature).unwrap();
                return Some(signature);
            }
        }
        None
    }

    fn extract_method_info(&self, method_info: &MethodInfo) -> Option<Method> {
        if self.is_skippable_method(&method_info.access_flags) {
            return None;
        }

        let name = method_info.get_name(&self.class_file.constant_pool);
        let sig = self.extract_method_signature(method_info);
        let descriptor = method_info.get_descriptor(&self.class_file.constant_pool);
        let descriptor = parse_method_descriptor(descriptor);

        let ret_ty = match descriptor.ret_desc {
            ReturnDescriptor::TyDesc(desc) => (&desc.ty).into(),
            ReturnDescriptor::Void => Ty::Void,
        };

        let type_params = if let Some(sig) = &sig {
            if let Some(type_params) = &sig.type_parameters {
                type_params
                    .iter()
                    .map(|type_param| type_param.identifier.to_string())
                    .collect()
            } else {
                Vec::new()
            }
        } else {
            Vec::new()
        };

        let param_tys = if let Some(sig) = &sig {
            sig.parameters
                .iter()
                .map(|param| match param {
                    TypeSignature::Reference(re) => re.into(),
                    TypeSignature::Base(ba) => ba.into(),
                })
                .collect()
        } else {
            let method_desc = parse_method_descriptor(descriptor.descriptor.as_str());
            method_desc
                .param_descs
                .iter()
                .map(|param| (&param.ty).into())
                .collect()
        };

        let annotations = self.extract_annotations(&method_info.attributes);

        Some(Method {
            name: name.to_string(),
            signature: sig,
            modifiers: "".to_string(),
            param_tys,
            ret_ty,
            type_params,
            annotations,
        })
    }

    fn extract_field_info(&self, field_info: &FieldInfo) -> Option<Field> {
        if self.is_skippable_field(&field_info.access_flags) {
            return None;
        }

        let name = field_info.get_name(&self.class_file.constant_pool);
        let descriptor = field_info.get_descriptor(&self.class_file.constant_pool);
        let descriptor = parse_field_descriptor(descriptor);

        let sig = self.extract_field_signature(field_info);

        let ty = if let Some(sig) = &sig {
            sig.into()
        } else {
            (&descriptor).into()
        };

        let annotations = self.extract_annotations(&field_info.attributes);

        Some(Field {
            name: name.to_string(),
            ty,
            signature: sig,
            modifiers: "".to_string(),
            annotations,
        })
    }

    fn extract_annotations(&self, attributes: &[Attribute]) -> Vec<Annotation> {
        attributes
            .iter()
            .filter_map(|attr| {
                match &attr.kind {
                    t @ (AttributeKind::RuntimeVisibleAnnotations {
                        num_annotations: _,
                        annotations,
                    }
                    | AttributeKind::RuntimeInvisibleAnnotations {
                        num_annotations: _,
                        annotations,
                    }) => {
                        Some(annotations.iter().map(move |annotation| {
                            let ConstantPoolInfo::Utf8 { utf8_str, .. } =
                                &self.class_file.constant_pool[annotation.type_index as usize]
                            else {
                                panic!("type_idex of annotation in {attr:?} indicates invalid constant pool index");
                            };

                            Annotation {
                                kind: match t {
                                    AttributeKind::RuntimeVisibleAnnotations { .. } => {
                                        AnnotationKind::RuntimeVisible
                                    }
                                    AttributeKind::RuntimeInvisibleAnnotations { .. } => {
                                        AnnotationKind::RuntimeInvisible
                                    }
                                    _ => unreachable!(),
                                },
                                ty: (&parse_field_descriptor(utf8_str)).into(),
                            }
                        }).collect::<Vec<_>>())
                    }
                    t @ (AttributeKind::RuntimeVisibleParameterAnnotations {
                        num_parameters: _,
                        parameter_annotations,
                    }
                    | AttributeKind::RuntimeInvisibleParameterAnnotations {
                        num_parameters: _,
                        parameter_annotations,
                    }) => {
                        Some(parameter_annotations.iter().flat_map(move |annotation| {
                            annotation
                                .annotations
                                .iter()
                                .map(move |annotation| {
                                    let ConstantPoolInfo::Utf8 { utf8_str, .. } =
                                        &self.class_file.constant_pool
                                            [annotation.type_index as usize]
                                    else {
                                        panic!("type_idex of annotation in {attr:?} indicates invalid constant pool index");
                                    };

                                    Annotation {
                                        kind: match t {
                                            AttributeKind::RuntimeVisibleParameterAnnotations {
                                                ..
                                            } => AnnotationKind::RuntimeVisibleParameter,
                                            AttributeKind::RuntimeInvisibleParameterAnnotations {
                                                ..
                                            } => AnnotationKind::RuntimeInvisibleParameter,
                                            _ => unreachable!(),
                                        },
                                        ty: (&parse_field_descriptor(utf8_str)).into(),
                                    }
                                })
                                .collect::<Vec<_>>()
                        }).collect())
                    },
                    t @ (AttributeKind::RuntimeVisibleTypeAnnotations {
                        num_annotations: _,
                        type_annotations,
                    }
                    | AttributeKind::RuntimeInvisibleTypeAnnotations {
                        num_annotations: _,
                        type_annotations,
                    }) => {
                        Some(type_annotations.iter().map(move |annotation| {
                            let ConstantPoolInfo::Utf8 { utf8_str, .. } =
                                &self.class_file.constant_pool[annotation.type_index as usize]
                            else {
                                panic!("type_idex of annotation in {attr:?} indicates invalid constant pool index");
                            };

                            Annotation {
                                kind: match t {
                                    AttributeKind::RuntimeVisibleTypeAnnotations { .. } => {
                                        AnnotationKind::RuntimeVisibleType
                                    }
                                    AttributeKind::RuntimeInvisibleTypeAnnotations { .. } => {
                                        AnnotationKind::RuntimeInvisibleType
                                    }
                                    _ => unreachable!(),
                                },
                                ty: (&parse_field_descriptor(utf8_str)).into(),
                            }
                        }).collect::<Vec<_>>())
                    }
                    _ => None,
                }
            })
            .flatten()
            .collect()
    }

    fn is_skippable_field(&self, access_flag: &FieldAccessFlags) -> bool {
        if self.context.target_access_modifiers.is_empty() {
            false
        } else {
            !self
                .context
                .target_access_modifiers
                .contains(access_flag.into())
        }
    }

    fn is_skippable_method(&self, access_flag: &MethodAccessFlags) -> bool {
        if self.context.target_access_modifiers.is_empty() {
            false
        } else {
            !self
                .context
                .target_access_modifiers
                .contains(access_flag.into())
        }
    }

    fn get_source_file_name(&self) -> Option<&str> {
        let mut class_file_name = None;
        for attr in &self.class_file.attributes {
            if let AttributeKind::SourceFile { sourcefile_index } = attr.kind {
                let name_info = &self.class_file.constant_pool[sourcefile_index as usize];
                if let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str,
                } = name_info
                {
                    class_file_name = Some(utf8_str as &str);
                }
            }
        }

        class_file_name
    }
}

pub fn extract_component(class_file: &ClassFile, context: &ExtractorContext) -> Component {
    let extractor = ComponentExtractor {
        class_file,
        context,
    };
    extractor.extract_component()
}

impl From<&FieldTy> for Ty {
    fn from(ty: &FieldTy) -> Self {
        match ty {
            FieldTy::Base(BaseTy::Boolean) => Ty::Prim(PrimTy::Boolean),
            FieldTy::Base(BaseTy::Byte) => Ty::Prim(PrimTy::Byte),
            FieldTy::Base(BaseTy::Char) => Ty::Prim(PrimTy::Char),
            FieldTy::Base(BaseTy::Double) => Ty::Prim(PrimTy::Double),
            FieldTy::Base(BaseTy::Float) => Ty::Prim(PrimTy::Float),
            FieldTy::Base(BaseTy::Int) => Ty::Prim(PrimTy::Int),
            FieldTy::Base(BaseTy::Long) => Ty::Prim(PrimTy::Long),
            FieldTy::Base(BaseTy::Short) => Ty::Prim(PrimTy::Short),
            FieldTy::Base(BaseTy::Void) => Ty::Prim(PrimTy::Void),
            FieldTy::Obj(obj) => Ty::Reference(TyName {
                package_name: None,
                name: obj.class_name.clone(),
            }),
            FieldTy::Array(a) => {
                let mut ty = a.ty.as_ref().into();
                for _ in 0..a.dims {
                    ty = Ty::Array(Box::new(ty), 1);
                }
                ty
            }
        }
    }
}

impl From<&BaseType> for Ty {
    fn from(value: &BaseType) -> Self {
        match value {
            BaseType::Byte => Ty::Prim(PrimTy::Byte),
            BaseType::Char => Ty::Prim(PrimTy::Char),
            BaseType::Double => Ty::Prim(PrimTy::Double),
            BaseType::Float => Ty::Prim(PrimTy::Float),
            BaseType::Int => Ty::Prim(PrimTy::Int),
            BaseType::Long => Ty::Prim(PrimTy::Long),
            BaseType::Short => Ty::Prim(PrimTy::Short),
            BaseType::Boolean => Ty::Prim(PrimTy::Boolean),
        }
    }
}

impl From<&ReferenceTypeSignature> for Ty {
    fn from(value: &ReferenceTypeSignature) -> Self {
        match value {
            ReferenceTypeSignature::Class(class_type) => Ty::Reference(TyName {
                package_name: class_type.package_specifier.clone(),
                name: class_type.simple_class_type_signature.identifier.clone(),
            }),
            ReferenceTypeSignature::TypeVariable(type_variable) => {
                Ty::TyVar(type_variable.identifier.to_string())
            }
            ReferenceTypeSignature::Array(array_type) => {
                let mut ty = array_type.java_type_signature.as_ref().into();
                let mut ty_depth = 1;

                while let Ty::Array(n_ty, _) = ty {
                    ty_depth += 1;
                    ty = *n_ty;
                }

                Ty::Array(Box::new(ty), ty_depth)
            }
        }
    }
}

impl From<&TypeSignature> for Ty {
    fn from(value: &TypeSignature) -> Self {
        match value {
            TypeSignature::Base(base_type) => base_type.into(),
            TypeSignature::Reference(reference_type) => reference_type.into(),
        }
    }
}

impl From<&FieldSignature> for Ty {
    fn from(value: &FieldSignature) -> Self {
        (&value.reference_type_signature).into()
    }
}

impl From<&FieldDescriptor> for Ty {
    fn from(value: &FieldDescriptor) -> Self {
        (&value.ty).into()
    }
}

impl From<&FieldAccessFlags> for AccessModifier {
    fn from(value: &FieldAccessFlags) -> Self {
        if value.contains(FieldAccessFlags::PUBLIC) {
            AccessModifier::PUBLIC
        } else if value.contains(FieldAccessFlags::PROTECTED) {
            AccessModifier::PROTECTED
        } else if value.contains(FieldAccessFlags::PRIVATE) {
            AccessModifier::PRIVATE
        } else {
            AccessModifier::DEFAULT
        }
    }
}

impl From<&MethodAccessFlags> for AccessModifier {
    fn from(value: &MethodAccessFlags) -> Self {
        if value.contains(MethodAccessFlags::PUBLIC) {
            AccessModifier::PUBLIC
        } else if value.contains(MethodAccessFlags::PROTECTED) {
            AccessModifier::PROTECTED
        } else if value.contains(MethodAccessFlags::PRIVATE) {
            AccessModifier::PRIVATE
        } else {
            AccessModifier::DEFAULT
        }
    }
}
