;; JavaScript/TypeScript Security Rules for Tree-sitter
;; XSS vulnerabilities
(call_expression
  function: (member_expression
    object: (identifier) @obj
    property: (property_identifier) @prop)
  arguments: (arguments
    (string) @content)
  (#eq? @obj "document")
  (#eq? @prop "write")
) @xss

(assignment_expression
  left: (member_expression
    object: (identifier) @obj
    property: (property_identifier) @prop)
  right: (string) @content
  (#eq? @obj "document")
  (#eq? @prop "innerHTML")
) @xss

;; Hardcoded secrets
(variable_declarator
  name: (identifier) @var_name
  value: (string) @secret
  (#match? @var_name "(?i)(password|secret|key|token|api_key)")
  (#match? @secret "^[\"'][A-Za-z0-9+/=]{20,}[\"']$")
) @hardcoded_secret

;; SQL injection
(call_expression
  function: (member_expression
    property: (property_identifier) @method)
  arguments: (arguments
    (template_literal) @query)
  (#match? @method "(?i)(query|execute|exec)")
) @sql_injection

;; Insecure random
(call_expression
  function: (member_expression
    object: (identifier) @obj
    property: (property_identifier) @prop)
  (#eq? @obj "Math")
  (#eq? @prop "random")
) @weak_random

;; eval usage
(call_expression
  function: (identifier) @func
  (#eq? @func "eval")
) @eval_usage