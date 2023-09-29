# gents

[![MIT/Apache 2.0](https://img.shields.io/badge/license-MIT/Mit-blue.svg)](./LICENSE)

`gents` is a tool for generating **Typescript** interfaces from **Rust** code.
We can easily use `serde-json` to communicate between **Rust** and **Typescript**,
without writing **Typescript** stubs trivially.
It is helpful when your API changes frequently.

It is designed for [LogiSheets](https://github.com/proclml/LogiSheets) and
is inspired by [`ts-rs`](https://github.com/Aleph-Alpha/ts-rs). Many thanks to them!

Your issues and PRs are welcome!

## Why do you need `gents`?

- Writing a easy web server in Rust and you hate things like `grpc-web`
- Writing a wasm project

## Why not  `ts-rs`?

- `ts-rs` generates the files when running `cargo test` and in this way we must
commit those generated files into our repo.
It is not necessary and is even an obstacle when we use other build tools like `bazel`.
`gents` acts as a binary to generate **Typescript** files.

- `gents` introduces a concept *Group* that from all the members in
this group files generated will be placed in the same directory. **Group** is seperate from the other group even though they can share some
dependecies. Therefore, `gents` requires you to specify the *file_name* on structs
or enums and to specify the *dir* on group, while `ts-rs` requires specifing the *path* on every item.

- `gents` helps you manage the export files. And it gathers all the dependencies automatically.

- `gents` is well support for referencing other crates.

- Code generated by `ts-rs` is not match our coding style.

## How to use `gents`?

In your **Rust** code:

You should import `gents` in your Cargo.toml.

```toml
[dev-dependencies]
gents = "0.4"
gents_derives = "0.4"
```

```rust
use gents_derives::TS;

#[derive(TS)]
#[ts(file_name = "person.ts", rename_all = "camelCase")]
pub struct Person {
    pub age: u16,
    pub en_name: String,
}

#[derive(TS)]
#[ts(file_name = "group.ts", rename_all = "camelCase")]
pub struct Group {
    pub name: String,
    pub capacity: u16,
    pub members: Vec<Person>,
    pub leader: Option<Person>,
}
```

`derive(TS)` generates code under `#[cfg(any(test, feature="gents"))]`, which means `gents` does not make any difference until you run it.

If all the structs or enums derived from `TS` are in the same crate,
we recommend that you can write a simple unit test to generate the files like below:

```rust

#[ignore]
#[test]
fn gents() {
    use gents::FileGroup;
    let mut group = FileGroup::new();
    // You don't need to add Person because it is a dependency of Group and it will be added automatically
    group.add::<Group>();
    // If you need to generate the index.ts file, set true.
    group.gen_files("outdir", false);
}
```

After running this test, there are 2 files generated in `outdir`:

- person.ts
- group.ts

Check more cases and usage in the `tests` folder.

If your `derive(TS)`s are from different crates, then you should need to define a feature called `gents`. Please check the detailed usage in [LogiSheets](https://github.com/proclml/LogiSheets/blob/master/crates/buildtools/src/generate.rs).
