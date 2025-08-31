use colored::{ColoredString, Colorize};
use cynic::http::ReqwestBlockingExt;
use cynic_introspection::{
    CapabilitiesQuery, CapabilitySet, IntrospectionQuery, SpecificationVersion,
};
use reqwest::blocking::Client;

use super::{GraphQlVersion, IntrospectArgs};

pub(crate) fn introspect(args: IntrospectArgs) -> Result<(), IntrospectError> {
    let client = Client::new();
    let capabilities = match args.server_version {
        GraphQlVersion::TwentyEighteen => SpecificationVersion::June2018.capabilities(),
        GraphQlVersion::TwentyTwentyOne => SpecificationVersion::October2021.capabilities(),
        GraphQlVersion::AutoDetect => detect_capabilities(&client, &args)?,
    };

    let response = client
        .build(&args)?
        .run_graphql(&IntrospectionQuery::with_capabilities(capabilities))?;

    let errors = response.errors.unwrap_or_default();
    if !errors.is_empty() {
        eprintln!("{}", "Errors while introspecting: ".red(),);

        for error in errors {
            eprintln!("- {}", error.message.red());
        }
        eprintln!();
    }

    let Some(data) = response.data else {
        return Err(IntrospectError::IntrospectionQueryFailed);
    };

    let schema = data.into_schema()?;

    match args.output {
        None => print!("{}", schema.to_sdl()),
        Some(path) => {
            std::fs::write(&path, schema.to_sdl())?;
            eprintln!("{}", format!("Schema was written to {path}").green())
        }
    }

    Ok(())
}

#[derive(thiserror::Error, Debug)]
pub(crate) enum IntrospectError {
    #[error("The GraphQL server returned an error")]
    GraphQlError,
    #[error(transparent)]
    HttpError(#[from] cynic::http::CynicReqwestError),
    #[error("Couldn't parse a header from {0}.  Make sure you've passed a header of the form `Name: Value`")]
    MalformedHeaderArgument(String),
    #[error("Couldn't convert introspection result into schema: {0}")]
    SchemaError(cynic_introspection::SchemaError),
    #[error(
        "The introspection query seems to have failed.  Try looking in the response for errors"
    )]
    IntrospectionQueryFailed,
    #[error("Couldn't write the schema to file: {0}")]
    IOError(#[from] std::io::Error),
}

impl From<cynic_introspection::SchemaError> for IntrospectError {
    fn from(value: cynic_introspection::SchemaError) -> Self {
        match value {
            cynic_introspection::SchemaError::IntrospectionQueryFailed => {
                IntrospectError::IntrospectionQueryFailed
            }
            other => IntrospectError::SchemaError(other),
        }
    }
}

fn detect_capabilities(
    client: &Client,
    args: &IntrospectArgs,
) -> Result<CapabilitySet, IntrospectError> {
    use cynic::QueryBuilder;

    let output = format!("Detecting capabilities of {}", args.url).bright_black();
    eprintln!("{output}");

    let capabilities = client
        .build(args)?
        .run_graphql(&CapabilitiesQuery::build(()))?
        .data
        .ok_or(IntrospectError::GraphQlError)?
        .capabilities();

    eprintln!("{}", capability_string(&capabilities));

    Ok(capabilities)
}

fn capability_string(caps: &CapabilitySet) -> ColoredString {
    match caps.version_supported() {
        SpecificationVersion::June2018 => {
            "Server supports the June 2018 specification".bright_black()
        }
        SpecificationVersion::October2021 => {
            "Server supports the October 2021 specification".bright_black()
        }
        _ => "Server supports an unknown version of GraphQL".bright_black(),
    }
}

trait ReqwestExt {
    fn build(
        &self,
        args: &IntrospectArgs,
    ) -> Result<reqwest::blocking::RequestBuilder, IntrospectError>;
}

impl ReqwestExt for Client {
    fn build(
        &self,
        args: &IntrospectArgs,
    ) -> Result<reqwest::blocking::RequestBuilder, IntrospectError> {
        let mut builder = self.post(&args.url);
        for header in &args.headers {
            let mut split = header.splitn(2, ':');
            let name = split
                .next()
                .ok_or_else(|| IntrospectError::MalformedHeaderArgument(header.clone()))?;
            let value = split
                .next()
                .ok_or_else(|| IntrospectError::MalformedHeaderArgument(header.clone()))?;
            builder = builder.header(name.trim(), value.trim());
        }
        Ok(builder)
    }
}
