;; Python Security Rules for Tree-sitter
;; SQL injection
(call
  function: (attribute
    object: (identifier) @obj
    attribute: (identifier) @method)
  arguments: (argument_list
    (string) @query)
  (#match? @method "(?i)(execute|executemany)")
) @sql_injection

;; Hardcoded secrets
(assignment
  left: (identifier) @var_name
  right: (string) @secret
  (#match? @var_name "(?i)(password|secret|key|token|api_key)")
  (#match? @secret "^[\"'][A-Za-z0-9+/=]{20,}[\"']$")
) @hardcoded_secret

;; eval usage
(call
  function: (identifier) @func
  (#eq? @func "eval")
) @eval_usage

;; exec usage
(call
  function: (identifier) @func
  (#eq? @func "exec")
) @exec_usage

;; Command injection
(call
  function: (attribute
    object: (identifier) @module
    attribute: (identifier) @func)
  arguments: (argument_list
    (string) @command)
  (#eq? @module "os")
  (#match? @func "(?i)(system|popen)")
) @command_injection

;; Pickle deserialization
(call
  function: (attribute
    object: (identifier) @module
    attribute: (identifier) @func)
  (#eq? @module "pickle")
  (#match? @func "(?i)(load|loads)")
) @pickle_deserialization

;; Insecure random
(call
  function: (attribute
    object: (identifier) @module
    attribute: (identifier) @func)
  (#eq? @module "random")
  (#match? @func "(?i)(random|randint|choice)")
) @weak_random