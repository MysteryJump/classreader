use bitflags::bitflags;
use nom::{
    bytes::complete::{tag, take},
    number::complete::{be_u16, be_u32, be_u8},
    Err, IResult,
};

#[derive(Debug)]
pub struct ClassFile {
    pub minor_version: u16,
    pub major_version: u16,
    pub constant_pool_count: u16,
    pub constant_pool: Vec<ConstantPoolInfo>,
    pub access_flags: AccessFlags,
    pub this_class: u16,
    pub super_class: u16,
    pub interfaces_count: u16,
    pub interfaces: Vec<u16>,
    pub fields_count: u16,
    pub fields: Vec<FieldInfo>,
    pub methods_count: u16,
    pub methods: Vec<MethodInfo>,
    pub attributes_count: u16,
    attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub enum ConstantPoolInfo {
    Class {
        name_index: u16,
    },
    Fieldref {
        class_index: u16,
        name_and_type_index: u16,
    },
    Methodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    InterfaceMethodref {
        class_index: u16,
        name_and_type_index: u16,
    },
    String {
        string_index: u16,
    },
    Integer {
        bytes: u32,
    },
    Float {
        bytes: u32,
    },
    Long {
        high_bytes: u32,
        low_bytes: u32,
    },
    Double {
        high_bytes: u32,
        low_bytes: u32,
    },
    NameAndType {
        name_index: u16,
        descriptor_index: u16,
    },
    Utf8 {
        length: u16,
        bytes: Vec<u8>,
        utf8_str: String,
    },
    MethodHandle {
        reference_kind: u8,
        reference_index: u16,
    },
    MethodType {
        descriptor_index: u16,
    },
    Dynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    InvokeDynamic {
        bootstrap_method_attr_index: u16,
        name_and_type_index: u16,
    },
    Module {
        name_index: u16,
    },
    Package {
        name_index: u16,
    },
}

bitflags! {
    #[derive(Debug)]
    pub struct AccessFlags: u16 {
        const PUBLIC = 0x0001;
        const FINAL = 0x0010;
        const SUPER = 0x0020;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
        const MODULE = 0x8000;
    }
}

bitflags! {
    #[derive(Debug)]
    pub struct FieldAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const VOLATILE = 0x0040;
        const TRANSIENT = 0x0080;
        const SYNTHETIC = 0x1000;
        const ENUM = 0x4000;
    }
}

#[derive(Debug)]
pub struct FieldInfo {
    pub access_flags: FieldAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    attributes: Vec<Attribute>,
}

bitflags! {
    #[derive(Debug)]
    pub struct MethodAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const SYNCHRONIZED = 0x0020;
        const BRIDGE = 0x0040;
        const VARARGS = 0x0080;
        const NATIVE = 0x0100;
        const ABSTRACT = 0x0400;
        const STRICT = 0x0800;
        const SYNTHETIC = 0x1000;
    }
}

#[derive(Debug)]
pub struct MethodInfo {
    pub access_flags: MethodAccessFlags,
    pub name_index: u16,
    pub descriptor_index: u16,
    pub attributes_count: u16,
    attributes: Vec<Attribute>,
}

#[derive(Debug)]
pub struct AttributeInfo {
    pub attribute_name_index: u16,
    pub attribute_length: u32,
    pub info: Vec<u8>,
}

#[derive(Debug)]
struct ExceptionTableEntry {
    start_pc: u16,
    end_pc: u16,
    handler_pc: u16,
    catch_type: u16,
}

#[derive(Debug)]
enum VerificationTypeInfo {
    Top,
    Integer,
    Float,
    Long,
    Double,
    Null,
    UninitializedThis,
    Object { cpool_index: u16 },
    Uninitialized { offset: u16 },
}

#[derive(Debug)]
enum StackMapFrameKind {
    Same, // frame_type:0-63
    SameLocals1StackItem {
        stack: VerificationTypeInfo,
    }, // frame_type:64-127
    SameLocals1StackItemExtended {
        offset_delta: u16,
        stack: VerificationTypeInfo,
    }, // frame_type:247
    Chop {
        offset_delta: u16,
    }, // frame_type:248-250
    SameExtended {
        offset_delta: u16,
    }, // frame_type:251
    Append {
        offset_delta: u16,
        locals: Vec<VerificationTypeInfo>,
    }, // frame_type:252-254
    Full {
        offset_delta: u16,
        number_of_locals: u16,
        locals: Vec<VerificationTypeInfo>,
        number_of_stack_items: u16,
        stack: Vec<VerificationTypeInfo>,
    }, // frame_type:255
}

#[derive(Debug)]
struct StackMapFrame {
    frame_type: u8,
    kind: StackMapFrameKind,
}

#[derive(Debug)]
struct BootstrapMethod {
    bootstrap_method_ref: u16,
    num_bootstrap_arguments: u16,
    bootstrap_arguments: Vec<u16>,
}

#[derive(Debug)]
struct Parameter {
    name_index: u16,
    access_flags: u16,
}

#[derive(Debug)]
struct InnerClass {
    inner_class_info_index: u16,
    outer_class_info_index: u16,
    inner_name_index: u16,
    inner_class_access_flags: InnerClassAccessFlags,
}

bitflags! {
    #[derive(Debug)]
    struct InnerClassAccessFlags: u16 {
        const PUBLIC = 0x0001;
        const PRIVATE = 0x0002;
        const PROTECTED = 0x0004;
        const STATIC = 0x0008;
        const FINAL = 0x0010;
        const INTERFACE = 0x0200;
        const ABSTRACT = 0x0400;
        const SYNTHETIC = 0x1000;
        const ANNOTATION = 0x2000;
        const ENUM = 0x4000;
    }
}

#[derive(Debug)]
struct RecordComponent {
    name_index: u16,
    descriptor_index: u16,
    attributes_count: u16,
    attributes: Vec<Attribute>,
}

#[derive(Debug)]
struct LineNumberTableEntry {
    start_pc: u16,
    line_number: u16,
}

#[derive(Debug)]
struct LocalVariableTableEntry {
    start_pc: u16,
    length: u16,
    name_index: u16,
    descriptor_index: u16,
    index: u16,
}

#[derive(Debug)]
struct LocalVariableTypeTableEntry {
    start_pc: u16,
    length: u16,
    name_index: u16,
    signature_index: u16,
    index: u16,
}

#[derive(Debug)]
struct Annotation {
    type_index: u16,
    num_element_value_pairs: u16,
    element_value_pairs: Vec<ElementValuePair>,
}

#[derive(Debug)]
struct ElementValuePair {
    element_name_index: u16,
    value: ElementValue,
}

#[derive(Debug)]
enum ElementValue {
    Byte {
        const_value_index: u16,
    },
    Char {
        const_value_index: u16,
    },
    Double {
        const_value_index: u16,
    },
    Float {
        const_value_index: u16,
    },
    Int {
        const_value_index: u16,
    },
    Long {
        const_value_index: u16,
    },
    Short {
        const_value_index: u16,
    },
    Boolean {
        const_value_index: u16,
    },
    String {
        const_value_index: u16,
    },
    EnumConst {
        type_name_index: u16,
        const_name_index: u16,
    },
    ClassInfoIndex {
        class_info_index: u16,
    },
    AnnotationValue {
        annotation: Annotation,
    },
    ArrayValue {
        num_values: u16,
        values: Vec<ElementValue>,
    },
}

#[derive(Debug)]
struct ParameterAnnotation {
    num_annotations: u16,
    annotations: Vec<Annotation>,
}

#[derive(Debug)]
struct TypeAnnotation {
    target_type: u8,
    target_info: TargetInfo,
    target_path: TypePath,
    type_index: u16,
    num_element_value_pairs: u16,
    element_value_pairs: Vec<ElementValuePair>,
}

#[derive(Debug)]
struct TargetInfo {
    target_type: u8,
    target_info_kind: TargetInfoKind,
}

#[derive(Debug)]
enum TargetInfoKind {
    TypeParameter {
        type_parameter_index: u8,
    },
    SuperType {
        super_type_index: u16,
    },
    TypeParameterBound {
        type_parameter_index: u8,
        bound_index: u8,
    },
    Empty,
    FormalParameter {
        formal_parameter_index: u8,
    },
    Throws {
        throws_type_index: u16,
    },
    LocalVar {
        table_length: u16,
        table: Vec<LocalVarTarget>,
    },
    Catch {
        exception_table_index: u16,
    },
    Offset {
        offset: u16,
    },
    TypeArgument {
        offset: u16,
        type_argument_index: u8,
    },
}

#[derive(Debug)]
struct LocalVarTarget {
    start_pc: u16,
    length: u16,
    index: u16,
}

#[derive(Debug)]
struct TypePath {
    path_length: u8,
    path: Vec<Path>,
}

#[derive(Debug)]
struct Path {
    type_path_kind: u8,
    type_argument_index: u8,
}

#[derive(Debug)]
struct ModuleRequires {
    requires_index: u16,
    requires_flags: u16,
    requires_version_index: u16,
}

#[derive(Debug)]
struct ModuleExports {
    exports_index: u16,
    exports_flags: u16,
    exports_to_count: u16,
    exports_to_index: Vec<u16>,
}

#[derive(Debug)]
struct ModuleOpens {
    opens_index: u16,
    opens_flags: u16,
    opens_to_count: u16,
    opens_to_index: Vec<u16>,
}

#[derive(Debug)]
struct ModuleProvides {
    provides_index: u16,
    provides_with_count: u16,
    provides_with_index: Vec<u16>,
}

#[derive(Debug)]
struct Attribute {
    attribute_name_index: u16,
    attribute_length: u32,
    kind: AttributeKind,
}

#[derive(Debug)]
enum AttributeKind {
    // Critical to correct interpretation
    ConstantValue {
        constant_value_index: u16,
    },
    Code {
        max_stack: u16,
        max_locals: u16,
        code_length: u32,
        code: Vec<u8>,
        exception_table_length: u16,
        exception_table: Vec<ExceptionTableEntry>,
        attributes_count: u16,
        attributes: Vec<Attribute>,
    },
    StackMapTable {
        number_of_entries: u16,
        entries: Vec<StackMapFrame>,
    },
    BootstrapMethods {
        num_bootstrap_methods: u16,
        bootstrap_methods: Vec<BootstrapMethod>,
    },
    NestHost {
        host_class_index: u16,
    },
    NestMembers {
        number_of_classes: u16,
        classes: Vec<u16>,
    },
    PermittedSubclasses {
        number_of_classes: u16,
        classes: Vec<u16>,
    },
    // Not critical to correct interpretation
    Exceptions {
        number_of_exceptions: u16,
        exception_index_table: Vec<u16>,
    },
    InnerClasses {
        number_of_classes: u16,
        classes: Vec<InnerClass>,
    },
    EnclosingMethod {
        class_index: u16,
        method_index: u16,
    },
    Synthetic,
    Signature {
        signature_index: u16,
    },
    Record {
        components_count: u16,
        components: Vec<RecordComponent>,
    },
    SourceFile {
        sourcefile_index: u16,
    },
    LineNumberTable {
        line_number_table_length: u16,
        line_number_table: Vec<LineNumberTableEntry>,
    },
    LocalVariableTable {
        local_variable_table_length: u16,
        local_variable_table: Vec<LocalVariableTableEntry>,
    },
    LocalVariableTypeTable {
        local_variable_type_table_length: u16,
        local_variable_type_table: Vec<LocalVariableTypeTableEntry>,
    },
    // Metadata
    SourceDebugExtension {
        debug_extension: Vec<u8>,
    },
    Deprecated,
    RuntimeVisibleAnnotations {
        num_annotations: u16,
        annotations: Vec<Annotation>,
    },
    RuntimeInvisibleAnnotations {
        num_annotations: u16,
        annotations: Vec<Annotation>,
    },
    RuntimeVisibleParameterAnnotations {
        num_parameters: u8,
        parameter_annotations: Vec<ParameterAnnotation>,
    },
    RuntimeInvisibleParameterAnnotations {
        num_parameters: u8,
        parameter_annotations: Vec<ParameterAnnotation>,
    },
    RuntimeVisibleTypeAnnotations {
        num_annotations: u16,
        type_annotations: Vec<TypeAnnotation>,
    },
    RuntimeInvisibleTypeAnnotations {
        num_annotations: u16,
        type_annotations: Vec<TypeAnnotation>,
    },
    AnnotationDefault {
        default_value: ElementValue,
    },
    MethodParameters {
        parameters_count: u8,
        parameters: Vec<Parameter>,
    },
    Module {
        module_name_index: u16,
        module_flags: u16,
        module_version_index: u16,
        requires_count: u16,
        requires: Vec<ModuleRequires>,
        exports_count: u16,
        exports: Vec<ModuleExports>,
        opens_count: u16,
        opens: Vec<ModuleOpens>,
        uses_count: u16,
        uses_index: Vec<u16>,
        provides_count: u16,
        provides: Vec<ModuleProvides>,
    },
    ModulePackages {
        package_count: u16,
        package_index: Vec<u16>,
    },
    ModuleMainClass {
        main_class_index: u16,
    },

    Unknown,
    Skipped {
        len: u32,
    },
}

fn parse_magic_number(input: &[u8]) -> IResult<&[u8], ()> {
    let (input, _) = tag([0xCA, 0xFE, 0xBA, 0xBE])(input)?;
    Ok((input, ()))
}

fn parse_version(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, version) = be_u16(input)?;
    Ok((input, version))
}

fn parse_constant_pool_count(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, count) = be_u16(input)?;
    Ok((input, count))
}

fn parse_constant_pool(input: &[u8], count: u16) -> IResult<&[u8], Vec<ConstantPoolInfo>> {
    let mut input = input;
    let mut constant_pool = Vec::with_capacity(count as usize);
    for idx in 1..count - 1 {
        let (i, constant_pool_info) = parse_constant_pool_info(input)?;
        input = i;
        if let ConstantPoolInfo::Utf8 {
            length,
            bytes: _,
            utf8_str: s,
        } = constant_pool_info.clone()
        {
            println!("idx:{}, length:{}, string:{:?}", idx, length, s);
        } else {
            println!("idx:{idx}, {:?}", constant_pool_info);
        }
        constant_pool.push(constant_pool_info);
    }
    Ok((input, constant_pool))
}

fn parse_constant_pool_info(input: &[u8]) -> IResult<&[u8], ConstantPoolInfo> {
    let (input, tag) = be_u8(input)?;
    match tag {
        7 => {
            let (input, name_index) = be_u16(input)?;
            Ok((input, ConstantPoolInfo::Class { name_index }))
        }
        9 => {
            let (input, class_index) = be_u16(input)?;
            let (input, name_and_type_index) = be_u16(input)?;
            Ok((
                input,
                ConstantPoolInfo::Fieldref {
                    class_index,
                    name_and_type_index,
                },
            ))
        }
        10 => {
            let (input, class_index) = be_u16(input)?;
            let (input, name_and_type_index) = be_u16(input)?;
            Ok((
                input,
                ConstantPoolInfo::Methodref {
                    class_index,
                    name_and_type_index,
                },
            ))
        }
        11 => {
            let (input, class_index) = be_u16(input)?;
            let (input, name_and_type_index) = be_u16(input)?;
            Ok((
                input,
                ConstantPoolInfo::InterfaceMethodref {
                    class_index,
                    name_and_type_index,
                },
            ))
        }
        8 => {
            let (input, string_index) = be_u16(input)?;
            Ok((input, ConstantPoolInfo::String { string_index }))
        }
        3 => {
            let (input, bytes) = be_u32(input)?;
            Ok((input, ConstantPoolInfo::Integer { bytes }))
        }
        4 => {
            let (input, bytes) = be_u32(input)?;
            Ok((input, ConstantPoolInfo::Float { bytes }))
        }
        5 => {
            let (input, high_bytes) = be_u32(input)?;
            let (input, low_bytes) = be_u32(input)?;
            Ok((
                input,
                ConstantPoolInfo::Long {
                    high_bytes,
                    low_bytes,
                },
            ))
        }
        6 => {
            let (input, high_bytes) = be_u32(input)?;
            let (input, low_bytes) = be_u32(input)?;
            Ok((
                input,
                ConstantPoolInfo::Double {
                    high_bytes,
                    low_bytes,
                },
            ))
        }
        12 => {
            let (input, name_index) = be_u16(input)?;
            let (input, descriptor_index) = be_u16(input)?;
            Ok((
                input,
                ConstantPoolInfo::NameAndType {
                    name_index,
                    descriptor_index,
                },
            ))
        }
        1 => {
            let (input, length) = be_u16(input)?;
            let (input, bytes) = take(length)(input)?;
            Ok((
                input,
                ConstantPoolInfo::Utf8 {
                    length,
                    bytes: bytes.to_vec(),
                    utf8_str: String::from_utf8(bytes.to_vec()).unwrap(),
                },
            ))
        }
        15 => {
            let (input, reference_kind) = be_u8(input)?;
            let (input, reference_index) = be_u16(input)?;
            Ok((
                input,
                ConstantPoolInfo::MethodHandle {
                    reference_kind,
                    reference_index,
                },
            ))
        }
        16 => {
            let (input, descriptor_index) = be_u16(input)?;
            Ok((input, ConstantPoolInfo::MethodType { descriptor_index }))
        }
        17 => {
            let (input, bootstrap_method_attr_index) = be_u16(input)?;
            let (input, name_and_type_index) = be_u16(input)?;
            Ok((
                input,
                ConstantPoolInfo::Dynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                },
            ))
        }
        18 => {
            let (input, bootstrap_method_attr_index) = be_u16(input)?;
            let (input, name_and_type_index) = be_u16(input)?;
            Ok((
                input,
                ConstantPoolInfo::InvokeDynamic {
                    bootstrap_method_attr_index,
                    name_and_type_index,
                },
            ))
        }
        19 => {
            let (input, name_index) = be_u16(input)?;
            Ok((input, ConstantPoolInfo::Module { name_index }))
        }
        20 => {
            let (input, name_index) = be_u16(input)?;
            Ok((input, ConstantPoolInfo::Package { name_index }))
        }
        _ => panic!("Unknown tag: {}", tag),
    }
}

fn parse_access_flags(input: &[u8]) -> IResult<&[u8], AccessFlags> {
    let (input, access_flags) = be_u16(input)?;
    Ok((input, AccessFlags::from_bits(access_flags).unwrap()))
}

fn parse_this_class(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, this_class) = be_u16(input)?;
    Ok((input, this_class))
}

fn parse_super_class(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, super_class) = be_u16(input)?;
    Ok((input, super_class))
}

fn parse_interfaces_count(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, interfaces_count) = be_u16(input)?;
    Ok((input, interfaces_count))
}

fn parse_interfaces(input: &[u8], count: u16) -> IResult<&[u8], Vec<u16>> {
    let mut input = input;
    let mut interfaces = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, interface) = be_u16(input)?;
        input = i;
        interfaces.push(interface);
    }
    Ok((input, interfaces))
}

fn parse_fields_count(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, fields_count) = be_u16(input)?;
    Ok((input, fields_count))
}

fn parse_methods_count(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, methods_count) = be_u16(input)?;
    Ok((input, methods_count))
}

fn parse_attributes_count(input: &[u8]) -> IResult<&[u8], u16> {
    let (input, attributes_count) = be_u16(input)?;
    Ok((input, attributes_count))
}

fn parse_exception_table_entry(input: &[u8]) -> IResult<&[u8], ExceptionTableEntry> {
    let (input, start_pc) = be_u16(input)?;
    let (input, end_pc) = be_u16(input)?;
    let (input, handler_pc) = be_u16(input)?;
    let (input, catch_type) = be_u16(input)?;
    Ok((
        input,
        ExceptionTableEntry {
            start_pc,
            end_pc,
            handler_pc,
            catch_type,
        },
    ))
}

fn parse_exception_table(input: &[u8], count: u16) -> IResult<&[u8], Vec<ExceptionTableEntry>> {
    let mut input = input;
    let mut exception_table = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, exception_table_entry) = parse_exception_table_entry(input)?;
        input = i;
        exception_table.push(exception_table_entry);
    }
    Ok((input, exception_table))
}

fn parse_verification_type_info(input: &[u8]) -> IResult<&[u8], VerificationTypeInfo> {
    let (input, tag) = be_u8(input)?;
    match tag {
        0 => Ok((input, VerificationTypeInfo::Top)),
        1 => Ok((input, VerificationTypeInfo::Integer)),
        2 => Ok((input, VerificationTypeInfo::Float)),
        5 => Ok((input, VerificationTypeInfo::Null)),
        6 => Ok((input, VerificationTypeInfo::UninitializedThis)),
        7 => {
            let (input, cpool_index) = be_u16(input)?;
            Ok((input, VerificationTypeInfo::Object { cpool_index }))
        }
        8 => {
            let (input, offset) = be_u16(input)?;
            Ok((input, VerificationTypeInfo::Uninitialized { offset }))
        }
        4 => Ok((input, VerificationTypeInfo::Long)),
        3 => Ok((input, VerificationTypeInfo::Double)),
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn parse_verification_type_info_list(
    input: &[u8],
    count: u16,
) -> IResult<&[u8], Vec<VerificationTypeInfo>> {
    let mut input = input;
    let mut verification_type_info_list = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, verification_type_info) = parse_verification_type_info(input)?;
        input = i;
        verification_type_info_list.push(verification_type_info);
    }
    Ok((input, verification_type_info_list))
}

fn parse_stack_map_frame_entry(input: &[u8]) -> IResult<&[u8], StackMapFrame> {
    let (input, frame_type) = be_u8(input)?;
    let stack_map_frame_kind = match frame_type {
        0..=63 => StackMapFrameKind::Same,
        64..=127 => {
            let (_, stack) = parse_verification_type_info(input)?;
            StackMapFrameKind::SameLocals1StackItem { stack }
        }
        247 => {
            let (input, offset_delta) = be_u16(input)?;
            let (_, stack) = parse_verification_type_info(input)?;
            StackMapFrameKind::SameLocals1StackItemExtended {
                offset_delta,
                stack,
            }
        }
        248..=250 => {
            let (_, offset_delta) = be_u16(input)?;
            StackMapFrameKind::Chop { offset_delta }
        }
        251 => {
            let (_, offset_delta) = be_u16(input)?;
            StackMapFrameKind::SameExtended { offset_delta }
        }
        252..=254 => {
            let (input, offset_delta) = be_u16(input)?;
            let (_, locals) = parse_verification_type_info_list(input, frame_type as u16 - 251)?;
            StackMapFrameKind::Append {
                offset_delta,
                locals,
            }
        }
        255 => {
            let (input, offset_delta) = be_u16(input)?;
            let (input, number_of_locals) = be_u16(input)?;
            let (input, locals) = parse_verification_type_info_list(input, number_of_locals)?;
            let (input, number_of_stack_items) = be_u16(input)?;
            let (_, stack) = parse_verification_type_info_list(input, number_of_stack_items)?;
            StackMapFrameKind::Full {
                offset_delta,
                number_of_locals,
                locals,
                number_of_stack_items,
                stack,
            }
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))?,
    };

    Ok((
        input,
        StackMapFrame {
            frame_type,
            kind: stack_map_frame_kind,
        },
    ))
}

fn parse_stack_map_frame(input: &[u8], count: u16) -> IResult<&[u8], Vec<StackMapFrame>> {
    let mut input = input;
    let mut stack_map_frame = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, stack_map_frame_entry) = parse_stack_map_frame_entry(input)?;
        input = i;
        stack_map_frame.push(stack_map_frame_entry);
    }
    Ok((input, stack_map_frame))
}

fn parse_bootstrap_method(input: &[u8]) -> IResult<&[u8], BootstrapMethod> {
    let (input, bootstrap_method_ref) = be_u16(input)?;
    let (input, num_bootstrap_arguments) = be_u16(input)?;
    let mut input = input;
    let mut bootstrap_arguments = Vec::with_capacity(num_bootstrap_arguments as usize);
    for _ in 0..num_bootstrap_arguments {
        let (i, bootstrap_argument) = be_u16(input)?;
        input = i;
        bootstrap_arguments.push(bootstrap_argument);
    }
    Ok((
        input,
        BootstrapMethod {
            bootstrap_method_ref,
            num_bootstrap_arguments,
            bootstrap_arguments,
        },
    ))
}

fn parse_bootstrap_methods(input: &[u8], count: u16) -> IResult<&[u8], Vec<BootstrapMethod>> {
    let mut input = input;
    let mut bootstrap_methods = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, bootstrap_method) = parse_bootstrap_method(input)?;
        input = i;
        bootstrap_methods.push(bootstrap_method);
    }
    Ok((input, bootstrap_methods))
}

fn parse_inner_class(input: &[u8]) -> IResult<&[u8], InnerClass> {
    let (input, inner_class_info_index) = be_u16(input)?;
    let (input, outer_class_info_index) = be_u16(input)?;
    let (input, inner_name_index) = be_u16(input)?;
    let (input, inner_class_access_flags) = be_u16(input)?;
    Ok((
        input,
        InnerClass {
            inner_class_info_index,
            outer_class_info_index,
            inner_name_index,
            inner_class_access_flags: InnerClassAccessFlags::from_bits(inner_class_access_flags)
                .unwrap(),
        },
    ))
}

fn parse_line_number_table_entry(input: &[u8]) -> IResult<&[u8], LineNumberTableEntry> {
    let (input, start_pc) = be_u16(input)?;
    let (input, line_number) = be_u16(input)?;
    Ok((
        input,
        LineNumberTableEntry {
            start_pc,
            line_number,
        },
    ))
}

fn parse_line_number_table(input: &[u8], count: u16) -> IResult<&[u8], Vec<LineNumberTableEntry>> {
    let mut input = input;
    let mut line_number_table = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, line_number_table_entry) = parse_line_number_table_entry(input)?;
        input = i;
        line_number_table.push(line_number_table_entry);
    }
    Ok((input, line_number_table))
}

fn parse_local_variable_table_entry(input: &[u8]) -> IResult<&[u8], LocalVariableTableEntry> {
    let (input, start_pc) = be_u16(input)?;
    let (input, length) = be_u16(input)?;
    let (input, name_index) = be_u16(input)?;
    let (input, descriptor_index) = be_u16(input)?;
    let (input, index) = be_u16(input)?;
    Ok((
        input,
        LocalVariableTableEntry {
            start_pc,
            length,
            name_index,
            descriptor_index,
            index,
        },
    ))
}

fn parse_local_variable_table(
    input: &[u8],
    count: u16,
) -> IResult<&[u8], Vec<LocalVariableTableEntry>> {
    let mut input = input;
    let mut local_variable_table = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, local_variable_table_entry) = parse_local_variable_table_entry(input)?;
        input = i;
        local_variable_table.push(local_variable_table_entry);
    }
    Ok((input, local_variable_table))
}

fn parse_local_variable_type_table_entry(
    input: &[u8],
) -> IResult<&[u8], LocalVariableTypeTableEntry> {
    let (input, start_pc) = be_u16(input)?;
    let (input, length) = be_u16(input)?;
    let (input, name_index) = be_u16(input)?;
    let (input, signature_index) = be_u16(input)?;
    let (input, index) = be_u16(input)?;
    Ok((
        input,
        LocalVariableTypeTableEntry {
            start_pc,
            length,
            name_index,
            signature_index,
            index,
        },
    ))
}

fn parse_local_variable_type_table(
    input: &[u8],
    count: u16,
) -> IResult<&[u8], Vec<LocalVariableTypeTableEntry>> {
    let mut input = input;
    let mut local_variable_type_table = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, local_variable_type_table_entry) = parse_local_variable_type_table_entry(input)?;
        input = i;
        local_variable_type_table.push(local_variable_type_table_entry);
    }
    Ok((input, local_variable_type_table))
}

fn parse_element_value_pair(input: &[u8]) -> IResult<&[u8], ElementValuePair> {
    let (input, element_name_index) = be_u16(input)?;
    let (input, value) = parse_element_value(input)?;
    Ok((
        input,
        ElementValuePair {
            element_name_index,
            value,
        },
    ))
}

fn parse_element_value(input: &[u8]) -> IResult<&[u8], ElementValue> {
    let (input, tag) = be_u8(input)?;

    match tag as char {
        'B' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Byte { const_value_index }))
        }
        'C' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Char { const_value_index }))
        }
        'D' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Double { const_value_index }))
        }
        'F' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Float { const_value_index }))
        }
        'I' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Int { const_value_index }))
        }
        'J' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Long { const_value_index }))
        }
        'S' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Short { const_value_index }))
        }
        'Z' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::Boolean { const_value_index }))
        }
        's' => {
            let (input, const_value_index) = be_u16(input)?;
            Ok((input, ElementValue::String { const_value_index }))
        }
        'e' => {
            let (input, type_name_index) = be_u16(input)?;
            let (input, const_name_index) = be_u16(input)?;
            Ok((
                input,
                ElementValue::EnumConst {
                    type_name_index,
                    const_name_index,
                },
            ))
        }
        'c' => {
            let (input, class_info_index) = be_u16(input)?;
            Ok((input, ElementValue::ClassInfoIndex { class_info_index }))
        }
        '@' => {
            let (input, annotation) = parse_annotation(input)?;
            Ok((input, ElementValue::AnnotationValue { annotation }))
        }
        '[' => {
            let (input, num_values) = be_u16(input)?;
            let mut input = input;
            let mut values = Vec::with_capacity(num_values as usize);
            for _ in 0..num_values {
                let (i, value) = parse_element_value(input)?;
                input = i;
                values.push(value);
            }
            Ok((input, ElementValue::ArrayValue { num_values, values }))
        }
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        ))),
    }
}

fn parse_annotation(input: &[u8]) -> IResult<&[u8], Annotation> {
    let (input, type_index) = be_u16(input)?;
    let (input, num_element_value_pairs) = be_u16(input)?;
    let mut input = input;
    let mut element_value_pairs = Vec::with_capacity(num_element_value_pairs as usize);
    for _ in 0..num_element_value_pairs {
        let (i, element_value_pair) = parse_element_value_pair(input)?;
        input = i;
        element_value_pairs.push(element_value_pair);
    }
    Ok((
        input,
        Annotation {
            type_index,
            num_element_value_pairs,
            element_value_pairs,
        },
    ))
}

fn parse_annotations(input: &[u8], count: u16) -> IResult<&[u8], Vec<Annotation>> {
    let mut input = input;
    let mut annotations = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, annotation) = parse_annotation(input)?;
        input = i;
        annotations.push(annotation);
    }
    Ok((input, annotations))
}

fn parse_parameter_annotation(input: &[u8]) -> IResult<&[u8], ParameterAnnotation> {
    let (input, num_annotations) = be_u16(input)?;
    let (input, annotations) = parse_annotations(input, num_annotations)?;
    Ok((
        input,
        ParameterAnnotation {
            num_annotations,
            annotations,
        },
    ))
}

fn parse_type_annotation(input: &[u8]) -> IResult<&[u8], TypeAnnotation> {
    let (input, target_type) = be_u8(input)?;
    let (input, target_info_kind) = match target_type {
        0x00 | 0x01 => parse_type_parameter_target(input)?,
        0x10 => parse_supertype_target(input)?,
        0x11 | 0x12 => parse_type_parameter_bound_target(input)?,
        0x13 | 0x14 | 0x15 => parse_empty_target(input)?,
        0x16 => parse_formal_parameter_target(input)?,
        0x17 => parse_throws_target(input)?,
        0x40 | 0x41 => parse_localvar_target(input)?,
        0x42 => parse_catch_target(input)?,
        0x43 | 0x44 | 0x45 | 0x46 => parse_offset_target(input)?,
        0x47 | 0x48 | 0x49 | 0x4A | 0x4B => parse_type_argument_target(input)?,
        _ => Err(nom::Err::Error(nom::error::Error::new(
            input,
            nom::error::ErrorKind::Tag,
        )))?,
    };

    let (input, target_path) = parse_type_path(input)?;
    let (input, type_index) = be_u16(input)?;
    let (input, num_element_value_pairs) = be_u16(input)?;
    let mut input = input;
    let mut element_value_pairs = Vec::with_capacity(num_element_value_pairs as usize);
    for _ in 0..num_element_value_pairs {
        let (i, element_value_pair) = parse_element_value_pair(input)?;
        input = i;
        element_value_pairs.push(element_value_pair);
    }

    Ok((
        input,
        TypeAnnotation {
            target_type,
            target_info: TargetInfo {
                target_type,
                target_info_kind,
            },
            target_path,
            type_index,
            num_element_value_pairs,
            element_value_pairs,
        },
    ))
}

fn parse_type_annotations(input: &[u8], count: u16) -> IResult<&[u8], Vec<TypeAnnotation>> {
    let mut input = input;
    let mut type_annotations = Vec::with_capacity(count as usize);
    for _ in 0..count {
        let (i, type_annotation) = parse_type_annotation(input)?;
        input = i;
        type_annotations.push(type_annotation);
    }
    Ok((input, type_annotations))
}

fn parse_type_parameter_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, type_parameter_index) = be_u8(input)?;
    Ok((
        input,
        TargetInfoKind::TypeParameter {
            type_parameter_index,
        },
    ))
}

fn parse_supertype_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, super_type_index) = be_u16(input)?;
    Ok((input, TargetInfoKind::SuperType { super_type_index }))
}

fn parse_type_parameter_bound_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, type_parameter_index) = be_u8(input)?;
    let (input, bound_index) = be_u8(input)?;
    Ok((
        input,
        TargetInfoKind::TypeParameterBound {
            type_parameter_index,
            bound_index,
        },
    ))
}

fn parse_empty_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    Ok((input, TargetInfoKind::Empty))
}

fn parse_formal_parameter_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, formal_parameter_index) = be_u8(input)?;
    Ok((
        input,
        TargetInfoKind::FormalParameter {
            formal_parameter_index,
        },
    ))
}

fn parse_throws_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, throws_type_index) = be_u16(input)?;
    Ok((input, TargetInfoKind::Throws { throws_type_index }))
}

fn parse_localvar_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, table_length) = be_u16(input)?;
    let mut input = input;
    let mut table = Vec::with_capacity(table_length as usize);
    for _ in 0..table_length {
        let (i, start_pc) = be_u16(input)?;
        let (i, length) = be_u16(i)?;
        let (i, index) = be_u16(i)?;
        input = i;
        table.push(LocalVarTarget {
            start_pc,
            length,
            index,
        });
    }
    Ok((
        input,
        TargetInfoKind::LocalVar {
            table_length,
            table,
        },
    ))
}

fn parse_catch_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, exception_table_index) = be_u16(input)?;
    Ok((
        input,
        TargetInfoKind::Catch {
            exception_table_index,
        },
    ))
}

fn parse_offset_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, offset) = be_u16(input)?;
    Ok((input, TargetInfoKind::Offset { offset }))
}

fn parse_type_argument_target(input: &[u8]) -> IResult<&[u8], TargetInfoKind> {
    let (input, offset) = be_u16(input)?;
    let (input, type_argument_index) = be_u8(input)?;
    Ok((
        input,
        TargetInfoKind::TypeArgument {
            offset,
            type_argument_index,
        },
    ))
}

fn parse_type_path(input: &[u8]) -> IResult<&[u8], TypePath> {
    let (input, path_length) = be_u8(input)?;
    let mut input = input;
    let mut path = Vec::with_capacity(path_length as usize);
    for _ in 0..path_length {
        let (i, type_path_entry) = parse_path(input)?;
        input = i;
        path.push(type_path_entry);
    }
    Ok((input, TypePath { path_length, path }))
}

fn parse_path(input: &[u8]) -> IResult<&[u8], Path> {
    let (input, type_path_kind) = be_u8(input)?;
    let (input, type_argument_index) = be_u8(input)?;
    Ok((
        input,
        Path {
            type_path_kind,
            type_argument_index,
        },
    ))
}

fn parse_method_parameter(input: &[u8]) -> IResult<&[u8], Parameter> {
    let (input, name_index) = be_u16(input)?;
    let (input, access_flags) = be_u16(input)?;
    Ok((
        input,
        Parameter {
            name_index,
            access_flags,
        },
    ))
}

fn parse_requires_info(input: &[u8]) -> IResult<&[u8], ModuleRequires> {
    let (input, requires_index) = be_u16(input)?;
    let (input, requires_flags) = be_u16(input)?;
    let (input, requires_version_index) = be_u16(input)?;
    Ok((
        input,
        ModuleRequires {
            requires_index,
            requires_flags,
            requires_version_index,
        },
    ))
}

fn parse_exports_info(input: &[u8]) -> IResult<&[u8], ModuleExports> {
    let (input, exports_index) = be_u16(input)?;
    let (input, exports_flags) = be_u16(input)?;
    let (input, exports_to_count) = be_u16(input)?;
    let mut input = input;
    let mut exports_to_index = Vec::with_capacity(exports_to_count as usize);
    for _ in 0..exports_to_count {
        let (i, exports_to_index_entry) = be_u16(input)?;
        input = i;
        exports_to_index.push(exports_to_index_entry);
    }
    Ok((
        input,
        ModuleExports {
            exports_index,
            exports_flags,
            exports_to_count,
            exports_to_index,
        },
    ))
}

fn parse_opens_info(input: &[u8]) -> IResult<&[u8], ModuleOpens> {
    let (input, opens_index) = be_u16(input)?;
    let (input, opens_flags) = be_u16(input)?;
    let (input, opens_to_count) = be_u16(input)?;
    let mut input = input;
    let mut opens_to_index = Vec::with_capacity(opens_to_count as usize);
    for _ in 0..opens_to_count {
        let (i, opens_to_index_entry) = be_u16(input)?;
        input = i;
        opens_to_index.push(opens_to_index_entry);
    }
    Ok((
        input,
        ModuleOpens {
            opens_index,
            opens_flags,
            opens_to_count,
            opens_to_index,
        },
    ))
}

fn parse_provides_info(input: &[u8]) -> IResult<&[u8], ModuleProvides> {
    let (input, provides_index) = be_u16(input)?;
    let (input, provides_with_count) = be_u16(input)?;
    let mut input = input;
    let mut provides_with_index = Vec::with_capacity(provides_with_count as usize);
    for _ in 0..provides_with_count {
        let (i, provides_with_index_entry) = be_u16(input)?;
        input = i;
        provides_with_index.push(provides_with_index_entry);
    }
    Ok((
        input,
        ModuleProvides {
            provides_index,
            provides_with_count,
            provides_with_index,
        },
    ))
}

pub fn parse_class_file(input: &[u8]) -> Option<ClassFile> {
    let (input, _) = parse_magic_number(input).unwrap();
    let (input, minor_version) = parse_version(input).unwrap();
    let (input, major_version) = parse_version(input).unwrap();

    let (input, constant_pool_count) = parse_constant_pool_count(input).unwrap();

    let (input, constant_pool) = parse_constant_pool(input, constant_pool_count).unwrap();

    let parser = ClassFileParser {
        constant_pool: &constant_pool,
    };

    let (input, access_flags) = parse_access_flags(input).unwrap();

    let (input, this_class) = parse_this_class(input).unwrap();

    let (input, super_class) = parse_super_class(input).unwrap();

    let (input, interfaces_count) = parse_interfaces_count(input).unwrap();

    let (input, interfaces) = parse_interfaces(input, interfaces_count).unwrap();

    let (input, fields_count) = parse_fields_count(input).unwrap();

    let (input, fields) = parser.parse_fields(input, fields_count).unwrap();

    let (input, methods_count) = parse_methods_count(input).unwrap();

    let (input, methods) = parser.parse_methods(input, methods_count).unwrap();

    let (input, attributes_count) = parse_attributes_count(input).unwrap();

    let (input, attributes) = parser.parse_attributes(input, attributes_count).unwrap();

    if !input.is_empty() {
        None
    } else {
        Some(ClassFile {
            minor_version,
            major_version,
            constant_pool_count,
            constant_pool,
            access_flags,
            this_class,
            super_class,
            interfaces_count,
            interfaces,
            fields_count,
            fields,
            methods_count,
            methods,
            attributes_count,
            attributes,
        })
    }
}

struct ClassFileParser<'a> {
    constant_pool: &'a [ConstantPoolInfo],
}

impl<'a> ClassFileParser<'a> {
    fn parse_fields<'b>(
        &'_ self,
        input: &'b [u8],
        count: u16,
    ) -> IResult<&'b [u8], Vec<FieldInfo>> {
        let mut input = input;
        let mut fields = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (i, field) = self.parse_field_info(input)?;
            input = i;
            fields.push(field);
        }
        Ok((input, fields))
    }

    fn parse_field_info<'b>(&'_ self, input: &'b [u8]) -> IResult<&'b [u8], FieldInfo> {
        let (input, access_flags) = be_u16(input)?;
        let (input, name_index) = be_u16(input)?;
        let (input, descriptor_index) = be_u16(input)?;
        let (input, attributes_count) = be_u16(input)?;
        let (input, attributes) = self.parse_attributes(input, attributes_count)?;
        Ok((
            input,
            FieldInfo {
                access_flags: FieldAccessFlags::from_bits(access_flags).unwrap(),
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            },
        ))
    }

    fn parse_attributes<'b>(
        &'_ self,
        input: &'b [u8],
        count: u16,
    ) -> IResult<&'b [u8], Vec<Attribute>> {
        let mut input = input;
        let mut attributes = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (i, attribute) = self.parse_attribute_info(input)?;
            input = i;
            attributes.push(attribute);
        }
        Ok((input, attributes))
    }

    fn parse_attribute_info<'b>(&'_ self, input: &'b [u8]) -> IResult<&'b [u8], Attribute> {
        let (input, attribute_name_index) = be_u16(input)?;

        let attribute_name = match &self.constant_pool[attribute_name_index as usize] {
            ConstantPoolInfo::Utf8 {
                length: _,
                bytes: _,
                utf8_str: s,
            } => s,
            _ => Err(nom::Err::Error(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Tag,
            )))?,
        };

        let (input, attribute_length) = be_u32(input)?;
        let (input, info) = take(attribute_length)(input)?;

        let (_, kind) = match attribute_name as &str {
            "ConstantValue" => {
                let (info, constant_value_index) = be_u16(info)?;
                (
                    info,
                    AttributeKind::ConstantValue {
                        constant_value_index,
                    },
                )
            }
            "Code" => {
                let (info, max_stack) = be_u16(info)?;
                let (info, max_locals) = be_u16(info)?;
                let (info, code_length) = be_u32(info)?;
                let (info, code) = take(code_length)(info)?;
                let (info, exception_table_length) = be_u16(info)?;
                let (info, exception_table) = parse_exception_table(info, exception_table_length)?;
                let (info, attributes_count) = be_u16(info)?;
                let (info, attributes) = self.parse_attributes(info, attributes_count)?;
                (
                    info,
                    AttributeKind::Code {
                        max_stack,
                        max_locals,
                        code_length,
                        code: code.to_vec(),
                        exception_table_length,
                        exception_table,
                        attributes_count,
                        attributes,
                    },
                )
            }
            "StackMapTable" => {
                let (info, number_of_entries) = be_u16(info)?;
                let (info, entries) = parse_stack_map_frame(info, number_of_entries)?;
                (
                    info,
                    AttributeKind::StackMapTable {
                        number_of_entries,
                        entries,
                    },
                )
            }
            "BootstrapMethods" => {
                let (info, num_bootstrap_methods) = be_u16(info)?;
                let (_, bootstrap_methods) = parse_bootstrap_methods(info, num_bootstrap_methods)?;
                (
                    info,
                    AttributeKind::BootstrapMethods {
                        num_bootstrap_methods,
                        bootstrap_methods,
                    },
                )
            }
            "NestHost" => {
                let (info, host_class_index) = be_u16(info)?;
                (info, AttributeKind::NestHost { host_class_index })
            }
            "NestMembers" => {
                let (mut info, number_of_classes) = be_u16(info)?;
                let mut classes = Vec::with_capacity(number_of_classes as usize);
                for _ in 0..number_of_classes {
                    let (i, class_index) = be_u16(info)?;
                    info = i;
                    classes.push(class_index);
                }

                (
                    info,
                    AttributeKind::NestMembers {
                        number_of_classes,
                        classes,
                    },
                )
            }
            "PermittedSubclasses" => {
                let (mut info, number_of_classes) = be_u16(info)?;
                let mut classes = Vec::with_capacity(number_of_classes as usize);
                for _ in 0..number_of_classes {
                    let (i, class_index) = be_u16(info)?;
                    info = i;
                    classes.push(class_index);
                }

                (
                    info,
                    AttributeKind::PermittedSubclasses {
                        number_of_classes,
                        classes,
                    },
                )
            }
            "Exceptions" => {
                let (mut info, number_of_exceptions) = be_u16(info)?;
                let mut exception_index_table = Vec::with_capacity(number_of_exceptions as usize);
                for _ in 0..number_of_exceptions {
                    let (i, exception_index) = be_u16(info)?;
                    info = i;
                    exception_index_table.push(exception_index);
                }
                (
                    info,
                    AttributeKind::Exceptions {
                        number_of_exceptions,
                        exception_index_table,
                    },
                )
            }
            "InnerClasses" => {
                let (mut info, number_of_classes) = be_u16(info)?;
                let mut classes = Vec::with_capacity(number_of_classes as usize);
                for _ in 0..number_of_classes {
                    let (i, inner_class) = parse_inner_class(info)?;
                    info = i;
                    classes.push(inner_class);
                }
                (
                    info,
                    AttributeKind::InnerClasses {
                        number_of_classes,
                        classes,
                    },
                )
            }
            "EnclosingMethod" => {
                let (info, class_index) = be_u16(info)?;
                let (info, method_index) = be_u16(info)?;
                (
                    info,
                    AttributeKind::EnclosingMethod {
                        class_index,
                        method_index,
                    },
                )
            }
            "Synthetic" => (info, AttributeKind::Synthetic),
            "Signature" => {
                let (info, signature_index) = be_u16(info)?;
                (info, AttributeKind::Signature { signature_index })
            }
            "Record" => {
                let (info, components_count) = be_u16(info)?;
                let (info, components) = self.parse_components(info, components_count)?;
                (
                    info,
                    AttributeKind::Record {
                        components_count,
                        components,
                    },
                )
            }
            "SourceFile" => {
                let (info, sourcefile_index) = be_u16(info)?;
                (info, AttributeKind::SourceFile { sourcefile_index })
            }
            "LineNumberTable" => {
                let (info, line_number_table_length) = be_u16(info)?;
                let (info, line_number_table) =
                    parse_line_number_table(info, line_number_table_length)?;
                (
                    info,
                    AttributeKind::LineNumberTable {
                        line_number_table_length,
                        line_number_table,
                    },
                )
            }
            "LocalVariableTable" => {
                let (info, local_variable_table_length) = be_u16(info)?;
                let (info, local_variable_table) =
                    parse_local_variable_table(info, local_variable_table_length)?;
                (
                    info,
                    AttributeKind::LocalVariableTable {
                        local_variable_table_length,
                        local_variable_table,
                    },
                )
            }
            "LocalVariableTypeTable" => {
                let (info, local_variable_type_table_length) = be_u16(info)?;
                let (info, local_variable_type_table) =
                    parse_local_variable_type_table(info, local_variable_type_table_length)?;
                (
                    info,
                    AttributeKind::LocalVariableTypeTable {
                        local_variable_type_table_length,
                        local_variable_type_table,
                    },
                )
            }
            "SourceDebugExtension" => {
                let (info, debug_extension) = take(attribute_length)(info)?;
                (
                    info,
                    AttributeKind::SourceDebugExtension {
                        debug_extension: debug_extension.to_vec(),
                    },
                )
            }
            "Deprecated" => (info, AttributeKind::Deprecated),
            "RuntimeVisibleAnnotations" => {
                let (info, num_annotations) = be_u16(info)?;
                let (info, annotations) = parse_annotations(info, num_annotations)?;
                (
                    info,
                    AttributeKind::RuntimeVisibleAnnotations {
                        num_annotations,
                        annotations,
                    },
                )
            }
            "RuntimeInvisibleAnnotations" => {
                let (info, num_annotations) = be_u16(info)?;
                let (info, annotations) = parse_annotations(info, num_annotations)?;
                (
                    info,
                    AttributeKind::RuntimeInvisibleAnnotations {
                        num_annotations,
                        annotations,
                    },
                )
            }
            "RuntimeVisibleParameterAnnotations" => {
                let (info, num_parameters) = be_u8(info)?;
                let mut input = info;
                let mut parameter_annotations = Vec::with_capacity(num_parameters as usize);
                for _ in 0..num_parameters {
                    let (i, annotation) = parse_parameter_annotation(input)?;
                    input = i;
                    parameter_annotations.push(annotation);
                }
                (
                    input,
                    AttributeKind::RuntimeVisibleParameterAnnotations {
                        num_parameters,
                        parameter_annotations,
                    },
                )
            }
            "RuntimeInvisibleParameterAnnotations" => {
                let (info, num_parameters) = be_u8(info)?;
                let mut input = info;
                let mut parameter_annotations = Vec::with_capacity(num_parameters as usize);
                for _ in 0..num_parameters {
                    let (i, annotation) = parse_parameter_annotation(input)?;
                    input = i;
                    parameter_annotations.push(annotation);
                }
                (
                    input,
                    AttributeKind::RuntimeInvisibleParameterAnnotations {
                        num_parameters,
                        parameter_annotations,
                    },
                )
            }
            "RuntimeVisibleTypeAnnotations" => {
                let (info, num_annotations) = be_u16(info)?;
                let (info, type_annotations) = parse_type_annotations(info, num_annotations)?;
                (
                    info,
                    AttributeKind::RuntimeVisibleTypeAnnotations {
                        num_annotations,
                        type_annotations,
                    },
                )
            }
            "RuntimeInvisibleTypeAnnotations" => {
                let (info, num_annotations) = be_u16(info)?;
                let (info, type_annotations) = parse_type_annotations(info, num_annotations)?;
                (
                    info,
                    AttributeKind::RuntimeInvisibleTypeAnnotations {
                        num_annotations,
                        type_annotations,
                    },
                )
            }
            "AnnotationDefault" => {
                let (info, default_value) = parse_element_value(info)?;
                (info, AttributeKind::AnnotationDefault { default_value })
            }
            "MethodParameters" => {
                let (info, parameters_count) = be_u8(info)?;
                let mut input = info;
                let mut parameters = Vec::with_capacity(parameters_count as usize);
                for _ in 0..parameters_count {
                    let (i, parameter) = parse_method_parameter(input)?;
                    input = i;
                    parameters.push(parameter);
                }
                (
                    input,
                    AttributeKind::MethodParameters {
                        parameters_count,
                        parameters,
                    },
                )
            }
            "Module" => {
                let (info, module_name_index) = be_u16(info)?;
                let (info, module_flags) = be_u16(info)?;
                let (info, module_version_index) = be_u16(info)?;
                let (info, requires_count) = be_u16(info)?;

                let mut input = info;
                let mut requires = Vec::with_capacity(requires_count as usize);
                for _ in 0..requires_count {
                    let (i, requires_info) = parse_requires_info(input)?;
                    input = i;
                    requires.push(requires_info);
                }

                let (mut info, exports_count) = be_u16(input)?;
                let mut exports = Vec::with_capacity(exports_count as usize);
                for _ in 0..exports_count {
                    let (i, exports_info) = parse_exports_info(info)?;
                    info = i;
                    exports.push(exports_info);
                }

                let (mut info, opens_count) = be_u16(info)?;
                let mut opens = Vec::with_capacity(opens_count as usize);
                for _ in 0..opens_count {
                    let (i, opens_info) = parse_opens_info(info)?;
                    info = i;
                    opens.push(opens_info);
                }

                let (mut info, uses_count) = be_u16(info)?;
                let mut uses_index = Vec::with_capacity(uses_count as usize);
                for _ in 0..uses_count {
                    let (i, uses_index_info) = be_u16(info)?;
                    info = i;
                    uses_index.push(uses_index_info);
                }

                let (mut info, provides_count) = be_u16(info)?;
                let mut provides = Vec::with_capacity(provides_count as usize);
                for _ in 0..provides_count {
                    let (i, provides_info) = parse_provides_info(info)?;
                    info = i;
                    provides.push(provides_info);
                }

                (
                    info,
                    AttributeKind::Module {
                        module_name_index,
                        module_flags,
                        module_version_index,
                        requires_count,
                        requires,
                        exports_count,
                        exports,
                        opens_count,
                        opens,
                        uses_count,
                        uses_index,
                        provides_count,
                        provides,
                    },
                )
            }
            "ModulePackages" => {
                let (info, package_count) = be_u16(info)?;
                let mut input = info;
                let mut package_index = Vec::with_capacity(package_count as usize);
                for _ in 0..package_count {
                    let (i, package_index_entry) = be_u16(input)?;
                    input = i;
                    package_index.push(package_index_entry);
                }
                (
                    input,
                    AttributeKind::ModulePackages {
                        package_count,
                        package_index,
                    },
                )
            }
            "ModuleMainClass" => {
                let (info, main_class_index) = be_u16(info)?;
                (info, AttributeKind::ModuleMainClass { main_class_index })
            }
            _ => {
                let (input, _) = take(attribute_length)(info)?;
                (input, AttributeKind::Unknown)
            }
        };

        Ok((
            input,
            Attribute {
                kind,
                attribute_name_index,
                attribute_length,
            },
        ))
    }

    fn parse_methods<'b>(
        &'_ self,
        input: &'b [u8],
        count: u16,
    ) -> IResult<&'b [u8], Vec<MethodInfo>> {
        let mut input = input;
        let mut methods = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (i, method) = self.parse_method_info(input)?;
            input = i;
            methods.push(method);
        }
        Ok((input, methods))
    }

    fn parse_method_info<'b>(&'_ self, input: &'b [u8]) -> IResult<&'b [u8], MethodInfo> {
        let (input, access_flags) = be_u16(input)?;
        let (input, name_index) = be_u16(input)?;
        let (input, descriptor_index) = be_u16(input)?;
        let (input, attributes_count) = be_u16(input)?;
        let (input, attributes) = self.parse_attributes(input, attributes_count)?;
        Ok((
            input,
            MethodInfo {
                access_flags: MethodAccessFlags::from_bits(access_flags).unwrap(),
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            },
        ))
    }

    fn parse_component_info<'b>(&'_ self, input: &'b [u8]) -> IResult<&'b [u8], RecordComponent> {
        let (input, name_index) = be_u16(input)?;
        let (input, descriptor_index) = be_u16(input)?;
        let (input, attributes_count) = be_u16(input)?;
        let (input, attributes) = self.parse_attributes(input, attributes_count)?;
        Ok((
            input,
            RecordComponent {
                name_index,
                descriptor_index,
                attributes_count,
                attributes,
            },
        ))
    }

    fn parse_components<'b>(
        &'_ self,
        input: &'b [u8],
        count: u16,
    ) -> IResult<&'b [u8], Vec<RecordComponent>> {
        let mut input = input;
        let mut components = Vec::with_capacity(count as usize);
        for _ in 0..count {
            let (i, component) = self.parse_component_info(input)?;
            input = i;
            components.push(component);
        }
        Ok((input, components))
    }
}
