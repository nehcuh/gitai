;; Function definitions
(function_item
  name: (identifier) @function.name) @definition.function

;; Struct definitions
(struct_item
  name: (type_identifier) @type.name) @definition.struct

;; Enum definitions
(enum_item
  name: (type_identifier) @type.name) @definition.enum

;; Trait definitions
(trait_item
  name: (type_identifier) @type.name) @definition.trait

;; Impl blocks
(impl_item
  type: (type_identifier) @type.name) @definition.impl

;; Module definitions
(mod_item
  name: (identifier) @module.name) @definition.module

;; Public constants
(const_item
  name: (identifier) @constant.name) @definition.constant

;; Public static variables
(static_item
  name: (identifier) @static.name) @definition.static

;; Type aliases
(type_item
  name: (type_identifier) @type.name) @definition.type
