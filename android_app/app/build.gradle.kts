import java.util.Properties

plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

// Host-side schema drift check. Runs before every build so drift between
// the Rust blueprints and the Kotlin side fails the build with a report
// naming the specific column. Skipped when the enclosing script has
// already run it (SCHEMA_DRIFT_CHECKED is set).
val schemaDriftCheck = tasks.register<Exec>("schemaDriftCheck") {
    workingDir = rootDir
    val checkScript = rootDir.resolve("schema_drift_detector/check.sh")
    commandLine(checkScript.absolutePath)
    isIgnoreExitValue = false
    onlyIf {
        System.getenv("SCHEMA_DRIFT_CHECKED").isNullOrEmpty() && checkScript.exists()
    }
}
tasks.named("preBuild").configure { dependsOn(schemaDriftCheck) }

// Machine-local values, gitignored via local.properties (standard
// Android convention, ignored at repo root and in android_app/).
// Add `server_url_default=ws://<your-pc-lan-ip>:3000/ws` to
// android_app/local.properties. If absent, BuildConfig.SERVER_URL_DEFAULT
// is empty and the app falls back to loopback (works with
// `adb reverse tcp:3000 tcp:3000`). Nothing personal is committed.
val localProps = Properties().apply {
    val f = rootProject.file("local.properties")
    if (f.exists()) f.inputStream().use { load(it) }
}
val serverUrlDefault = localProps.getProperty("server_url_default") ?: ""

android {
    namespace = "z.zndroid"
    compileSdk = 37

    defaultConfig {
        applicationId = "z.zndroid"
        minSdk = 31
        targetSdk = 37
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"

        buildConfigField("String", "SERVER_URL_DEFAULT", "\"$serverUrlDefault\"")
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
        buildConfig = true
    }
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation("net.java.dev.jna:jna:5.16.0@aar")
    testImplementation(libs.junit)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.junit)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
    debugImplementation(libs.androidx.compose.ui.tooling)
}