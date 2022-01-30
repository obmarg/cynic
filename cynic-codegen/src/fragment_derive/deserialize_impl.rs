use proc_macro2::TokenStream;
use quote::format_ident;

use crate::Ident;

use super::{fragment_derive_type::FragmentDeriveType, FragmentDeriveField};
use crate::schema::types as schema;

pub struct DeserializeImpl {
    target_struct: Ident,
    fields: Vec<Field>,
}

struct Field {
    rust_name: proc_macro2::Ident,
    field_variant_name: proc_macro2::Ident,
    serialized_name: String,
}

impl DeserializeImpl {
    pub fn new(
        fields: &[(&FragmentDeriveField, &schema::Field<'_>)],
        name: &syn::Ident,
    ) -> DeserializeImpl {
        let target_struct = Ident::new_spanned(&name.to_string(), name.span());
        let fields = fields
            .iter()
            .map(|(field, schema_field)| process_field(field, schema_field))
            .collect();

        DeserializeImpl {
            target_struct,
            fields,
        }
    }
}

impl quote::ToTokens for DeserializeImpl {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;
        let serialized_names = self
            .fields
            .iter()
            .map(|f| proc_macro2::Literal::string(&f.serialized_name))
            .collect::<Vec<_>>();
        let field_variant_names = self
            .fields
            .iter()
            .map(|f| &f.field_variant_name)
            .collect::<Vec<_>>();
        let field_names = self.fields.iter().map(|f| &f.rust_name).collect::<Vec<_>>();

        let expecting_str =
            proc_macro2::Literal::string(&format!("struct {}", self.target_struct.rust_name()));

        let struct_name = proc_macro2::Literal::string(&self.target_struct.rust_name());

        // Note: I've typed this all out already but I _could_ just
        // generate a struct with the write serde attrs and this impl just becomes a case
        // of converting it...?

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
                                            #field_names = Some(map.next_value()?);
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

fn process_field(field: &FragmentDeriveField, schema_field: &schema::Field<'_>) -> Field {
    // Should be ok to unwrap since we only accept struct style input
    let rust_name = field.ident.as_ref().unwrap();
    let field_variant_name = rust_name.clone();

    Field {
        field_variant_name,
        serialized_name: field
            .alias()
            .unwrap_or_else(|| schema_field.name.as_str().to_string()),
        rust_name: rust_name.clone(),
    }
}
