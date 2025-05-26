; Function definitions
(function_item
  name: (identifier) @function.name) @function

; Struct definitions
(struct_item
  name: (type_identifier) @struct.name) @struct

; Enum definitions
(enum_item
  name: (type_identifier) @enum.name) @enum

; Trait definitions
(trait_item
  name: (type_identifier) @trait.name) @trait

; Impl blocks
(impl_item
  type: (type_identifier) @impl.name) @impl

; Module definitions
(mod_item
  name: (identifier) @module.name) @module

; Use declarations
(use_declaration) @use

; Constants
(const_item
  name: (identifier) @constant.name) @constant

; Static variables
(static_item
  name: (identifier) @static.name) @static

; Let bindings
(let_declaration
  pattern: (identifier) @variable.name) @variable

; Parameters
(parameter
  pattern: (identifier) @parameter.name) @parameter

; Field declarations
(field_declaration
  name: (field_identifier) @field.name) @field

; Macro calls
(macro_invocation
  macro: (identifier) @macro.name) @macro

; Type aliases
(type_item
  name: (type_identifier) @type_alias.name) @type_alias

; Function calls
(call_expression
  function: (identifier) @function_call.name) @function_call

; Method calls
(call_expression
  function: (field_expression
    field: (field_identifier) @method_call.name)) @method_call

; Identifiers
(identifier) @identifier

; Type identifiers
(type_identifier) @type_identifier

; Field identifiers
(field_identifier) @field_identifier