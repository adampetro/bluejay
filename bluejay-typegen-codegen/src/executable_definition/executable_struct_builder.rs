use crate::executable_definition::{ExecutableStruct, ExecutableTypeBuilder};
use crate::{
    attributes::doc_string,
    names::{module_ident, type_ident},
    CodeGenerator,
};
use std::ops::Not;
use syn::parse_quote;

pub(crate) struct ExecutableStructBuilder<'a, C: CodeGenerator> {
    executable_struct: &'a ExecutableStruct<'a>,
    code_generator: &'a C,
}

impl<'a, C: CodeGenerator> ExecutableStructBuilder<'a, C> {
    pub(crate) fn build(
        executable_struct: &'a ExecutableStruct<'a>,
        code_generator: &'a C,
    ) -> Vec<syn::Item> {
        let instance = Self {
            executable_struct,
            code_generator,
        };

        let name_ident = instance.name_ident();
        let attributes = instance.attributes();
        let fields = code_generator.fields_for_executable_struct(executable_struct);
        let lifetime = instance.lifetime();

        let mut items: Vec<syn::Item> = vec![parse_quote! {
            #(#attributes)*
            pub struct #name_ident #lifetime #fields
        }];

        items.extend(
            instance
                .code_generator
                .additional_impls_for_executable_struct(instance.executable_struct)
                .into_iter()
                .map(Into::into),
        );

        items.extend(instance.nested_module());

        items
    }

    fn name_ident(&self) -> syn::Ident {
        type_ident(self.executable_struct.parent_name())
    }

    fn attributes(&self) -> Vec<syn::Attribute> {
        let mut attributes = Vec::new();
        attributes.extend(self.executable_struct.description().map(doc_string));
        attributes.extend(
            self.code_generator
                .attributes_for_executable_struct(self.executable_struct),
        );

        attributes
    }

    fn nested_module(&self) -> Option<syn::Item> {
        let nested = self
            .executable_struct
            .fields()
            .iter()
            .flat_map(|field| {
                ExecutableTypeBuilder::build(field.r#type().base(), self.code_generator)
            })
            .collect::<Vec<syn::Item>>();

        nested.is_empty().not().then(|| {
            let module_ident = module_ident(self.executable_struct.parent_name());
            parse_quote! {
                pub mod #module_ident {
                    #(#nested)*
                }
            }
        })
    }

    fn lifetime(&self) -> Option<syn::Generics> {
        self.executable_struct
            .borrows()
            .then(|| parse_quote! { <'a> })
    }
}
