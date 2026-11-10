# The activity is found by name from the manifest. craft-activity's consumer rules keep its own
# classes (called by JNI) and GameActivity's Java side.
-keep class io.github.dexpdfcraft.** { *; }
