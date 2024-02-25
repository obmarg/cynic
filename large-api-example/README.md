```
$ tree . -I target -I Cargo.lock
.           <-- workspace root
├── Cargo.toml
├── README.md
├── api     <-- uses query structs
│   ├── Cargo.toml
│   └── src
│       └── main.rs
├── query   <-- define query structs and check against schema
│   ├── Cargo.toml
│   ├── build.rs
│   └── src
│       └── lib.rs
└── schema  <-- generate schema structs
    ├── Cargo.toml
    ├── build.rs
    └── src
        └── lib.rs
```
