# apktools-rs

Tools for android apk file.

# Usage

Get package name:
```
$ apktools packagename <apkfile>
$ com.your.package.name
```

Get debuggable:
```
$ apktools debuggable <apkfile>
$ false
```

output:
* `true`: debug apk
* `false`: release apk

# Build

```
cargo build --release
```
