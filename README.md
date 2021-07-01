# apktools-rs

Tools for android apk file.

# Usage

Get package name:
```
$ apktools packagename <apkfile>
$ output: com.your.package.name
```

Set `debuggable=true`:
```
$ apktools debuggable <apkfile>
$ output: success
```

# Build

```
cargo build --release
```
