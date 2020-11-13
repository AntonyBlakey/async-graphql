# A GraphQL server library implemented in Rust

`async-graphql-no-send` is a fork of [`async-graphql`](https://github.com/async-graphql/async-graphql)
that removes all `Send` and `Sync` requirements so that it can be used in projects running in single-threaded
contexts e.g. wasm32, that are written with that explicit assumption e.g. they use `Rc` rather than `Arc`.

Most of the default features have been removed to ease tracking the upstream project.
