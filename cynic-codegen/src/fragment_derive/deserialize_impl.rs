use proc_macro2::TokenStream;

use {
    super::FragmentDeriveField,
    crate::{generics_for_serde, schema::types as schema},
};

pub enum DeserializeImpl<'a> {
    Standard(StandardDeserializeImpl<'a>),
    Spreading(SpreadingDeserializeImpl<'a>),
}

pub struct StandardDeserializeImpl<'a> {
    target_struct: &'a syn::Ident,
    fields: Vec<Field>,
    generics: &'a syn::Generics,
}

pub struct SpreadingDeserializeImpl<'a> {
    target_struct: &'a syn::Ident,
    fields: Vec<Field>,
    generics: &'a syn::Generics,
}

struct Field {
    rust_name: proc_macro2::Ident,
    ty: syn::Type,
    field_variant_name: proc_macro2::Ident,
    serialized_name: Option<String>,
    is_spread: bool,
    is_flattened: bool,
}

impl<'a> DeserializeImpl<'a> {
    pub fn new(
        fields: &[(&FragmentDeriveField, Option<&schema::Field<'_>>)],
        name: &'a syn::Ident,
        generics: &'a syn::Generics,
    ) -> Self {
        let spreading = fields.iter().any(|f| *f.0.spread);

        let target_struct = name;
        let fields = fields
            .iter()
            .map(|(field, schema_field)| process_field(field, *schema_field))
            .collect();

        match spreading {
            true => DeserializeImpl::Spreading(SpreadingDeserializeImpl {
                target_struct,
                fields,
                generics,
            }),
            false => DeserializeImpl::Standard(StandardDeserializeImpl {
                target_struct,
                fields,
                generics,
            }),
        }
    }
}

impl quote::ToTokens for DeserializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            DeserializeImpl::Standard(inner) => inner.to_tokens(tokens),
            DeserializeImpl::Spreading(inner) => inner.to_tokens(tokens),
        }
    }
}

impl quote::ToTokens for StandardDeserializeImpl<'_> {
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
                    #field_name = Some(__map.next_value::<cynic::__private::Flattened<#ty>>()?.into_inner());
                }
            } else {
                quote! {
                    #field_name = Some(__map.next_value()?);
                }
            }
        });

        let struct_name = self.target_struct.to_string();
        let expecting_str = proc_macro2::Literal::string(&format!("struct {}", &struct_name));
        let struct_name = proc_macro2::Literal::string(&struct_name);

        let (_, ty_generics, _) = self.generics.split_for_impl();
        let generics_with_de = generics_for_serde::with_de_and_deserialize_bounds(self.generics);
        let (impl_generics, ty_generics_with_de, where_clause) = generics_with_de.split_for_impl();

        tokens.append_all(quote! {
            #[automatically_derived]
            impl #impl_generics cynic::serde::Deserialize<'de> for #target_struct #ty_generics #where_clause {
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: cynic::serde::Deserializer<'de>,
                {
                    #[derive(cynic::serde::Deserialize)]
                    #[serde(field_identifier, crate="cynic::serde")]
                    #[allow(non_camel_case_types)]
                    enum __FragmentDeriveField {
                        #(
                            #[serde(rename = #serialized_names)]
                            #field_variant_names,
                        )*
                        #[serde(other)]
                        __Other
                    }

                    struct Visitor #generics_with_de #where_clause {
                        marker: ::core::marker::PhantomData<#target_struct #ty_generics>,
                        lifetime: ::core::marker::PhantomData<&'de ()>,
                    }

                    impl #impl_generics cynic::serde::de::Visitor<'de> for Visitor #ty_generics_with_de #where_clause {
                        type Value = #target_struct #ty_generics;

                        fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                            formatter.write_str(#expecting_str)
                        }

                        fn visit_map<V>(self, mut __map: V) -> Result<Self::Value, V::Error>
                        where
                            V: cynic::serde::de::MapAccess<'de>,
                        {
                            #(
                                let mut #field_names = None;
                            )*
                            while let Some(__key) = __map.next_key()? {
                                match __key {
                                    #(
                                        __FragmentDeriveField::#field_variant_names => {
                                            if #field_names.is_some() {
                                                return Err(cynic::serde::de::Error::duplicate_field(#serialized_names));
                                            }
                                            #field_decodes
                                        }
                                    )*
                                    __FragmentDeriveField::__Other => {
                                        __map.next_value::<cynic::serde::de::IgnoredAny>()?;
                                    }
                                }
                            }
                            #(
                                let #field_names = #field_names.ok_or_else(|| cynic::serde::de::Error::missing_field(#serialized_names))?;
                            )*
                            Ok(#target_struct {
                                #(#field_names),*
                            })
                        }
                    }

                    const FIELDS: &'static [&str] = &[#(#serialized_names),*];

                    deserializer.deserialize_struct(
                        #struct_name,
                        FIELDS,
                        Visitor {
                            marker: ::core::marker::PhantomData,
                            lifetime: ::core::marker::PhantomData,
                        },
                    )
                }
            }
        });
    }
}

impl quote::ToTokens for SpreadingDeserializeImpl<'_> {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        use quote::{quote, TokenStreamExt};

        let target_struct = &self.target_struct;

        let field_inserts = self.fields.iter().map(|f| {
            let field_name = &f.rust_name;
            let field_ty = &f.ty;
            if f.is_spread {
                quote! {
                    #field_name: <#field_ty as cynic::serde::Deserialize<'de>>::deserialize(
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
                        cynic::__private::Flattened<#field_ty>
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

        let (_, ty_generics, where_clause) = self.generics.split_for_impl();
        let generics_with_de = generics_for_serde::with_de_and_deserialize_bounds(self.generics);
        let (impl_generics, _, _) = generics_with_de.split_for_impl();

        tokens.append_all(quote! {
            #[automatically_derived]
            impl #impl_generics cynic::serde::Deserialize<'de> for #target_struct #ty_generics #where_clause {
                fn deserialize<__D>(deserializer: __D) -> Result<Self, __D::Error>
                where
                    __D: cynic::serde::Deserializer<'de>,
                {
                    let spreadable = ::cynic::__private::Spreadable::<__D::Error>::deserialize(deserializer)?;

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
