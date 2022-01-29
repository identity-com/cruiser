use easy_proc::ArgumentList;
use proc_macro2::Span;
use syn::{AttrStyle, Attribute, Ident, LitInt, LitStr, Path};

#[derive(ArgumentList)]
pub struct Cool {
    /// The ident of the whole attribute, not required and can only be one
    #[argument(attr_ident)]
    pub attr_ident: Ident,
    /// [`true`] if arg is present
    #[argument(presence)]
    pub boolean_value: bool,
    /// Required argument of form `count = 10`
    pub count: LitInt,
    /// Optional argument, if present of form `size = 3`
    pub size: Option<LitInt>,
    /// Required argument with option type
    #[argument(raw_type)]
    pub health: Option<LitInt>,
    /// Custom parsing, including equals. Uses parse function.
    /// Ex: `custom_parse cool`
    #[argument(custom)]
    pub custom_parse: Ident,
    /// Optional with default value. Also implies `raw_type`
    #[argument(default = Ident::new("default", Span::call_site()))]
    pub default: Ident,
    /// Many, 0 or more
    pub many: Vec<LitStr>,
}
