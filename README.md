## ***`Condex`***

---

Extract tokens by simple condition expression.


[![CI][ci-badge]][ci-url]
[![Crates.io][crates-badge]][crates-url]
[![Licensed][license-badge]][license-url]
[![Twitter][twitter-badge]][twitter-url]

[ci-badge]: https://github.com/just-do-halee/condex/actions/workflows/rust.yml/badge.svg
[crates-badge]: https://img.shields.io/crates/v/condex.svg?labelColor=383636
[license-badge]: https://img.shields.io/crates/l/condex?labelColor=383636
[twitter-badge]: https://img.shields.io/twitter/follow/do_halee?style=flat&logo=twitter&color=4a4646&labelColor=333131&label=just-do-halee

[ci-url]: https://github.com/just-do-halee/condex/actions
[twitter-url]: https://twitter.com/do_halee
[crates-url]: https://crates.io/crates/condex
[license-url]: https://github.com/just-do-halee/condex
| [Docs](https://docs.rs/condex) | [Latest Note](https://github.com/just-do-halee/condex/blob/main/CHANGELOG.md) |

```toml
[dependencies]
condex = "1.0.0"
```

---

# Example
```rust

    #[derive(Debug, Clone, Copy, PartialEq)]
    enum Token {
        TagName,
        NameType,
        Value,
        AllInOne,
    }
    impl TokenKind for Token {}

    let mut builder = CondexBuilder::new(&[
        (Token::TagName, &["@-("]),
        (Token::NameType, &["[(,]  -  :  - [,=]"]),
        (Token::Value, &["=-[,)]"]),
        (Token::AllInOne, &["@-(", "[(,]  -  :  - [,=]", "=-[,)]"]),
    ]);

    let source = "@hello-man(name: type = value, name2: type2, name3: type3 = value3)";

    for (i, c) in source.char_indices() {
        builder.test(c, i);
    }
    let finals = builder.finalize_with_source(source);
    eprintln!("{:#?}", finals);
    
```
