# langram
A tiny library that implements some popular grammar parsing algorithms

# dependecies
- make
- xdg-open
- rust (edition 2021)
- multimap (version 0.9.1)
- anyhow (version 1.0.75)

# building
Run either of these in order to build the release version
```
$ make release
$ cargo build --all-features --release
```
To generate the test coverage info run the following script
```
$ source scripts/coverage.sh
```
You then will be able to find the coverage at the following path
```
$ xdg-open target/debug/coverage/index.html
```

# external links
LR(1)
- https://neerc.ifmo.ru/wiki/index.php?title=LR(1)-%D1%80%D0%B0%D0%B7%D0%B1%D0%BE%D1%80

LR(0)
- https://neerc.ifmo.ru/wiki/index.php?title=LR(0)-%D1%80%D0%B0%D0%B7%D0%B1%D0%BE%D1%80 

Earley
- https://neerc.ifmo.ru/wiki/index.php?title=%D0%90%D0%BB%D0%B3%D0%BE%D1%80%D0%B8%D1%82%D0%BC_%D0%AD%D1%80%D0%BB%D0%B8
