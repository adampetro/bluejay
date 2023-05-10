use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, spanned::Spanned};

#[proc_macro_derive(AsIter)]
pub fn derive_as_iter(item: TokenStream) -> TokenStream {
    derive_as_iter_internal(parse_macro_input!(item as syn::DeriveInput))
        .unwrap_or_else(syn::Error::into_compile_error)
        .into()
}

fn derive_as_iter_internal(input: syn::DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    if let syn::Data::Struct(parsed_struct) = &input.data {
        match &parsed_struct.fields {
            syn::Fields::Named(named_fields) => {
                derive_for_struct(&input.ident, &input.generics, &named_fields.named)
            }
            syn::Fields::Unnamed(unnamed_fields) => {
                derive_for_struct(&input.ident, &input.generics, &unnamed_fields.unnamed)
            }
            syn::Fields::Unit => Err(syn::Error::new(input.span(), "expected struct with fields")),
        }
    } else {
        Err(syn::Error::new(input.span(), "expected struct"))
    }
}

fn derive_for_struct(
    ident: &syn::Ident,
    generics: &syn::Generics,
    fields: &syn::punctuated::Punctuated<syn::Field, syn::token::Comma>,
) -> syn::Result<proc_macro2::TokenStream> {
    if let [(vec_field_type, vec_field_ident)] = fields
        .into_iter()
        .enumerate()
        .filter_map(|(idx, field)| {
            if let syn::Type::Path(p) = &field.ty {
                (p.path.segments.len() == 1)
                    .then_some(p.path.segments.first().unwrap())
                    .and_then(|segment| {
                        if segment.ident == "Vec" {
                            if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                                (args.args.len() == 1)
                                    .then_some(args.args.first().unwrap())
                                    .and_then(|generic_arg| {
                                        if let syn::GenericArgument::Type(ty) = generic_arg {
                                            Some(ty)
                                        } else {
                                            None
                                        }
                                    })
                            } else {
                                None
                            }
                        } else {
                            None
                        }
                    })
                    .map(|ty| {
                        (
                            ty,
                            field.ident.as_ref().map_or_else(
                                || syn::Index::from(idx).to_token_stream(),
                                |ident| ident.to_token_stream(),
                            ),
                        )
                    })
            } else {
                None
            }
        })
        .collect::<Vec<_>>()
        .as_slice()
    {
        let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

        let iterator_where_clause = if let Some(lifetime) = generics.lifetimes().next() {
            let lifetime = &lifetime.lifetime;
            quote! {
                where #lifetime: '_a
            }
        } else {
            quote! {}
        };

        Ok(quote! {
            impl #impl_generics bluejay_core::AsIter for #ident #ty_generics #where_clause {
                type Item = #vec_field_type;
                type Iterator<'_a> = std::slice::Iter<'_a, Self::Item> #iterator_where_clause;

                fn iter(&self) -> Self::Iterator<'_> {
                    self.#vec_field_ident.iter()
                }

                fn is_empty(&self) -> bool {
                    self.#vec_field_ident.is_empty()
                }

                fn len(&self) -> usize {
                    self.#vec_field_ident.len()
                }
            }
        })
    } else {
        Err(syn::Error::new(
            fields.span(),
            "expected a single field of vector type",
        ))
    }
}
