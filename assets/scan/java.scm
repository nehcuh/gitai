;; Java Security Rules for Tree-sitter
;; SQL injection
(method_invocation
  name: (identifier) @method
  arguments: (argument_list
    (string_literal) @query)
  (#match? @method "(?i)(executeQuery|executeUpdate|execute)")
) @sql_injection

;; Hardcoded passwords
(variable_declarator
  name: (identifier) @var_name
  value: (string_literal) @secret
  (#match? @var_name "(?i)(password|secret|key|token)")
) @hardcoded_secret

;; Insecure random
(method_invocation
  object: (identifier) @obj
  name: (identifier) @method
  (#eq? @obj "Math")
  (#eq? @method "random")
) @weak_random

;; Command injection
(method_invocation
  object: (field_access
    object: (identifier) @runtime
    field: (identifier) @builder)
  name: (identifier) @method
  arguments: (argument_list
    (string_literal) @command)
  (#eq? @runtime "Runtime")
  (#eq? @builder "getRuntime")
  (#eq? @method "exec")
) @command_injection

;; Deserialization
(method_invocation
  name: (identifier) @method
  (#match? @method "(?i)(readObject|readUnshared)")
) @deserialization