pretty simple library.

### core words:

##### `msg_key: &str`  -> Message key in a .ftl file.

##### `arg_name: &str` -> arguemnt name in the fluent entries definition.

### core functions:
##### `build_language_0`    -> String replacement
##### `build_language_1`    -> Provude one variable to the Fluent library.
##### `build_language_2`    -> 
##### `build_language_*`    -> Provide multiple variables to the Fluent library.
##### `build_language_fns`  -> Generate `FluentValue`s using functions.
##### `build_language`      -> Input a list of FluentValue objects.
##### `init_lang`           -> Called before all build_language* functions.

### For language negotiation:

- A  **languages directory** and **desired language** will be inputed into the `init_lang` function.
    
- All directories within **languages directory** are treated as individual languages.
    
- Each languages is compared with **desired language** using the `language_matches_score` function.
    
- A score-sorted list is then passed to FluentBundle.
    
- Languages of the same type are sorted alphabetically. For example, given the desired language `en`, the available languages are `en_US` and `en_GB`, the chosen language would be en_GB (due to string order).
    
### Default languages directory: ./i18n/fluent

#

> Example of directory structure:

    - i18n/fluent:
        - en_US
            - lang.ftl
        - pl
            - lang.ftl

