# mproto

*disclaimer: mproto is experimental and not ready for general use. Its wire format is subject to change. See the "status" section below.*

Mproto is a serialization system. Object types are described in mproto's schema language, and code to work with those types is generated for different programming languages. It is similar to other serialization projects such as Cap'n Proto's serialzation, Thrift's type system, and Flatbuffers.

Mproto's primary purpose is to serve as the serialization system for [modrpc](https://github.com/modrpc-org/modrpc), but it can be used independently as well.

## Features

- Record types - `struct`
- Tagged unions - `enum`
- Type parameters - `struct Foo<Bar, Baz>`
- Built-in `option` and `result` types
- Support for both eager and lazy decoding - "only read what you need"
- Optional `no_std` and no-alloc support in generated Rust code
- Language targets:
    [x] Rust
    [x] TypeScript
    [ ] (planned) Python
    [ ] TBD

## What does a schema look like?

```rust
struct Vec2 {
    x: f32,
    y: f32,
}

struct Foo<Bar, Baz> {
    a_number: u32,
    bar: Bar,
    an_option: option<i64>,
    baz: result<Success<Baz>, string>,
}

enum Success<Baz> {
    One {
        x: bool,
        positions: [Vec2],
    }
    Two {
        baz: Baz,
        blob: [u8],
    }
    Nested {
        success: box<Success<Baz>>,
    }
}
```

## Status

Mproto is still experimental and should be considered a proof-of-concept.

### Tooling issues

To date, virtually no effort has been put into producing helpful error messages when parsing / generating code from schemas.

Related to the previous point, there is no IR layer of the AST yet. Type references are resolved while generating code, which can result in cryptic panic messages when generating code. At some point I want to add an IR layer where references are resolved and helpful error messages are emitted before attempting to generate code.

### Notes on schema language and encoding

At some point I want to add fixed-sized arrays, e.g. `[Foo; 16]`.

Should we introduce some complexity to make `bool` fields take up a single bit on the wire?

The `box` primitive has two use-cases:
- allowing recursive objects, and
- manually introducing indirection to an enum to avoid wasting wire space (see the following section about the current encoding scheme).

Currently the codegen'd Rust owned representation of a `box` always maps to a Rust `Box` - this is necessary for the recursive types use-case. But it is not necessary for the enum indirection. Should mproto's codegen detect whether a type can be recursive and only use `Box` in the owned Rust representation if necessary? Probably so - we'll need to add a recursive type detector in the compiler at some point regardless to emit an error rather than overflowing the stack.

## Encoding scheme and object size

Mproto's wire format is a binary encoding scheme, and so should be more compact than human-readable encodings like JSON. But in general, mproto does not try to have the most space-efficient wire format. If wire size matters, mproto encoded messages can be compressed via [zero-run-length encoding](https://en.wikipedia.org/wiki/Run-length_encoding) or lz4, for example, before being sent over the wire.

Under the hood, every mproto-encoded message has two partitions - the base area and the scratch area. Datums whose size is static (does not depend on runtime value) are placed in the base area. The length of the portion of an encoded datum that's in the base area is referred to as the datum's base length. Datums whose encoeded size *does* depend on runtime values bump-allocate space within the scratch area and store the offset of that scratch space in the base area.

All primitives besides `string`, `list`, and `box` are statically sized. Integer types take up their full bitwidth on the wire - so a `u32` is 4 bytes regardless of the runtime value. Structs and enums made purely of statically sized fields are also statically sized.

An enum's base length is the largest base length of its variants. If there is a large disparity between an enum's variants' base lengths, `box` can be used to add a layer of indirection between an enum and its variants' fields to avoid wasting wire space:

```rust
enum Foo {
    Bar { bar: box<Bar> },
    Baz { wow: u8 },
}

struct Bar {
    much_datas: [u8; 1024],
    many_size: BigObject,
}
```

## License

Apache 2.0
