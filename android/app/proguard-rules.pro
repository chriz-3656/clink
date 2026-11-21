-keepattributes *Annotation*
-keepclassmembers class * {
    @androidx.compose.runtime.Composable *;
}
-keep class kotlinx.serialization.** { *; }
-keepclassmembers class * {
    *** Companion;
}
