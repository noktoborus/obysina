// The package of Obusina: the activity of GameActivity in Java, and the
// library of `android/src` built by cargo-ndk for every ABI below.

plugins {
    id("com.android.application")
}

/// The ABIs of the package: phones, and the emulator of a desktop.
val abis = listOf("arm64-v8a", "x86_64")
/// The lowest android the package runs on; the libraries are built for it.
val minimum = 26
/// The root of the cargo workspace.
val workspace = rootProject.projectDir.parentFile
/// The version of the program: `version` of `[workspace.package]` of the
/// Cargo.toml of the workspace, which `make VERSION=x.y.z` sets.
val release = Regex("""(?ms)^\[workspace\.package\]$.*?^version = "(\d+)\.(\d+)\.(\d+)"$""")
    .find(workspace.resolve("Cargo.toml").readText())
    ?.destructured
    ?.let { (major, minor, patch) -> listOf(major, minor, patch).map(String::toInt) }
    ?: error("no version x.y.z in [workspace.package] of Cargo.toml")
/// The code of the version, which android counts up: two digits for the
/// minor number and two for the patch.
val releaseCode = release.let { (major, minor, patch) ->
    require(minor < 100 && patch < 100) { "the minor and the patch number are below 100" }
    major * 10000 + minor * 100 + patch
}

android {
    namespace = "ru.styxheim.obusina"
    compileSdk = 37
    ndkVersion = "30.0.16248370"
    System.getenv("ANDROID_NDK_HOME")?.let { ndkPath = it }

    defaultConfig {
        applicationId = "ru.styxheim.obusina"
        minSdk = minimum
        targetSdk = 37
        versionCode = releaseCode
        versionName = release.joinToString(".")
        ndk { abiFilters += abis }
    }

    signingConfigs {
        create("release") {
            System.getenv("OBUSINA_KEYSTORE")?.let { path ->
                storeFile = file(path)
                storePassword = System.getenv("OBUSINA_KEYSTORE_PASSWORD")
                keyAlias = System.getenv("OBUSINA_KEY_ALIAS")
                keyPassword = System.getenv("OBUSINA_KEY_PASSWORD")
            }
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            if (System.getenv("OBUSINA_KEYSTORE") != null) {
                signingConfig = signingConfigs.getByName("release")
            }
        }
    }

    sourceSets {
        getByName("debug").jniLibs.directories += layout.buildDirectory.dir("rust/debug").get().asFile.path
        getByName("release").jniLibs.directories += layout.buildDirectory.dir("rust/release").get().asFile.path
    }
}

dependencies {
    implementation("androidx.appcompat:appcompat:1.8.0")
    implementation("androidx.games:games-activity:4.4.0")
}

for ((variant, flags) in listOf("debug" to listOf(), "release" to listOf("--release"))) {
    val name = variant.replaceFirstChar { it.uppercase() }
    val cargo = tasks.register<Exec>("cargo$name") {
        group = "build"
        description = "Builds libobusina_android.so of every ABI with cargo-ndk."
        workingDir = workspace
        val output = layout.buildDirectory.dir("rust/$variant").get().asFile
        commandLine(
            listOf("cargo", "ndk", "--platform", "$minimum", "--output-dir", output.path) +
                abis.flatMap { listOf("--target", it) } +
                listOf("build", "--package", "obusina-android") + flags,
        )
    }
    tasks.matching { it.name == "merge${name}JniLibFolders" }.configureEach { dependsOn(cargo) }
}
