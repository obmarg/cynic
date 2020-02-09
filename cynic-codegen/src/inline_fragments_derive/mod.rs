use proc_macro2::{Span, TokenStream};

use crate::{Ident, TypePath};

mod attributes;
use attributes::InlineFragmentsDeriveAttributes;

pub fn inline_fragments_derive(ast: &syn::DeriveInput) -> Result<TokenStream, syn::Error> {
    inline_fragments_derive_impl(ast, attributes::parse(&ast.attrs)?)
}

fn inline_fragments_derive_impl(
    ast: &syn::DeriveInput,
    attributes: InlineFragmentsDeriveAttributes,
) -> Result<TokenStream, syn::Error> {
    use quote::{quote, quote_spanned};

    let schema = std::fs::read_to_string(&attributes.schema_path.value).map_err(|e| {
        syn::Error::new(
            attributes.schema_path.span,
            format!("Could not load schema file: {}", e),
        )
    })?;

    let schema = graphql_parser::schema::parse_schema(&schema).map_err(|e| {
        syn::Error::new(
            attributes.schema_path.span,
            format!("Could not parse schema file: {}", e),
        )
    })?;

    if !find_union_type(&attributes.graphql_type.value, &schema) {
        return Err(syn::Error::new(
            attributes.graphql_type.span,
            format!(
                "Could not find a Union type named {}",
                attributes.graphql_type.value
            ),
        ));
    }

    let argument_struct = if let Some(arg_struct) = attributes.argument_struct {
        let arg_struct_val = Ident::new(&arg_struct.value);
        let argument_struct = quote_spanned! { arg_struct.span => #arg_struct_val };
        syn::parse2(argument_struct)?
    } else {
        syn::parse2(quote! { () })?
    };

    if let syn::Data::Enum(data_enum) = &ast.data {
        let inline_fragments_impl = InlineFragmentsImpl {
            target_struct: ast.ident.clone(),
            type_lock: TypePath::concat(&[
                Ident::new_spanned(&attributes.query_module.value, attributes.query_module.span)
                    .into(),
                Ident::for_type(&attributes.graphql_type.value).into(),
            ]),
            argument_struct,
            possible_types: possible_types_from_enum(data_enum)?,
            graphql_type_name: attributes.graphql_type.value,
        };

        Ok(quote! { #inline_fragments_impl })
    } else {
        Err(syn::Error::new(
            Span::call_site(),
            format!("InlineFragments can only be derived from an enum"),
        ))
    }
}

fn possible_types_from_enum(
    data_enum: &syn::DataEnum,
) -> Result<Vec<(syn::Ident, syn::Type)>, syn::Error> {
    use syn::{spanned::Spanned, Fields};

    let mut result = vec![];
    for variant in &data_enum.variants {
        match &variant.fields {
            Fields::Unnamed(fields) => {
                if fields.unnamed.len() != 1 {
                    return Err(syn::Error::new(
                        variant.fields.span(),
                        "InlineFragments derive requires enum variants to have one field",
                    ));
                }
                let field = fields.unnamed.first().unwrap();
                result.push((variant.ident.clone(), field.ty.clone()));
            }
            Fields::Named(_) => {
                return Err(syn::Error::new(
                    variant.fields.span(),
                    "Can't derive InlineFragments on an enum with named fields",
                ))
            }
            Fields::Unit => {
                return Err(syn::Error::new(
                    variant.fields.span(),
                    "Can't derive InlineFragments on an enum with a unit variant",
                ))
            }
        }
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
            impl ::cynic::InlineFragments for #target_struct {
                type TypeLock = #type_lock;
                type Arguments = #arguments;

                fn fragments(arguments: Self::Arguments) ->
                Vec<(String, ::cynic::SelectionSet<'static, Self, Self::TypeLock>)>
                {
                    use ::cynic::QueryFragment;

                    let mut rv = vec![];
                    #(
                        rv.push((
                            #internal_types::graphql_type(),
                            #internal_types::fragment(arguments)
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

fn find_union_type(name: &str, schema: &graphql_parser::schema::Document) -> bool {
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
