# kv-server
支持 set、get、mset、mget、exists、mexists、getall命令，以及subscribe、unsubscribe。publish命令
## 运行
```sh
cargo run --bin server
```
## 执行
```sh
cargo run --bin cli get t1 k1 
```
i64: i@@123, String: s@@123, f64: d@@123, bool: f@@false,Vec<u8> b@@123
```sh
cargo run --bin cli set t2 k2 s@@123
```
