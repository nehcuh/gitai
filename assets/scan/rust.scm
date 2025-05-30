;; Rust Security Rules for Tree-sitter
;; Unsafe blocks
(unsafe_block) @unsafe_usage

;; Hardcoded secrets
(let_declaration
  pattern: (identifier) @var_name
  value: (string_literal) @secret
  (#match? @var_name "(?i)(password|secret|key|token|api_key)")
  (#match? @secret "^[\"'][A-Za-z0-9+/=]{20,}[\"']$")
) @hardcoded_secret

;; Command execution
(call_expression
  function: (scoped_identifier
    path: (identifier) @module
    name: (identifier) @func)
  arguments: (arguments
    (string_literal) @command)
  (#eq? @module "Command")
  (#eq? @func "new")
) @command_execution

;; Unwrap usage (potential panic)
(call_expression
  function: (field_expression
    field: (field_identifier) @method)
  (#eq? @method "unwrap")
) @unwrap_usage

;; expect usage (potential panic)
(call_expression
  function: (field_expression
    field: (field_identifier) @method)
  (#eq? @method "expect")
) @expect_usage

;; Raw pointer dereference
(unary_expression
  operator: "*"
  argument: (identifier) @ptr
) @raw_pointer_deref