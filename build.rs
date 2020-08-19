fn main() {
    const SKIA_FILES: &[&str] = &[
        "skia-pipeline/SkOpts.cpp",
        "skia-pipeline/SkRasterPipeline.cpp",
    ];

    for entry in std::fs::read_dir("skia-pipeline").unwrap() {
        if let Ok(entry) = entry {
            println!("cargo:rerun-if-changed={}", entry.path().display());
        }
    }

    // TODO: what about 32bit ARM?
    #[cfg(not(target_pointer_width = "64"))]
    {
        panic!("only 64bit target is supported");
    }

    // Force clang, otherwise there is no point in this feature.
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

    let mut build = cc::Build::new();
    build.cpp(true);
    build.files(SKIA_FILES);
    build.include("skia-pipeline");

    #[cfg(target_os = "windows")]
    {
        // TODO: -march=native
        build.flag("-Wno-unused-parameter");
    }

    #[cfg(not(target_os = "windows"))]
    {
        build.flag("-std=c++11");
        build.flag("-nostdlib");
        build.flag("-fno-exceptions");
        build.flag("-fno-rtti");
        build.flag("-fstrict-aliasing");
        build.flag("-Wno-unused-parameter");

        // Skia built via gn doesn't set it, but without
        // it we're getting huge performance regression.
        // Not sure why.
        #[cfg(feature = "target-cpu")]
        {
            build.flag("-march=native");
        }
    }

    build.compile("libskiac.a"); // name doesn't matter
}
