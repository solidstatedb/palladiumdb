# palladiumdb
A cuncurrent key value store, column oriented database and more, written in rust.

## Overview

palladiumdb is a rust port of the [silicondb](https://github.com/solidstatedb/silicondb) project. This has the same goals as silicondb, with the added objective for the author to become familiar with the Rust ecosystem and language.

As with silicondb, this project does not aim to with existing NoSQL databases or provides any novel features. 

## 💻 Storage Engine modules

- [x] A concurrent in memory map implemented as a hashtable.
- [ ] A lock-free variant of the map mentioned above.
- [ ] A wait-free simulation of the lock-free map.
- [ ] A lock-free concurrent SSTable simulated in a wait-free manner.
- [ ] Compaction operations on SSTable.
- [ ] Column oriented storage support. (Needs further elaboration)
- [ ] Content correctness verification and replication using merkle trees for SSTable and Column oriented storage.
- [ ] Message sequencing with merkle trees.

## 🏂 Features
- [ ] Partial implementation of the Redis protocol:
    - [ ] Key, Value retreival
    - [ ] Key Ranged operations
- [ ] REST API server for accessing data.
- [ ] CAP compliant.

## 🏗️ Build Instructions

### Pre-requisites

- cargo

```
git clone git@github.com:solidstatedb/palladiumdb.git
cd palladiumdb
cargo build --release
```

This produces a binary `target/release/palladiumdb` which is the silicondb daemon server.

## 🧪 Testing

```
cargo test
```

This should run all tests.

## License

`palladiumdb` is licensed under the MIT License See [LICENSE](./LICENSE) for the full license text.

