fn main() {
    println!("cargo:rerun-if-env-changed=SKIA_DIR");
    println!("cargo:rerun-if-env-changed=SKIA_LIB_DIR");

    println!("cargo:rerun-if-changed=skia-c/skia_c.cpp");
    println!("cargo:rerun-if-changed=skia-c/skia_c.hpp");

    #[cfg(target_os = "windows")]
    {
        std::env::set_var("CC", "clang-cl");
        std::env::set_var("CXX", "clang-cl");
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::env::set_var("CC", "clang");
        std::env::set_var("CXX", "clang++");
    }

    let skia_dir = std::env::var("SKIA_DIR").unwrap();
    let skia_path = std::path::Path::new(&skia_dir);

    let mut build = cc::Build::new();

    build.cpp(true);
    build.file("skia-c/skia_c.cpp");
    build.include("skia-c");
    build.include(skia_path);

    #[cfg(target_os = "windows")]
    {
        build.flag("/std:c++17");
        build.flag("-Wno-unused-function");
        build.flag("-Wno-unused-parameter");
    }

    #[cfg(not(target_os = "windows"))]
    {
        build.flag("-std=c++17");
        build.flag("-fPIC");
        build.flag("-fno-exceptions");
        build.flag("-fno-rtti");
        build.flag("-fstrict-aliasing");
        build.flag("-fvisibility=hidden");
        build.flag("-fdata-sections");
        build.flag("-ffunction-sections");
        build.flag("-fvisibility-inlines-hidden");
        build.flag("-Wno-unused-function");
        build.flag("-Wno-unused-parameter");
    }

    build.compile("libskiac.a");

    let skia_lib_dir = std::env::var("SKIA_LIB_DIR").expect("SKIA_LIB_DIR is not set");
    println!("cargo:rustc-link-search={}", skia_lib_dir);
    println!("cargo:rustc-link-lib=skia");
}
