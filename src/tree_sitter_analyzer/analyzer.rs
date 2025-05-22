use std::{path::PathBuf, time::SystemTime};

use std::collections::HashMap;
use tree_sitter::Tree;

use tree_sitter::{Language, Query};

use crate::config::TreeSitterConfig;
use crate::errors::TreeSitterError;

use super::core::{
    get_tree_sitter_c, get_tree_sitter_cpp, get_tree_sitter_go, get_tree_sitter_java,
    get_tree_sitter_js, get_tree_sitter_python, get_tree_sitter_rust,
};

#[derive(Debug)]
pub struct TreeSitterAnalyzer {
    pub config: TreeSitterConfig,
    pub project_root: PathBuf,
    languages: HashMap<String, Language>,
    file_asts: HashMap<PathBuf, FileAst>, // Cache for parsed file ASTs
    queries: HashMap<String, Query>,      // Cache for compiled queries
                                          // parser_cache: HashMap<String, Parser>, // If we want to reuse parsers
}

impl TreeSitterAnalyzer {
    pub fn new(config: TreeSitterConfig) -> Result<Self, TreeSitterError> {
        let mut analyzer = Self {
            config,
            project_root: PathBuf::new(), // Set later with set_project_root
            languages: HashMap::new(),
            file_asts: HashMap::new(),
            queries: HashMap::new(),
        };

        analyzer.initialize_languages()?;
        analyzer.initialize_queries()?;
        Ok(analyzer)
    }

    fn initialize_languages(&mut self) -> Result<(), TreeSitterError> {
        // Load languages based on config or defaults
        // Example for Rust and Java
        if self.config.languages.contains(&"rust".to_string()) {
            self.languages
                .insert("rust".to_string(), get_tree_sitter_rust());
        }
        if self.config.languages.contains(&"java".to_string()) {
            self.languages
                .insert("java".to_string(), get_tree_sitter_java());
        }

        // Add Python and Go based on configuration
        if self.config.languages.contains(&"python".to_string()) {
            self.languages
                .insert("python".to_string(), get_tree_sitter_python());
        }
        if self.config.languages.contains(&"go".to_string()) {
            self.languages
                .insert("go".to_string(), get_tree_sitter_go());
        }
        if self.config.languages.contains(&"js".to_string()) {
            self.languages
                .insert("js".to_string(), get_tree_sitter_js());
        }
        if self.config.languages.contains(&"c".to_string()) {
            self.languages.insert("c".to_string(), get_tree_sitter_c());
        }
        if self.config.languages.contains(&"cpp".to_string()) {
            self.languages
                .insert("cpp".to_string(), get_tree_sitter_cpp());
        }
        // Potentially load tree_sitter_javascript if configured
        Ok(())
    }

    fn initialize_queries(&mut self) -> Result<(), TreeSitterError> {
        if self.languages.contains_key("rust") {
            let rust_query_pattern = self.get_rust_query_pattern();
            let rust_lang = self.languages.get("rust").unwrap(); // Safe due to check
            let query = Query::new(rust_lang, &rust_query_pattern)
                .map_err(|e| TreeSitterError::QueryError(format!("Rust query error: {}", e)))?;
            self.queries.insert("rust".to_string(), query);
        }
        if self.languages.contains_key("java") {
            let java_query_pattern = self.get_java_query_pattern();
            let java_lang = self.languages.get("java").unwrap(); // Safe due to check
            let query = Query::new(java_lang, &java_query_pattern)
                .map_err(|e| TreeSitterError::QueryError(format!("Rust query error: {}", e)))?;
            self.queries.insert("rust".to_string(), query);
        }
        Ok(())
    }

    pub(crate) fn get_javascript_query_pattern(&self) -> String {
        r#"
        ; ▒▒▒ 基本声明 ▒▒▒
        ; 函数声明
        (function_declaration
          name: (identifier) @function.name
        ) @function.declaration

        ; 箭头函数
        (arrow_function) @function.arrow

        ; 函数表达式
        (function
          name: (identifier)? @function.name
        ) @function.expression

        ; 变量声明（支持 let/const/var）
        (variable_declarator
          name: (_) @variable.name
          value: (_)? @variable.value
        ) @variable.declaration

        ; ▒▒▒ 类相关 ▒▒▒
        (class_declaration
          name: (identifier) @class.name
          body: (class_body) @class.body
        ) @class.declaration

        (method_definition
          name: (property_identifier) @method.name
        ) @method.declaration

        (public_field_definition
          name: (property_identifier) @property.name
          value: (_)? @property.value
        ) @property.declaration

        ; ▒▒▒ 模块系统 ▒▒▒
        ; 导入语句
        (import_statement
          source: (string) @import.source
        ) @import.declaration

        ; 导出语句
        (export_statement
          declaration: (_)? @export.declaration
        ) @export

        ; ▒▒▒ JSX 支持 ▒▒▒
        (jsx_element
          open_tag: (jsx_opening_element
            name: (_) @jsx.tag
          )
          close_tag: (jsx_closing_element
            name: (_) @jsx.tag
          )
        ) @jsx.element

        (jsx_self_closing_element
          name: (_) @jsx.tag
        ) @jsx.self-closing

        (jsx_attribute
          name: (jsx_attribute_name) @jsx.attr.name
          value: (_)? @jsx.attr.value
        ) @jsx.attribute

        ; ▒▒▒ 现代语法 ▒▒▒
        ; 解构赋值
        (variable_declarator
          pattern: (object_pattern) @destruct.object
        ) @destruct.declaration

        (variable_declarator
          pattern: (array_pattern) @destruct.array
        ) @destruct.declaration

        ; 模板字符串
        (template_string) @string.template
        (template_substitution
          (_) @template.expression
        ) @template.substitution

        ; ▒▒▒ 异步/生成器 ▒▒▒
        (generator_function
          name: (identifier)? @function.name
        ) @function.generator

        (generator_function_declaration
          name: (identifier) @function.name
        ) @function.generator.declaration

        (await_expression) @expression.await

        ; ▒▒▒ 装饰器 ▒▒▒
        (decorator
          (call_expression
            function: (_) @decorator.name
          )
        ) @decorator

        ; ▒▒▒ 类型注释 (TypeScript/Flow) ▒▒▒
        (type_alias_declaration
          name: (type_identifier) @type.name
        ) @type.declaration

        (type_annotation
          (type) @type.annotation
        ) @annotation.type

        ; ▒▒▒ 注释 ▒▒▒
        (comment) @comment

        ; ▒▒▒ 高级模式 ▒▒▒
        ; Promise链调用
        (call_expression
          function: (member_expression
            object: (call_expression) @promise.chain
            property: (property_identifier) @promise.method
          )
        ) @promise.chain-call

        ; 三元表达式
        (ternary_expression) @expression.ternary

        ; 对象属性简写
        (pair
          key: (property_identifier) @object.key
          value: (identifier) @object.value-shorthand
          (#eq? @object.key @object.value-shorthand)
        ) @object.shorthand-property
        "#
        .to_string()
    }

    // Method to get Rust query pattern (moved from the original monolithic file)
    pub(crate) fn get_rust_query_pattern(&self) -> String {
        r#"
        r#"
        ; 函数定义
        (function_item
          name: (identifier) @function.name
        ) @function.declaration

        ; 结构体定义
        (struct_item
          name: (type_identifier) @struct.name
        ) @struct.declaration

        ; 枚举定义
        (enum_item
          name: (type_identifier) @enum.name
        ) @enum.declaration

        ; 特性定义
        (trait_item
          name: (type_identifier) @trait.name
        ) @trait.declaration

        ; 实现块（已增强）
        (impl_item
          trait: (type_identifier)? @impl.trait
          type: (_) @impl.type
        ) @impl.declaration

        ; 模块定义（已增强）
        (mod_item
          name: (identifier) @module.name
          body: (declaration_list)? @module.body
        ) @module.declaration

        ; 常量定义（已增强）
        (const_item
          name: (identifier) @const.name
          value: (_) @const.value
        ) @const.declaration

        ; 静态变量定义（已增强）
        (static_item
          name: (identifier) @static.name
          value: (_) @static.value
        ) @static.declaration

        ; 类型别名
        (type_item
          name: (type_identifier) @type_alias.name
        ) @type_alias.declaration

        ; 宏定义
        (macro_definition
          name: (identifier) @macro.name
        ) @macro.declaration

        ; 使用声明
        (use_declaration) @use.declaration

        ; 属性
        (attribute_item) @attribute
        "#
        .to_string()
    }

    // Method to get Java query pattern (moved from the original monolithic file)
    pub(crate) fn get_java_query_pattern(&self) -> String {
        r#"
        ; 包声明
        (package_declaration
          (scoped_identifier) @package.name
        ) @package.declaration

        ; 导入声明
        (import_declaration
          (scoped_identifier) @import.name
        ) @import.declaration

        ; 类定义
        (class_declaration
          name: (identifier) @class.name
          type_parameters: (type_parameters)? @class.generics
        ) @class.declaration

        ; 接口定义
        (interface_declaration
          name: (identifier) @interface.name
          type_parameters: (type_parameters)? @interface.generics
        ) @interface.declaration

        ; 枚举定义
        (enum_declaration
          name: (identifier) @enum.name
        ) @enum.declaration

        ; 注解接口定义
        (annotation_type_declaration
          name: (identifier) @annotation.name
        ) @annotation.declaration

        ; 方法定义
        (method_declaration
          name: (identifier) @method.name
          type_parameters: (type_parameters)? @method.generics
          return_type: (type) @method.return_type
          parameters: (formal_parameters) @method.parameters
        ) @method.declaration

        ; 构造函数
        (constructor_declaration
          name: (identifier) @constructor.name
          parameters: (formal_parameters) @constructor.parameters
        ) @constructor.declaration

        ; 字段定义
        (field_declaration
          (variable_declarator
            name: (identifier) @field.name
            value: (_)? @field.value
          ) @field.declaration
        )

        ; 注解使用
        (marker_annotation
          name: (identifier) @annotation.name
        ) @annotation

        (annotation
          name: (identifier) @annotation.name
          arguments: (annotation_argument_list)? @annotation.arguments
        ) @annotation

        ; 局部变量定义
        (local_variable_declaration
          (variable_declarator
            name: (identifier) @variable.name
            value: (_)? @variable.value
          ) @variable.declaration
        )

        ; try-with-resources
        (try_with_resources_statement
          (resource_specification
            (resource) @resource.declaration
          )
        ) @try.resources

        ; Lambda 表达式
        (lambda_expression) @lambda

        ; 泛型类型参数
        (type_parameter
          name: (identifier) @type.param.name
          bounds: (type_bound)? @type.param.bounds
        ) @type.param.declaration

        ; 静态初始化块
        (static_initializer) @static.block

        ; 实例初始化块
        (instance_initializer) @instance.block

        ; 注释捕获
        (line_comment) @comment.line
        (block_comment) @comment.block
        "#
        .to_string()
    }

    pub(crate) fn get_go_query_pattern(&self) -> String {
        r#"
        ; 包声明
        (package_clause
          (package_identifier) @package.name
        ) @package.declaration

        ; 导入声明
        (import_declaration
          (import_spec
            path: (interpreted_string_literal) @import.path
            name: (package_identifier)? @import.alias
          ) @import.spec
        ) @import.declaration

        ; 函数定义
        (function_declaration
          name: (identifier) @function.name
          parameters: (parameter_list) @function.parameters
          result: (type)? @function.return_type
        ) @function.declaration

        ; 方法定义（接收器）
        (method_declaration
          receiver: (parameter_list) @method.receiver
          name: (identifier) @method.name
          parameters: (parameter_list) @method.parameters
          result: (type)? @method.return_type
        ) @method.declaration

        ; 结构体定义
        (type_declaration
          (type_spec
            name: (type_identifier) @struct.name
            type: (struct_type
              (field_declaration_list
                (field_declaration
                  name: (field_identifier) @struct.field.name
                  type: (type) @struct.field.type
                  tag: (raw_string_literal)? @struct.field.tag
                ) @struct.field
              )
            ) @struct.body
          )
        ) @struct.declaration

        ; 接口定义
        (type_declaration
          (type_spec
            name: (type_identifier) @interface.name
            type: (interface_type
              (method_spec
                name: (field_identifier) @interface.method.name
                parameters: (parameter_list)? @interface.method.parameters
                result: (type)? @interface.method.return_type
              ) @interface.method
            ) @interface.body
          )
        ) @interface.declaration

        ; 变量声明
        (var_declaration
          (var_spec
            name: (identifier) @variable.name
            type: (type)? @variable.type
            value: (expression_list)? @variable.value
          ) @variable.declaration
        )

        ; 短变量声明 (:=)
        (short_var_declaration
          left: (expression_list
            (identifier) @variable.name
          )
          right: (expression_list) @variable.value
        ) @short_var.declaration

        ; 常量声明
        (const_declaration
          (const_spec
            name: (identifier) @constant.name
            value: (expression_list)? @constant.value
          ) @constant.declaration
        )

        ; 类型别名
        (type_declaration
          (type_spec
            name: (type_identifier) @type_alias.name
            type: (type) @type_alias.target
          )
        ) @type_alias.declaration

        ; 通道操作
        (send_statement
          channel: (expression) @channel.name
          value: (expression) @channel.value
        ) @channel.operation

        (receive_operation
          left: (expression_list)? @channel.receiver
          right: (expression) @channel.source
        ) @channel.operation

        ; Goroutine 和 defer
        (go_statement
          (call_expression) @goroutine.call
        ) @goroutine

        (defer_statement
          (call_expression) @defer.call
        ) @defer

        ; Select 语句
        (select_statement
          (communication_case
            (send_statement) @select.send
            (receive_statement) @select.receive
          ) @select.case
        ) @select.block

        ; 错误处理模式
        (if_statement
          condition: (binary_expression
            left: (identifier) @error.variable
            operator: "!="
            right: (nil)
          )
          consequence: (block
            (return_statement) @error.return
          )
        ) @error.handling

        ; 注释捕获
        (comment) @comment
        "#
        .to_string()
    }

    pub(crate) fn get_python_query_pattern(&self) -> String {
        r#"
        ; 模块级定义
        ;; 函数定义（含异步函数）
        (function_definition
          name: (identifier) @function.name
          parameters: (parameters) @function.parameters
          return_type: (type)? @function.return_type
        ) @function.declaration

        (decorated_definition
          decorator: (decorator)+ @function.decorator
          definition: (function_definition) @function.declaration
        )

        ;; 类定义
        (class_definition
          name: (identifier) @class.name
          superclasses: (argument_list)? @class.parents
        ) @class.declaration

        ;; 变量赋值（捕获多变量赋值）
        (assignment
          left: (identifier) @variable.name
          right: (_) @variable.value
        ) @variable.declaration

        (assignment
          left: (tuple
            (identifier)* @variable.name
          )
          right: (tuple) @variable.value
        ) @variable.declaration

        ; 控制结构
        ;; 条件语句
        (if_statement) @control.if
        (elif_clause) @control.elif
        (else_clause) @control.else

        ;; 循环结构
        (for_statement) @control.for
        (while_statement) @control.while
        (break_statement) @control.break
        (continue_statement) @control.continue

        ; 异常处理
        (try_statement) @exception.try
        (except_clause
          (identifier)? @exception.type
          (identifier)? @exception.name
        ) @exception.except
        (finally_clause) @exception.finally
        (raise_statement) @exception.raise

        ; 导入系统
        (import_statement
          name: (dotted_name) @import.module
        ) @import.declaration

        (import_from_statement
          module: (dotted_name)? @import.module
          name: (dotted_name) @import.name
        ) @import.from

        (aliased_import
          name: (dotted_name) @import.name
          alias: (identifier) @import.alias
        ) @import.alias

        ; 类型注解
        (type_alias
          name: (identifier) @type.name
          (type) @type.definition
        ) @type.declaration

        (typed_parameter
          name: (identifier) @param.name
          type: (type)? @param.type
        )

        (function_definition
          return_type: (type)? @function.return_type
        )

        ; 特殊语法
        ;; 上下文管理器
        (with_statement
          (with_clause
            (with_item
              context: (_) @context.manager
              alias: (identifier)? @context.alias
            )+
          )
        ) @control.with

        ;; 生成器表达式
        (generator_expression) @comprehension.generator
        (list_comprehension) @comprehension.list
        (dictionary_comprehension) @comprehension.dict

        ;; Lambda 表达式
        (lambda) @expression.lambda

        ;; 装饰器
        (decorator
          (identifier) @decorator.name
        ) @decorator

        ; 文档和注释
        (comment) @comment.line
        (string
          (string_content) @docstring.content
          (#match? @docstring.content "^(\"\"\"|''')")
        ) @docstring

        ; 特殊方法（如 __init__）
        (function_definition
          name: (identifier) @method.name
          (#match? @method.name "^__[a-z]+__$")
        ) @method.magic

        ; 异步相关
        (async_statement) @control.async
        (async_for_statement) @control.async_for
        (async_with_statement) @control.async_with
        "#
        .to_string()
    }
}

// Defines the type of change in a Git diff
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    #[allow(dead_code)]
    Copied,
    #[allow(dead_code)]
    TypeChanged,
}

// Represents a hunk range in git diff format (@@ -a,b +c,d @@)
#[derive(Debug, Clone)]
pub struct HunkRange {
    pub start: usize,
    pub count: usize,
}

// Represents a single hunk in a Git diff
#[derive(Debug, Clone)]
pub struct DiffHunk {
    #[allow(dead_code)]
    pub old_range: HunkRange,
    pub new_range: HunkRange,
    #[allow(dead_code)]
    pub lines: Vec<String>,
}

// Represents a changed file in a Git diff
#[derive(Debug, Clone)]
pub struct ChangedFile {
    pub path: PathBuf,
    pub change_type: ChangeType,
    pub hunks: Vec<DiffHunk>,
    pub file_mode_change: Option<String>,
}

// 文件AST结构
// 这个结构体代表一个文件的语法分析树(AST)
// 使用tree-sitter提供的实际Tree类型
#[derive(Debug, Clone)]
pub struct FileAst {
    /// 文件路径
    pub path: PathBuf,
    /// tree-sitter解析树
    pub tree: Tree,
    /// 源代码
    pub source: String,
    /// 内容哈希值
    pub content_hash: String,
    /// 最后解析时间
    #[allow(dead_code)]
    pub last_parsed: SystemTime,
    /// 语言标识
    pub language_id: String,
}
