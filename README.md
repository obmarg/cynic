# cynic

A GraphQL library for rust, built with the following principles:

1. Users should own the data structures used for querying.  Other libraries use macros to generate structs from a GraphQL query but this robs the user of control over the data.
2. Queries should be type safe.
3. Provide derives for the default behaviour, but allow that to be overridden when needed.
