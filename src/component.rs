use crate::{
    class_file::{
        AccessFlags, AttributeKind, ClassFile, ConstantPoolInfo, FieldAccessFlags, FieldInfo,
        MethodAccessFlags, MethodInfo,
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

#[derive(Debug)]
pub struct Component {
    minor_version: u16,
    major_version: u16,
    kind: ComponentKind,
    class_file_name: String,
}

#[derive(Debug)]
pub enum ComponentKind {
    Class(Class),
    // Interface(Interface),
    // Module(Module),
}

#[derive(Debug)]
pub struct TyName {
    package_name: Option<String>,
    name: String,
}

#[derive(Debug)]
pub enum Ty {
    Prim(PrimTy),
    Reference(TyName),
    TyVar(String),
    Array(Box<Ty>, usize),
    Void,
}

#[derive(Debug)]
pub enum PrimTy {
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

#[derive(Debug)]
pub struct Class {
    qualified_name: String,
    super_class: Option<String>,
    interfaces: Vec<String>,
    signature: ClassSignature,
    methods: Vec<Method>,
    fields: Vec<Field>,
}

#[derive(Debug)]
pub struct Method {
    name: String,
    signature: Option<MethodSignature>,
    modifiers: String,
    param_tys: Vec<Ty>,
    ret_ty: Ty,
    type_params: Vec<String>,
}

#[derive(Debug)]
pub struct Field {
    name: String,
    ty: Ty,
    signature: Option<FieldSignature>,
    modifiers: String,
}

// struct Module {
//     access_flags: u16,
//     name_index: u16,
//     version_index: u16,
//     requires: Vec<Require>,
//     exports: Vec<Export>,
//     opens: Vec<Open>,
//     uses: Vec<Use>,
//     provides: Vec<Provide>,
//     attributes: Vec<AttributeInfo>,
// }

struct ComponentExtractor<'a, 'ctxt> {
    class_file: &'a ClassFile,
    context: &'ctxt ExtractorContext,
}

bitflags::bitflags! {
    #[derive(Debug)]
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

impl<'a> ComponentExtractor<'a, '_> {
    fn extract_component(&self) -> Component {
        let class_file_name = self.get_source_file_name();
        let class_file = &self.class_file;

        if class_file.access_flags.contains(AccessFlags::INTERFACE) {
            todo!()
        } else if class_file.access_flags.contains(AccessFlags::MODULE) {
            todo!()
        }

        let ConstantPoolInfo::Class { name_index } =
            &class_file.constant_pool[class_file.this_class as usize]
        else {
            panic!("Class file has no this_class indexing into constant pool")
        };
        let ConstantPoolInfo::Utf8 {
            bytes: _,
            length: _,
            utf8_str: qualified_name,
        } = &class_file.constant_pool[*name_index as usize]
        else {
            panic!("Class file has no name")
        };

        let ConstantPoolInfo::Class { name_index } =
            &class_file.constant_pool[class_file.super_class as usize]
        else {
            panic!("Class file has no super_class indexing into constant pool")
        };
        let super_class = if *name_index != 0 {
            let ConstantPoolInfo::Utf8 {
                bytes: _,
                length: _,
                utf8_str: super_class,
            } = &class_file.constant_pool[*name_index as usize]
            else {
                panic!("Class file has no name")
            };
            Some(super_class.replace('/', "."))
        } else {
            None
        };

        let signature = self.extract_class_signature();
        let class_sig = if let Some(signature) = signature {
            let (_, class_signature) = parse_class_signature(&signature).unwrap();
            Some(class_signature)
        } else {
            None
        }
        .unwrap();

        let interfaces = class_file
            .interfaces
            .iter()
            .map(|interface| {
                let ConstantPoolInfo::Class { name_index } =
                    &class_file.constant_pool[*interface as usize]
                else {
                    panic!("Class file has no interface indexing into constant pool")
                };
                let ConstantPoolInfo::Utf8 {
                    bytes: _,
                    length: _,
                    utf8_str: interface,
                } = &class_file.constant_pool[*name_index as usize]
                else {
                    panic!("Class file has no name")
                };
                interface.replace('/', ".")
            })
            .collect();

        let methods = class_file
            .methods
            .iter()
            .filter_map(|x| self.extract_method_info(x))
            .collect::<Vec<_>>();

        let fields = class_file
            .fields
            .iter()
            .filter_map(|x| self.extract_field_info(x))
            .collect::<Vec<_>>();

        Component {
            minor_version: class_file.minor_version,
            major_version: class_file.major_version,
            kind: ComponentKind::Class(Class {
                qualified_name: qualified_name.replace('/', "."),
                super_class,
                interfaces,
                signature: class_sig,
                methods,
                fields,
            }),
            class_file_name: class_file_name.unwrap().to_string(),
        }
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

        Some(Method {
            name: name.to_string(),
            signature: sig,
            modifiers: "".to_string(),
            param_tys,
            ret_ty,
            type_params,
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

        Some(Field {
            name: name.to_string(),
            ty,
            signature: sig,
            modifiers: "".to_string(),
        })
    }

    fn is_skippable_field(&self, access_flag: &FieldAccessFlags) -> bool {
        println!("{:?}", access_flag);
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
