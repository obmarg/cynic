use proc_macro2::TokenStream;

use crate::Ident;

use super::FragmentDeriveField;
use crate::schema::types as schema;

pub enum DeserializeImpl {
    Standard(StandardDeserializeImpl),
    Spreading(SpreadingDeserializeImpl),
}

pub struct StandardDeserializeImpl {
    target_struct: Ident,
    fields: Vec<Field>,
}

pub struct SpreadingDeserializeImpl {
    target_struct: Ident,
    fields: Vec<Field>,
}

struct Field {
    rust_name: proc_macro2::Ident,
    ty: syn::Type,
    field_variant_name: proc_macro2::Ident,
    serialized_name: Option<String>,
    is_spread: bool,
    is_flattened: bool,
}

impl DeserializeImpl {
    pub fn new(
        fields: &[(&FragmentDeriveField, Option<&schema::Field<'_>>)],
        name: &syn::Ident,
    ) -> DeserializeImpl {
        let spreading = fields.iter().any(|f| *f.0.spread);

        let target_struct = Ident::new_spanned(&name.to_string(), name.span());
        let fields = fields
            .iter()
            .map(|(field, schema_field)| process_field(field, *schema_field))
            .collect();

        match spreading {
            true => DeserializeImpl::Spreading(SpreadingDeserializeImpl {
                target_struct,
                fields,
            }),
            false => DeserializeImpl::Standard(StandardDeserializeImpl {
                target_struct,
                fields,
            }),
        }
    }
}

impl quote::ToTokens for DeserializeImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DeserializeImpl::Standard(inner) => inner.to_tokens(tokens),
            DeserializeImpl::Spreading(inner) => inner.to_tokens(tokens),
        }
    }
}

impl quote::ToTokens for StandardDeserializeImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;
        let serialized_names = self
            .fields
            .iter()
            .map(|f| {
                proc_macro2::Literal::string(
                    f.serialized_name
                        .as_deref()
                        .expect("non-spread fields must have a serialized_name"),
                )
            })
            .collect::<Vec<_>>();
        let field_variant_names = self
            .fields
            .iter()
            .map(|f| &f.field_variant_name)
            .collect::<Vec<_>>();
        let field_names = self.fields.iter().map(|f| &f.rust_name).collect::<Vec<_>>();
        let field_decodes = self.fields.iter().map(|f| {
            let field_name = &f.rust_name;
            let ty = &f.ty;
            if f.is_flattened {
                quote! {
                    #field_name = Some(map.next_value::<::cynic::__private::Flattened<#ty>>()?.into_inner());
                }
            } else {
                quote! {
                    #field_name = Some(map.next_value()?);
                }
            }
        });

        let expecting_str =
            proc_macro2::Literal::string(&format!("struct {}", self.target_struct.rust_name()));

        let struct_name = proc_macro2::Literal::string(&self.target_struct.rust_name());

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::serde::Deserialize<'de> for #target_struct {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::cynic::serde::Deserializer<'de>,
                {
                    #[derive(::cynic::serde::Deserialize)]
                    #[serde(field_identifier, crate="::cynic::serde")]
                    #[allow(non_camel_case_types)]
                    enum Field {
                        #(
                            #[serde(rename = #serialized_names)]
                            #field_variant_names,
                        )*
                        #[serde(other)]
                        __Other
                    }

                    struct Visitor;

                    impl <'de> ::cynic::serde::de::Visitor<'de> for Visitor {
                        type Value = #target_struct;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                            formatter.write_str(#expecting_str)
                        }

                        fn visit_map<V>(self, mut map: V) -> Result<#target_struct, V::Error>
                        where
                            V: ::cynic::serde::de::MapAccess<'de>,
                        {
                            #(
                                let mut #field_names = None;
                            )*
                            while let Some(key) = map.next_key()? {
                                match key {
                                    #(
                                        Field::#field_variant_names => {
                                            if #field_names.is_some() {
                                                return Err(::cynic::serde::de::Error::duplicate_field(#serialized_names));
                                            }
                                            #field_decodes
                                        }
                                    )*
                                    Field::__Other => {
                                        map.next_value::<::cynic::serde::de::IgnoredAny>()?;
                                    }
                                }
                            }
                            #(
                                let #field_names = #field_names.ok_or_else(|| ::cynic::serde::de::Error::missing_field(#serialized_names))?;
                            )*
                            Ok(#target_struct {
                                #(#field_names),*
                            })
                        }
                    }

                    const FIELDS: &'static [&'static str] = &[#(#serialized_names),*];

                    deserializer.deserialize_struct(#struct_name, FIELDS, Visitor)
                }
            }
        });
    }
}

impl quote::ToTokens for SpreadingDeserializeImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;

        let field_inserts = self.fields.iter().map(|f| {
            let field_name = &f.rust_name;
            let field_ty = &f.ty;
            if f.is_spread {
                quote! {
                    #field_name: <#field_ty as ::cynic::serde::Deserialize<'de>>::deserialize(
                        spreadable.spread_deserializer()
                    )?,
                }
            } else if f.is_flattened {
                let serialized_name = proc_macro2::Literal::string(
                    f.serialized_name
                        .as_deref()
                        .expect("non spread fields must have a serialized_name"),
                );
                quote! {
                    #field_name: spreadable.deserialize_field::<
                        ::cynic::__private::Flattened<#field_ty>
                    >(#serialized_name)?.into_inner()
                }
            } else {
                let serialized_name = proc_macro2::Literal::string(
                    f.serialized_name
                        .as_deref()
                        .expect("non spread fields must have a serialized_name"),
                );
                quote! {
                    #field_name: spreadable.deserialize_field(#serialized_name)?
                }
            }
        });

        tokens.append_all(quote! {
            #[automatically_derived]
            impl<'de> ::cynic::serde::Deserialize<'de> for #target_struct {
                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: ::cynic::serde::Deserializer<'de>,
                {
                    let spreadable = ::cynic::__private::Spreadable::<D::Error>::deserialize(deserializer)?;

                    Ok(#target_struct {
                        #(#field_inserts)*
                    })
                }
            }
        });
    }
}

fn process_field(field: &FragmentDeriveField, schema_field: Option<&schema::Field<'_>>) -> Field {
    // Should be ok to unwrap since we only accept struct style input
    let rust_name = field.ident.as_ref().unwrap();
    let field_variant_name = rust_name.clone();

    Field {
        field_variant_name,
        serialized_name: field
            .alias()
            .or_else(|| schema_field.map(|f| f.name.as_str().to_string())),
        rust_name: rust_name.clone(),
        ty: field.ty.clone(),
        is_spread: *field.spread,
        is_flattened: *field.flatten,
    }
}
