//! macro-magic integration tests.
use ::macro_magic::*;

__declare_features!(
    default_macro: default_macro_parse;

    @default: section;

    priority {
        attr: [(priority = $priority_value:tt) => ($priority_value)];
    };
    section {
        attr: [(section = $($section_path:tt)*) => (($($section_path)*))];
        example: "[section = ] ::path::to::SECTION";
        validate: [(($section_path:path))];
    };
);

__test!(__extract_meta[default_macro_parse]:
    (default_macro) => (priority = () default , section = () default ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(priority = 1)) => (priority = 1 value , section = () default ,));

// Single path segment
__test!(__extract_meta[default_macro_parse]:
    (default_macro(SECTION_NAME)) => (priority = () default , section = (SECTION_NAME) value ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(section = SECTION_NAME)) => (priority = () default , section = (SECTION_NAME) value ,));

// Extern prelude
__test!(__extract_meta[default_macro_parse]:
    (default_macro(::SECTION_NAME)) => (priority = () default , section = (::SECTION_NAME) value ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(section = ::SECTION_NAME)) => (priority = () default , section = (::SECTION_NAME) value ,));

// Super path
__test!(__extract_meta[default_macro_parse]:
    (default_macro(super::SECTION_NAME)) => (priority = () default , section = (super::SECTION_NAME) value ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(section = super::SECTION_NAME)) => (priority = () default , section = (super:: SECTION_NAME) value ,));

// Crate path
__test!(__extract_meta[default_macro_parse]:
    (default_macro(crate::SECTION_NAME)) => (priority = () default , section = (crate::SECTION_NAME) value ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(section = crate::SECTION_NAME)) => (priority = () default , section = (crate:: SECTION_NAME) value ,));

// Self path
__test!(__extract_meta[default_macro_parse]:
    (default_macro(self::SECTION_NAME)) => (priority = () default , section = (self::SECTION_NAME) value ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(section = self::SECTION_NAME)) => (priority = () default , section = (self:: SECTION_NAME) value ,));

// Multiple path segments
__test!(__extract_meta[default_macro_parse]:
    (default_macro(::path::to::SECTION_NAME)) => (priority = () default , section = (::path::to::SECTION_NAME) value ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(section = ::path::to::SECTION_NAME)) => (priority = () default , section = (::path::to::SECTION_NAME) value ,));

// Multiple path segments with super
__test!(__extract_meta[default_macro_parse]:
    (default_macro(super::path::to::SECTION_NAME)) => (priority = () default , section = (super::path::to::SECTION_NAME) value ,));
__test!(__extract_meta[default_macro_parse]:
    (default_macro(section = super::path::to::SECTION_NAME)) => (priority = () default , section = (super::path::to::SECTION_NAME) value ,));
