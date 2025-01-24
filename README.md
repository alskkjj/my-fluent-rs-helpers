pretty simple library.

### core words:
    msg_key: &str  -> Message key in a .ftl file.
    arg_name: &str -> arguemnt name in the fluent entries definition.

### core functions:
    build_language_0    -> string replacement
    build_language_1    -> give one variable to fluent library.
    build_language_2    -> 
    build_language_*    -> give * variable to fluent library.
    build_language_fns  -> generate __FluentValue__s by functions.
    build_language      -> input FluentValue list argument.
    init_lang           -> called before all build_language* functions.

### For language negotiation:
    A  __languages directory__ and __desired language__ will be inputed into function `init_lang`.
    All directories in __languages directory__ will be considered as a kind of language.
    All kinds of languages will be compared with desired language by function `language_matches_score`.
    A score-sorted list will be fed to FluentBundle.
    The same *types* languages will be sorted by alphabeta order. For example, the languages are `en_US`, `en_GB` with desried language `en`, the choiced language would be en_GB because of string order.
    
### default languages directory: ./i18n/fluent

An example of directory structure:
    i18n/fluent:
        -- en_US
            -- lang.ftl
        -- pl
            -- lang.ftl

