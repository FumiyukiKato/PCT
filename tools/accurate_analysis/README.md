using Sqlite3 and diesel


sqlite3 setup by diesel cli
```
$ cargo install diesel_cli --no-default-features --features sqlite 
$ diesel migration run --database-url=trajectory.db  
```

Query from file
```
$ cargo run --release -- -i ../trajectory/data/client -m query 
```

Insert from file
```
$  cargo run -- -i input.csv -m insert
```

Delte whole data
```
$  cargo run --  -m trunc
```