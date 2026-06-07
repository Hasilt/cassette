use wasmtime::{ExternType, FuncType, ValType};

pub fn format_signature(func_ty: &FuncType) -> String {
    let params: Vec<String> = func_ty.params().map(|p| p.to_string()).collect();
    let results: Vec<String> = func_ty.results().map(|r| r.to_string()).collect();
    format!("({}) -> ({})", params.join(", "), results.join(", "))
}

pub fn kind_label(ty: &ExternType) -> &'static str {
    match ty {
        ExternType::Func(_) => "function",
        ExternType::Memory(_) => "memory",
        ExternType::Table(_) => "table",
        ExternType::Global(_) => "global",
        ExternType::Tag(_) => "tag",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExportKind {
    Function,
    Memory,
    Table,
    Global,
    Tag,
}

impl std::fmt::Display for ExportKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExportKind::Function => write!(f, "function"),
            ExportKind::Memory => write!(f, "memory"),
            ExportKind::Table => write!(f, "table"),
            ExportKind::Global => write!(f, "global"),
            ExportKind::Tag => write!(f, "tag"),
        }
    }
}

impl ExportKind {
    pub fn from_extern_type(ty: &ExternType) -> Self {
        match ty {
            ExternType::Func(_) => ExportKind::Function,
            ExternType::Memory(_) => ExportKind::Memory,
            ExternType::Table(_) => ExportKind::Table,
            ExternType::Global(_) => ExportKind::Global,
            ExternType::Tag(_) => ExportKind::Tag,
        }
    }
}

#[derive(Debug, Clone)]
pub struct FuncSignature {
    pub params: Vec<ValType>,
    pub results: Vec<ValType>,
}

impl FuncSignature {
    pub fn from_func_type(ft: &FuncType) -> Self {
        Self {
            params: ft.params().collect(),
            results: ft.results().collect(),
        }
    }

    pub fn format(&self) -> String {
        let params: Vec<String> = self.params.iter().map(|p| p.to_string()).collect();
        let results: Vec<String> = self.results.iter().map(|r| r.to_string()).collect();
        format!("({}) -> ({})", params.join(", "), results.join(", "))
    }
}

#[derive(Debug, Clone)]
pub struct ExportInfo {
    pub name: String,
    pub kind: ExportKind,
    pub signature: Option<FuncSignature>,
}