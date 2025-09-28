use std::path::PathBuf;

use clap::{Args, Parser, Subcommand, ValueEnum};
use colored::Colorize;

mod introspect;

fn main() {
    let cli = Cli::parse();

    #[allow(clippy::single_match)]
    match cli.command {
        Some(Commands::Introspect(args)) => {
            if let Err(error) = introspect::introspect(args) {
                let output = format!("{error}").red();
                eprintln!("{output}");
            }
        }
        Some(Commands::Querygen(args)) => {
            let schema = std::fs::read_to_string(&args.schema).unwrap();
            let query = std::fs::read_to_string(&args.query).unwrap();

            println!(
                "{}",
                cynic_querygen::document_to_fragment_structs(
                    query,
                    schema,
                    &cynic_querygen::QueryGenOptions::default()
                )
                .unwrap()
            );
        }
        None => {}
    }
}

/// CLI for the cynic, the Rust GraphQL client library
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
#[command(arg_required_else_help(true))]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Runs an introspection query against a GraphQL server and outputs the servers schema
    Introspect(IntrospectArgs),

    /// Runs querygen that allows you to generate Rust code from a schema and for a query
    Querygen(QuerygenArgs),
}

#[derive(Args)]
struct IntrospectArgs {
    /// The URL of the GraphQL schema that we should introspect
    url: String,
    /// Any headers to send with the introspection request
    ///
    /// These should be in HTTP format e.g. `-H "Authorization: Bearer a_token_123"`
    #[arg(short = 'H', long = "header")]
    headers: Vec<String>,
    /// The name of a file we should output the schema into.
    ///
    /// By default we print to stdout.
    #[arg(short, long)]
    output: Option<String>,
    /// The version of the GraphQL specificaiton that the remote GraphQL server implements
    ///
    /// Different versions of GraphQL expose different fields via introspection, so we need to know
    /// which set of fields to ask for.
    ///
    /// By default we run an additional query to figure out what the server we're talking to
    /// supports.
    #[arg(long, default_value_t = GraphQlVersion::AutoDetect)]
    server_version: GraphQlVersion,
}

#[derive(clap::Parser)]
struct QuerygenArgs {
    /// The path to a GraphQL schema file.
    #[arg(long)]
    schema: PathBuf,

    /// The path to a GraphQL query file.
    #[arg(long)]
    query: PathBuf,
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, ValueEnum, Default)]
enum GraphQlVersion {
    /// Run an introspection query compatible with the 2018 GraphQL specification
    #[value(name = "2018")]
    TwentyEighteen,
    /// Run an introspection query compatible with the 2021 GraphQL specification
    #[value(name = "2021")]
    TwentyTwentyOne,
    /// Run an introspection query compatible with the 2025 GraphQL specification
    #[value(name = "2025")]
    TwentyTwentyFive,
    /// Run an additional query to determine what the GraphQL server supports
    #[value(name = "auto")]
    #[default]
    AutoDetect,
}

impl std::fmt::Display for GraphQlVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            GraphQlVersion::TwentyEighteen => write!(f, "2018"),
            GraphQlVersion::TwentyTwentyOne => write!(f, "2021"),
            GraphQlVersion::TwentyTwentyFive => write!(f, "2025"),
            GraphQlVersion::AutoDetect => write!(f, "auto"),
        }
    }
}
