# NativeActivity finds the activity by name from the manifest, and the Rust library will call into
# it through JNI (phase 3): keep the app's classes and their members.
-keep class io.github.dexpdfcraft.** { *; }
