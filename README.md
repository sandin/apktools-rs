# apktools-rs

Tools for android apk file.

# Usage

Get package name:
```
$ apktools packagename <apkfile>
$ output: com.your.package.name
```

Get debuggable:
```
$ apktools debuggable <apkfile>
$ output: false
```

* `true`: debug apk
* `false`: release apk

# Build

```
cargo build --release
```
