# Working with Large APIs

Some APIs have fairly large schemas, and this introduces some performance
challenges for cynic.  Runtime performance should be unaffected, but it can
lead to extended compile times and make rust-analyzer less responsive than it
would otherwise be.

There's several tricks to help with this though:

### Registering Schemas with rkyv

If you're not already you should be [pre-registering your schema](./schemas.md).

You should also enable the `rkyv` feature flag in `cynic_codegen`.  This allows
the pre-registration to store schemas in an optimised format, which avoids a
lot of comparatively slow parsing.

### Splitting Crates

Definitely consider moving your schema module into a separate crate.  These
modules contain _a lot_ of generated code and are quite expensive to compile.
Moving them to a separate crate should reduce the chance that unrelated changes
cause it to recompile.  Note that you'll need to register your schema in both
the schema module crate and any crate where you use the cynic derives.

You can also consider moving your query structs into their own crate, for
reasons similar to the above.  Though it may be worth testing whether this
actually helps - with rkyv turned on these shouldn't be too slow.  But it
really depends on how many of them you have.
