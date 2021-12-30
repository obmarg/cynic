#[derive(Debug)]
pub struct UseSchemaParams {
    pub schema_filename: String,
}

impl UseSchemaParams {
    fn new(schema_filename: String) -> Self {
        UseSchemaParams { schema_filename }
    }
}

impl syn::parse::Parse for UseSchemaParams {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let lit_str = input.parse::<syn::LitStr>()?;

        Ok(UseSchemaParams {
            schema_filename: lit_str.value(),
        })
    }
}
