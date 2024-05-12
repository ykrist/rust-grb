# TODO

Merge in non-breaking parts of dogfood branch.

Benchmarks:

- Batch add var/constr, set/get attribute methods: are they even worth supporting?

For next major release:

- Replace `c_char` in public interface with `char`
- Rework attributes/parameters, one type per attribute/parameter instead of enums
  - Keep the type parameters in `ParamGet/Set` traits.
  - Will simplify the codegen and exports, and hopefully make lints/IDE hints better
  - Completely static checking

- Add a `attr::Attribute` (analogous to `param::Parameter`).
- Implement `From<param::ParamStruct>` for `param::Parameter`, same for attributes.
- Basic Serde support for dynamic parameters and attributes (feature gated)

Sys crate:

- Opaque types like`Model` should be declared with a zero-variant enum, not struct.

Future Ideas:

- `Model::add_vars()` to return a `HashMap` (builder type?). Should be generic over the hasher type.
