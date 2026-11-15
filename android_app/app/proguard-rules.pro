# Referenced by app/build.gradle.kts in the release buildType's
# proguardFiles list. isMinifyEnabled is false today, so Gradle doesn't
# read this file. It needs to exist so that turning minify on doesn't
# fail immediately with "file not found".
#
# When R8 is enabled, add keep rules here for:
#   - JNA (net.java.dev.jna) and the UniFFI-generated binding classes,
#     which JNA loads reflectively.
#   - The z.zndroid.debug.* broadcast receivers (only if debug-only
#     builds need them kept; release builds don't ship the debug
#     source set at all).
#   - Any future reflection-driven code.
