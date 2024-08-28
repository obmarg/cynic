#[derive(Debug)]
pub struct UseSchemaParams {
    pub schema_filename: String,
}

impl syn::parse::Parse for UseSchemaParams {
    fn parse(input: syn::parse::ParseStream<'_>) -> syn::Result<Self> {
        let lit_str = input.parse::<syn::LitStr>()?;

        Ok(UseSchemaParams {
            schema_filename: lit_str.value(),
        })
    }
}
