# Hydra

Community bindings for the Libsql DB written in rust using JanetRS and the Rust implementation of Libsql.

### Usage example

```clojure

(import hydra as :db)
(def in-memory-db
  (db/open-local-db ":memory:"))

(db/query "select 1;" @[])

```

### Installation

```sh
$ jpm install https://github.com/RNGKing/hydra
```

### To do

* Batch Executions and Queries
* A better understanding of Rust FFI
* Remote executions through LibSQL
