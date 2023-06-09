use quote::{ToTokens, TokenStreamExt};
use syn::parse::Parse;

mod kw {
    syn::custom_keyword!(borrow);
}

pub(crate) enum DocumentInput {
    Path(syn::LitStr),
    Dsl {
        bracket: syn::token::Bracket,
        contents: proc_macro2::TokenStream,
    },
}

impl Parse for DocumentInput {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        if input.peek(syn::token::Bracket) {
            let contents;
            Ok(Self::Dsl {
                bracket: syn::bracketed!(contents in input),
                contents: contents.parse()?,
            })
        } else {
            input.parse().map(Self::Path)
        }
    }
}

impl ToTokens for DocumentInput {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        match self {
            Self::Path(path) => tokens.append(path.token()),
            DocumentInput::Dsl { bracket, contents } => {
                bracket.surround(tokens, |tokens| tokens.extend(contents.clone()))
            }
        }
    }
}

impl DocumentInput {
    pub(crate) fn read_to_string(&self) -> syn::Result<String> {
        match self {
            Self::Path(path) => Self::read_file(path),
            Self::Dsl { contents, .. } => Ok(contents.to_string()),
        }
    }

    fn read_file(filename: &syn::LitStr) -> syn::Result<String> {
        let cargo_manifest_dir =
            std::env::var("CARGO_MANIFEST_DIR").map_err(|_| syn::Error::new(filename.span(), "Environment variable CARGO_MANIFEST_DIR was not set but is needed to resolve relative paths"))?;
        let base_path = std::path::PathBuf::from(cargo_manifest_dir);

        let file_path = base_path.join(filename.value());

        std::fs::read_to_string(file_path)
            .map_err(|err| syn::Error::new(filename.span(), format!("{}", err)))
    }
}

pub(crate) struct Input {
    pub(crate) schema: DocumentInput,
    pub(crate) borrow: bool,
}

impl Parse for Input {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let schema: DocumentInput = input.parse()?;

        let mut borrow: Option<syn::LitBool> = None;

        while !input.is_empty() {
            input.parse::<syn::Token![,]>()?;
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::borrow) {
                Self::parse_key_value(&input, &mut borrow)?;
            } else {
                return Err(lookahead.error());
            }
        }

        let borrow = borrow.map_or(false, |borrow| borrow.value);

        Ok(Self { schema, borrow })
    }
}

impl Input {
    fn parse_key_value<V: syn::parse::Parse>(
        input: &syn::parse::ParseStream,
        value: &mut Option<V>,
    ) -> syn::Result<()> {
        let key: syn::Ident = input.parse()?;

        if value.is_some() {
            return Err(syn::Error::new(
                key.span(),
                format!("Duplicate entry for `{}`", key),
            ));
        }

        input.parse::<syn::Token![=]>()?;
        *value = Some(input.parse()?);
        Ok(())
    }
}
