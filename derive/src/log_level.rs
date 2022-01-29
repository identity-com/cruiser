use proc_macro2::{Ident, TokenStream};
use proc_macro_error::abort;
use syn::parse::{Parse, ParseStream};

#[derive(Copy, Clone, Debug, Ord, PartialOrd, Eq, PartialEq)]
pub enum LogLevel {
    None = 0,
    Error = 1,
    Warn = 2,
    Info = 3,
    Debug = 4,
    Trace = 5,
}
impl LogLevel {
    pub fn if_level(
        self,
        level: LogLevel,
        tokens: impl FnOnce(LogLevel) -> TokenStream,
    ) -> TokenStream {
        if level <= self {
            tokens(self)
        } else {
            TokenStream::new()
        }
    }
}
impl Parse for LogLevel {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident: Ident = input.parse()?;
        Ok(match ident.to_string().as_str() {
            "none" => Self::None,
            "error" => Self::Error,
            "warn" => Self::Warn,
            "info" => Self::Info,
            "debug" => Self::Debug,
            "trace" => Self::Trace,
            x => abort!(ident, "Unknown log level `{}`", x),
        })
    }
}
impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}
