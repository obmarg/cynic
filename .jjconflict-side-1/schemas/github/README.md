### GitHub Schema

This crate contains type definitions for using cynic with the GitHub schema.

It's in a separate crate because the GitHub schema is quite large and defining
it in it's own crate means we don't have to recompile it as often.  

It also acts as a nice test of whether we can define schemas in separate crates
from queries.
