# Android C++ Symbol Issue Fix

This document explains how to fix the `__cxa_pure_virtual` symbol error on Android.

## The Issue

The error occurs because the Android app is missing the C++ standard library that provides the `__cxa_pure_virtual` symbol, which is required by the Rust code when compiled for Android.

## Solution

The solution involves several steps:

1. Create a `.cargo/config.toml` file in the `src-tauri` directory with the following content:

```toml
[target.aarch64-linux-android]
rustflags = ["-C", "link-arg=-lc++_shared"]

[target.armv7-linux-androideabi]
rustflags = ["-C", "link-arg=-lc++_shared"]

[target.i686-linux-android]
rustflags = ["-C", "link-arg=-lc++_shared"]

[target.x86_64-linux-android]
rustflags = ["-C", "link-arg=-lc++_shared"]
```

2. Modify the `build.rs` script to explicitly link the C++ shared library when building for Android:

```rust
// In build.rs
let target = env::var("TARGET").unwrap_or_default();
if target.contains("android") {
    println!("cargo:rustc-link-lib=c++_shared");
}
```

3. Run the `copy-cxx-lib.sh` script to copy the C++ shared library from the NDK to the Android project's jniLibs directory:

```bash
./copy-cxx-lib.sh
```

4. Make sure the Android build is configured to include the C++ shared library in the APK:

```
# In app/build.gradle.kts
android {
    // ...
    defaultConfig {
        // ...
        externalNativeBuild {
            cmake {
                arguments("-DANDROID_STL=c++_shared")
            }
        }
    }
}
```

## Additional Notes

- The C++ standard library (`libc++_shared.so`) must be included in the APK for each supported architecture.
- The app must load this library at runtime.
- This issue is specific to Android, as other platforms include the C++ standard library by default.

## References

- [Android NDK documentation on C++ support](https://developer.android.com/ndk/guides/cpp-support)
- [Rust documentation on linking with C++ libraries](https://doc.rust-lang.org/rustc/platform-support/android.html)