# changed-files-rs

![Release](https://github.com/zcong1993/changed-files-rs/workflows/Release/badge.svg)
[![version](https://img.shields.io/crates/v/changed-files-rs.svg?colorB=319e8c)](https://crates.io/crates/changed-files-rs)

> rust port jest-changed-files

此项目为玩具项目, 意图在于熟悉 rust 语言和 rust CI 配置.

CI 配置 release 时触发以下操作:

1. 发布包到 crates.io
1. 构建 linux 和 osx 的二进制文件, 并上传 release assets
1. 更新 homebrew formula 源

## License

MIT &copy; zcong1993
