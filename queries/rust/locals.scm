; locals.scm - temporary placeholder for Rust locals queries

; Function definitions create scope
(function_item
  name: (identifier) @local.definition.function)

; Parameter definitions
(parameters
  (parameter
    pattern: (identifier) @local.definition.parameter))

; Variable bindings
(let_declaration
  pattern: (identifier) @local.definition.variable)

; References
(identifier) @local.reference