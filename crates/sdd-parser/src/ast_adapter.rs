use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AstNode {
    pub kind: String,
    pub name: Option<String>,
    pub start_line: usize,
    pub end_line: usize,
    pub start_column: usize,
    pub end_column: usize,
    pub children: Vec<AstNode>,
    pub metadata: HashMap<String, String>,
}

impl AstNode {
    pub fn new(kind: impl Into<String>, start_line: usize, end_line: usize) -> Self {
        Self {
            kind: kind.into(),
            name: None,
            start_line,
            end_line,
            start_column: 0,
            end_column: 0,
            children: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    pub fn with_name(mut self, name: impl Into<String>) -> Self {
        self.name = Some(name.into());
        self
    }

    pub fn with_position(mut self, start_col: usize, end_col: usize) -> Self {
        self.start_column = start_col;
        self.end_column = end_col;
        self
    }

    pub fn add_child(&mut self, child: AstNode) {
        self.children.push(child);
    }

    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub name: String,
    pub parameters: Vec<Parameter>,
    pub return_type: Option<String>,
    pub visibility: Visibility,
    pub is_async: bool,
    pub is_static: bool,
    pub doc_comment: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: Option<String>,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum Visibility {
    Public,
    Private,
    Protected,
    Internal,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypeSignature {
    pub name: String,
    pub kind: TypeKind,
    pub generics: Vec<String>,
    pub fields: Vec<FieldSignature>,
    pub methods: Vec<FunctionSignature>,
    pub implements: Vec<String>,
    pub visibility: Visibility,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum TypeKind {
    Struct,
    Class,
    Interface,
    Trait,
    Enum,
    TypeAlias,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSignature {
    pub name: String,
    pub field_type: Option<String>,
    pub visibility: Visibility,
    pub is_mutable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleSignature {
    pub name: String,
    pub imports: Vec<Import>,
    pub functions: Vec<FunctionSignature>,
    pub types: Vec<TypeSignature>,
    pub submodules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub path: String,
    pub aliases: Vec<String>,
    pub is_wildcard: bool,
}

pub trait AstAdapter {
    fn parse_file(&self, path: &Path) -> Result<AstNode, AstError>;
    fn parse_source(&self, source: &str, language: &str) -> Result<AstNode, AstError>;
    fn extract_signatures(&self, node: &AstNode) -> ModuleSignature;
    fn supported_languages(&self) -> Vec<&str>;
}

#[derive(Debug, thiserror::Error)]
pub enum AstError {
    #[error("unsupported language: {0}")]
    UnsupportedLanguage(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("file not found: {0}")]
    FileNotFound(String),
    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

pub struct TreeSitterAdapter {
    languages: Vec<String>,
}

impl TreeSitterAdapter {
    pub fn new() -> Self {
        Self {
            languages: vec![
                "rust".to_string(),
                "python".to_string(),
                "javascript".to_string(),
                "typescript".to_string(),
                "go".to_string(),
            ],
        }
    }

    fn detect_language(&self, path: &Path) -> Option<String> {
        let ext = path.extension()?.to_str()?;
        match ext {
            "rs" => Some("rust".to_string()),
            "py" => Some("python".to_string()),
            "js" => Some("javascript".to_string()),
            "ts" => Some("typescript".to_string()),
            "go" => Some("go".to_string()),
            _ => None,
        }
    }
}

impl Default for TreeSitterAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl AstAdapter for TreeSitterAdapter {
    fn parse_file(&self, path: &Path) -> Result<AstNode, AstError> {
        if !path.exists() {
            return Err(AstError::FileNotFound(path.to_string_lossy().to_string()));
        }

        let source = std::fs::read_to_string(path)?;
        let language = self
            .detect_language(path)
            .ok_or_else(|| AstError::UnsupportedLanguage("unknown extension".to_string()))?;

        self.parse_source(&source, &language)
    }

    fn parse_source(&self, source: &str, language: &str) -> Result<AstNode, AstError> {
        if !self.languages.contains(&language.to_string()) {
            return Err(AstError::UnsupportedLanguage(language.to_string()));
        }

        let mut root = AstNode::new("module", 0, source.lines().count());

        for (line_num, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("fn ") || trimmed.starts_with("pub fn ") {
                let name = extract_function_name(trimmed);
                let mut func_node =
                    AstNode::new("function", line_num, line_num).with_position(0, line.len());
                if let Some(name) = name {
                    func_node = func_node.with_name(name);
                }
                root.add_child(func_node);
            }

            if trimmed.starts_with("struct ") || trimmed.starts_with("pub struct ") {
                let name = extract_type_name(trimmed);
                let mut struct_node =
                    AstNode::new("struct", line_num, line_num).with_position(0, line.len());
                if let Some(name) = name {
                    struct_node = struct_node.with_name(name);
                }
                root.add_child(struct_node);
            }

            if trimmed.starts_with("trait ") || trimmed.starts_with("pub trait ") {
                let name = extract_type_name(trimmed);
                let mut trait_node =
                    AstNode::new("trait", line_num, line_num).with_position(0, line.len());
                if let Some(name) = name {
                    trait_node = trait_node.with_name(name);
                }
                root.add_child(trait_node);
            }

            if trimmed.starts_with("impl ") {
                let name = extract_impl_name(trimmed);
                let mut impl_node =
                    AstNode::new("impl", line_num, line_num).with_position(0, line.len());
                if let Some(name) = name {
                    impl_node = impl_node.with_name(name);
                }
                root.add_child(impl_node);
            }

            if trimmed.starts_with("use ") {
                let mut use_node =
                    AstNode::new("use", line_num, line_num).with_position(0, line.len());
                use_node = use_node.with_metadata(
                    "path",
                    trimmed.trim_start_matches("use ").trim_end_matches(';'),
                );
                root.add_child(use_node);
            }
        }

        Ok(root)
    }

    fn extract_signatures(&self, node: &AstNode) -> ModuleSignature {
        let mut module = ModuleSignature {
            name: "module".to_string(),
            imports: Vec::new(),
            functions: Vec::new(),
            types: Vec::new(),
            submodules: Vec::new(),
        };

        for child in &node.children {
            match child.kind.as_str() {
                "function" => {
                    if let Some(name) = &child.name {
                        module.functions.push(FunctionSignature {
                            name: name.clone(),
                            parameters: Vec::new(),
                            return_type: None,
                            visibility: Visibility::Private,
                            is_async: false,
                            is_static: false,
                            doc_comment: None,
                        });
                    }
                }
                "struct" | "trait" => {
                    if let Some(name) = &child.name {
                        module.types.push(TypeSignature {
                            name: name.clone(),
                            kind: if child.kind == "struct" {
                                TypeKind::Struct
                            } else {
                                TypeKind::Trait
                            },
                            generics: Vec::new(),
                            fields: Vec::new(),
                            methods: Vec::new(),
                            implements: Vec::new(),
                            visibility: Visibility::Private,
                        });
                    }
                }
                "use" => {
                    if let Some(path) = child.metadata.get("path") {
                        module.imports.push(Import {
                            path: path.clone(),
                            aliases: Vec::new(),
                            is_wildcard: path.contains('*'),
                        });
                    }
                }
                _ => {}
            }
        }

        module
    }

    fn supported_languages(&self) -> Vec<&str> {
        self.languages.iter().map(|s| s.as_str()).collect()
    }
}

fn extract_function_name(line: &str) -> Option<String> {
    let line = line.trim();
    let start = if line.starts_with("pub fn ") {
        7
    } else if line.starts_with("fn ") {
        3
    } else {
        return None;
    };
    let rest = &line[start..];
    let end = rest.find('(').or_else(|| rest.find('<'))?;
    Some(rest[..end].trim().to_string())
}

fn extract_type_name(line: &str) -> Option<String> {
    let line = line.trim();
    let start = if line.starts_with("pub struct ") {
        11
    } else if line.starts_with("struct ") {
        7
    } else if line.starts_with("pub trait ") {
        10
    } else if line.starts_with("trait ") {
        6
    } else {
        return None;
    };
    let rest = &line[start..];
    let end = rest.find(['{', '<', ' ', '('])?;
    Some(rest[..end].trim().to_string())
}

fn extract_impl_name(line: &str) -> Option<String> {
    let line = line.trim();
    if !line.starts_with("impl ") {
        return None;
    }
    let rest = &line[5..];
    let end = rest.find(['{', '<', ' '])?;
    Some(rest[..end].trim().to_string())
}

pub fn extract_all_signatures(source: &str, language: &str) -> Result<ModuleSignature, AstError> {
    let adapter = TreeSitterAdapter::new();
    let ast = adapter.parse_source(source, language)?;
    Ok(adapter.extract_signatures(&ast))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ast_node_creation() {
        let node = AstNode::new("function", 1, 10)
            .with_name("my_func")
            .with_position(0, 50);

        assert_eq!(node.kind, "function");
        assert_eq!(node.name, Some("my_func".to_string()));
        assert_eq!(node.start_line, 1);
        assert_eq!(node.end_line, 10);
    }

    #[test]
    fn test_ast_node_with_children() {
        let mut parent = AstNode::new("module", 0, 100);
        let child = AstNode::new("function", 5, 10).with_name("test");
        parent.add_child(child);

        assert_eq!(parent.children.len(), 1);
        assert_eq!(parent.children[0].name, Some("test".to_string()));
    }

    #[test]
    fn test_tree_sitter_adapter_creation() {
        let adapter = TreeSitterAdapter::new();
        assert!(adapter.supported_languages().contains(&"rust"));
    }

    #[test]
    fn test_parse_rust_source() {
        let adapter = TreeSitterAdapter::new();
        let source = r#"
use std::collections::HashMap;

pub struct MyStruct {
    field: i32,
}

pub fn my_function(x: i32) -> bool {
    x > 0
}

fn private_function() {}
"#;

        let ast = adapter.parse_source(source, "rust").unwrap();
        assert_eq!(ast.kind, "module");
        assert!(!ast.children.is_empty());
    }

    #[test]
    fn test_extract_signatures() {
        let adapter = TreeSitterAdapter::new();
        let source = r#"
use std::io;

pub fn public_func() {}
fn private_func() {}

pub struct MyStruct {}
"#;

        let ast = adapter.parse_source(source, "rust").unwrap();
        let sigs = adapter.extract_signatures(&ast);

        assert_eq!(sigs.functions.len(), 2);
        assert_eq!(sigs.types.len(), 1);
        assert_eq!(sigs.imports.len(), 1);
    }

    #[test]
    fn test_unsupported_language() {
        let adapter = TreeSitterAdapter::new();
        let result = adapter.parse_source("code", "unknown");
        assert!(matches!(result, Err(AstError::UnsupportedLanguage(_))));
    }

    #[test]
    fn test_extract_function_name() {
        assert_eq!(extract_function_name("fn test()"), Some("test".to_string()));
        assert_eq!(
            extract_function_name("pub fn my_func(x: i32)"),
            Some("my_func".to_string())
        );
        assert_eq!(extract_function_name("not a function"), None);
    }

    #[test]
    fn test_extract_type_name() {
        assert_eq!(
            extract_type_name("struct MyStruct {"),
            Some("MyStruct".to_string())
        );
        assert_eq!(
            extract_type_name("pub trait MyTrait {"),
            Some("MyTrait".to_string())
        );
        assert_eq!(extract_type_name("not a type"), None);
    }

    #[test]
    fn test_extract_all_signatures() {
        let source = "pub fn test() {} struct Foo {}";
        let result = extract_all_signatures(source, "rust").unwrap();
        assert!(!result.functions.is_empty() || !result.types.is_empty());
    }
}
