use std::fmt;
use std::sync::Arc;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MetaType {
    Void,
    Unknown,
    Int,
    Uint,
    Bool,
    Code,
    Float,
    Ptr,
    Array,
    Struct,
    Union,
    Enum,
    FuncProto,
    Utf8,
    Utf16,
    Utf32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DataType {
    pub name: String,
    pub size: usize,
    pub meta: MetaType,
    pub description: String,
}

impl DataType {
    pub fn new(name: impl Into<String>, size: usize, meta: MetaType) -> Self {
        Self {
            name: name.into(),
            size,
            meta,
            description: String::new(),
        }
    }

    pub fn void() -> Self {
        Self::new("void", 0, MetaType::Void)
    }

    pub fn bool_type() -> Self {
        Self::new("bool", 1, MetaType::Bool)
    }

    pub fn u8() -> Self {
        Self::new("uint8_t", 1, MetaType::Uint)
    }

    pub fn i8() -> Self {
        Self::new("int8_t", 1, MetaType::Int)
    }

    pub fn u16() -> Self {
        Self::new("uint16_t", 2, MetaType::Uint)
    }

    pub fn i16() -> Self {
        Self::new("int16_t", 2, MetaType::Int)
    }

    pub fn u32() -> Self {
        Self::new("uint32_t", 4, MetaType::Uint)
    }

    pub fn i32() -> Self {
        Self::new("int32_t", 4, MetaType::Int)
    }

    pub fn u64() -> Self {
        Self::new("uint64_t", 8, MetaType::Uint)
    }

    pub fn i64() -> Self {
        Self::new("int64_t", 8, MetaType::Int)
    }

    pub fn f32() -> Self {
        Self::new("float", 4, MetaType::Float)
    }

    pub fn f64() -> Self {
        Self::new("double", 8, MetaType::Float)
    }

    pub fn pointer(pointee: Arc<DataType>, ptr_size: usize) -> Self {
        Self::new(format!("{}*", pointee.name), ptr_size, MetaType::Ptr)
    }

    pub fn array(element: Arc<DataType>, count: usize) -> Self {
        Self::new(
            format!("{}[{}]", element.name, count),
            element.size * count,
            MetaType::Array,
        )
    }

    pub fn unknown(size: usize) -> Self {
        Self::new(format!("undefined{}", size), size, MetaType::Unknown)
    }

    pub fn is_integer(&self) -> bool {
        matches!(self.meta, MetaType::Int | MetaType::Uint)
    }

    pub fn is_float(&self) -> bool {
        self.meta == MetaType::Float
    }

    pub fn is_void(&self) -> bool {
        self.meta == MetaType::Void
    }

    pub fn is_pointer(&self) -> bool {
        self.meta == MetaType::Ptr
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.name)
    }
}

#[derive(Debug, Clone)]
pub struct StructField {
    pub name: String,
    pub offset: usize,
    pub data_type: Arc<DataType>,
}

#[derive(Debug, Clone)]
pub struct StructType {
    pub base: DataType,
    pub fields: Vec<StructField>,
}

impl StructType {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            base: DataType::new(name, 0, MetaType::Struct),
            fields: Vec::new(),
        }
    }

    pub fn add_field(&mut self, name: impl Into<String>, data_type: Arc<DataType>) {
        let offset = self.base.size;
        self.fields.push(StructField {
            name: name.into(),
            offset,
            data_type: data_type.clone(),
        });
        self.base.size += data_type.size;
    }
}

#[derive(Debug, Clone)]
pub struct EnumMember {
    pub name: String,
    pub value: i64,
}

#[derive(Debug, Clone)]
pub struct EnumType {
    pub base: DataType,
    pub members: Vec<EnumMember>,
}

impl EnumType {
    pub fn new(name: impl Into<String>, size: usize) -> Self {
        Self {
            base: DataType::new(name, size, MetaType::Enum),
            members: Vec::new(),
        }
    }

    pub fn add_member(&mut self, name: impl Into<String>, value: i64) {
        self.members.push(EnumMember {
            name: name.into(),
            value,
        });
    }
}

#[derive(Debug, Default)]
pub struct DataTypeManager {
    types: Vec<Arc<DataType>>,
}

impl DataTypeManager {
    pub fn new() -> Self {
        let mut mgr = Self::default();
        mgr.register_builtins();
        mgr
    }

    fn register_builtins(&mut self) {
        self.add(DataType::void());
        self.add(DataType::bool_type());
        self.add(DataType::u8());
        self.add(DataType::i8());
        self.add(DataType::u16());
        self.add(DataType::i16());
        self.add(DataType::u32());
        self.add(DataType::i32());
        self.add(DataType::u64());
        self.add(DataType::i64());
        self.add(DataType::f32());
        self.add(DataType::f64());
    }

    pub fn add(&mut self, dt: DataType) -> Arc<DataType> {
        let arc = Arc::new(dt);
        self.types.push(arc.clone());
        arc
    }

    pub fn find_by_name(&self, name: &str) -> Option<Arc<DataType>> {
        self.types.iter().find(|t| t.name == name).cloned()
    }

    pub fn find_by_size_and_meta(&self, size: usize, meta: MetaType) -> Option<Arc<DataType>> {
        self.types
            .iter()
            .find(|t| t.size == size && t.meta == meta)
            .cloned()
    }

    pub fn types(&self) -> &[Arc<DataType>] {
        &self.types
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builtin_types() {
        let mgr = DataTypeManager::new();
        let void = mgr.find_by_name("void").unwrap();
        assert!(void.is_void());
        assert_eq!(void.size, 0);

        let u32_type = mgr.find_by_name("uint32_t").unwrap();
        assert!(u32_type.is_integer());
        assert_eq!(u32_type.size, 4);

        let f64_type = mgr.find_by_name("double").unwrap();
        assert!(f64_type.is_float());
        assert_eq!(f64_type.size, 8);
    }

    #[test]
    fn pointer_type() {
        let base = Arc::new(DataType::i32());
        let ptr = DataType::pointer(base, 8);
        assert!(ptr.is_pointer());
        assert_eq!(ptr.size, 8);
        assert_eq!(ptr.name, "int32_t*");
    }

    #[test]
    fn struct_type() {
        let mut s = StructType::new("my_struct");
        s.add_field("x", Arc::new(DataType::i32()));
        s.add_field("y", Arc::new(DataType::i32()));
        assert_eq!(s.base.size, 8);
        assert_eq!(s.fields.len(), 2);
        assert_eq!(s.fields[1].offset, 4);
    }

    #[test]
    fn enum_type() {
        let mut e = EnumType::new("color", 4);
        e.add_member("RED", 0);
        e.add_member("GREEN", 1);
        e.add_member("BLUE", 2);
        assert_eq!(e.members.len(), 3);
        assert_eq!(e.base.size, 4);
    }

    #[test]
    fn find_by_size_and_meta() {
        let mgr = DataTypeManager::new();
        let result = mgr.find_by_size_and_meta(4, MetaType::Uint);
        assert_eq!(result.unwrap().name, "uint32_t");
    }
}
