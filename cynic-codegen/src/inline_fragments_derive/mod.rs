use darling::util::SpannedValue;
use proc_macro2::{Span, TokenStream};

use crate::{load_schema, schema, Ident, TypePath};

pub mod input;

pub use input::InlineFragmentsDeriveInput;

use input::InlineFragmentsDeriveVariant;

pub fn inline_fragments_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    use darling::FromDeriveInput;

    match InlineFragmentsDeriveInput::from_derive_input(ast) {
        Ok(input) => inline_fragments_derive_impl(input),
        Err(e) => Ok(e.write_errors()),
    }
}

pub(crate) fn inline_fragments_derive_impl(
    input: InlineFragmentsDeriveInput,
) -> Result<TokenStream, syn::Error> {
    use quote::{quote, quote_spanned};

    let schema =
        load_schema(&*input.schema_path).map_err(|e| e.to_syn_error(input.schema_path.span()))?;

    if !find_union_type(&input.graphql_type, &schema) {
        return Err(syn::Error::new(
            input.graphql_type.span(),
            format!("Could not find a Union type named {}", &*input.graphql_type),
        ));
    }

    let argument_struct = if let Some(arg_struct) = input.argument_struct {
        let span = arg_struct.span();
        let arg_struct_val: Ident = arg_struct.into();
        let argument_struct = quote_spanned! { span => #arg_struct_val };
        syn::parse2(argument_struct)?
    } else {
        syn::parse2(quote! { () })?
    };

    if let darling::ast::Data::Enum(variants) = &input.data {
        let inline_fragments_impl = InlineFragmentsImpl {
            target_struct: input.ident.clone(),
            type_lock: TypePath::concat(&[
                Ident::new_spanned(&*input.query_module, input.query_module.span()).into(),
                Ident::for_type(&*input.graphql_type).into(),
            ]),
            argument_struct,
            possible_types: possible_types_from_variants(variants)?,
            graphql_type_name: (*input.graphql_type).clone(),
        };

        Ok(quote! { #inline_fragments_impl })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            format!("InlineFragments can only be derived from an enum"),
        ))
    }
}

fn possible_types_from_variants(
    variants: &[SpannedValue<InlineFragmentsDeriveVariant>],
) -> Result<Vec<(syn::Ident, syn::Type)>, syn::Error> {
    let mut result = vec![];
    for variant in variants {
        if variant.fields.style != darling::ast::Style::Tuple || variant.fields.fields.len() != 1 {
            return Err(syn::Error::new(
                variant.span(),
                "InlineFragments derive requires enum variants to have one unnamed field",
            ));
        }
        let field = variant.fields.fields.first().unwrap();
        result.push((variant.ident.clone(), field.ty.clone()));
    }
    Ok(result)
}

struct InlineFragmentsImpl {
    target_struct: syn::Ident,
    type_lock: TypePath,
    argument_struct: syn::Type,
    possible_types: Vec<(syn::Ident, syn::Type)>,
    graphql_type_name: String,
}

impl quote::ToTokens for InlineFragmentsImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;
        let type_lock = &self.type_lock;
        let arguments = &self.argument_struct;
        let internal_types: Vec<_> = self.possible_types.iter().map(|(_, ty)| ty).collect();
        let variants: Vec<_> = self.possible_types.iter().map(|(v, _)| v).collect();
        let graphql_type = proc_macro2::Literal::string(&self.graphql_type_name);

        tokens.append_all(quote! {
            #[automatically_derived]
            impl ::cynic::InlineFragments for #target_struct {
                type TypeLock = #type_lock;
                type Arguments = #arguments;

                fn fragments(context: ::cynic::FragmentContext<'_, Self::Arguments>) ->
                    Vec<(String, ::cynic::SelectionSet<'static, Self, Self::TypeLock>)>
                {
                    use ::cynic::QueryFragment;

                    let args = context.args;

                    let mut rv = vec![];
                    #(
                        rv.push((
                            #internal_types::graphql_type(),
                            #internal_types
                                ::fragment(::cynic::FragmentContext::with_args(args))
                                .map(#target_struct::#variants)
                                .transform_typelock()
                        ));
                    )*
                    rv
                }

                fn graphql_type() -> String {
                    #graphql_type.to_string()
                }
            }
        });
    }
}

fn find_union_type(name: &str, schema: &schema::Document) -> bool {
    for definition in &schema.definitions {
        use graphql_parser::schema::{Definition, TypeDefinition};
        match definition {
            Definition::TypeDefinition(TypeDefinition::Union(union)) => {
                if union.name == name {
                    return true;
                }
            }
            _ => {}
        }
    }
    return false;
}
