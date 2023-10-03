//! build.rs for `libafl_targets`

use std::{env, fs::File, io::Write, path::Path};

#[allow(clippy::too_many_lines)]
fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = out_dir.to_string_lossy().to_string();
    //let out_dir_path = Path::new(&out_dir);
    let src_dir = Path::new("src");

    let dest_path = Path::new(&out_dir).join("constants.rs");
    let mut constants_file = File::create(dest_path).expect("Could not create file");

    let edges_map_size: usize = option_env!("LIBAFL_EDGES_MAP_SIZE")
        .map_or(Ok(65536), str::parse)
        .expect("Could not parse LIBAFL_EDGES_MAP_SIZE");
    let cmp_map_size: usize = option_env!("LIBAFL_CMP_MAP_SIZE")
        .map_or(Ok(65536), str::parse)
        .expect("Could not parse LIBAFL_CMP_MAP_SIZE");
    let aflpp_cmplog_map_w: usize = option_env!("LIBAFL_AFLPP_CMPLOG_MAP_W")
        .map_or(Ok(65536), str::parse)
        .expect("Could not parse LIBAFL_AFLPP_CMPLOG_MAP_W");
    let aflpp_cmplog_map_h: usize = option_env!("LIBAFL_AFLPP_CMPLOG_MAP_W")
        .map_or(Ok(32), str::parse)
        .expect("Could not parse LIBAFL_AFLPP_CMPLOG_MAP_W");
    let cmplog_map_w: usize = option_env!("LIBAFL_CMPLOG_MAP_W")
        .map_or(Ok(65536), str::parse)
        .expect("Could not parse LIBAFL_CMPLOG_MAP_W");
    let cmplog_map_h: usize = option_env!("LIBAFL_CMPLOG_MAP_H")
        .map_or(Ok(32), str::parse)
        .expect("Could not parse LIBAFL_CMPLOG_MAP_H");
    let acc_map_size: usize = option_env!("LIBAFL_ACCOUNTING_MAP_SIZE")
        .map_or(Ok(65536), str::parse)
        .expect("Could not parse LIBAFL_ACCOUNTING_MAP_SIZE");

    write!(
        constants_file,
        "// These constants are autogenerated by build.rs

        /// The size of the edges map
        pub const EDGES_MAP_SIZE: usize = {edges_map_size};
        /// The size of the cmps map
        pub const CMP_MAP_SIZE: usize = {cmp_map_size};
        /// The width of the aflpp cmplog map
        pub const AFLPP_CMPLOG_MAP_W: usize = {aflpp_cmplog_map_w};
        /// The height of the aflpp cmplog map
        pub const AFLPP_CMPLOG_MAP_H: usize = {aflpp_cmplog_map_h};
        /// The width of the `CmpLog` map
        pub const CMPLOG_MAP_W: usize = {cmplog_map_w};
        /// The height of the `CmpLog` map
        pub const CMPLOG_MAP_H: usize = {cmplog_map_h};
        /// The size of the accounting maps
        pub const ACCOUNTING_MAP_SIZE: usize = {acc_map_size};
"
    )
    .expect("Could not write file");

    println!("cargo:rerun-if-env-changed=LIBAFL_EDGES_MAP_SIZE");
    println!("cargo:rerun-if-env-changed=LIBAFL_CMP_MAP_SIZE");
    println!("cargo:rerun-if-env-changed=LIBAFL_AFLPP_CMPLOG_MAP_W");
    println!("cargo:rerun-if-env-changed=LIBAFL_AFLPP_CMPLOG_MAP_H");
    println!("cargo:rerun-if-env-changed=LIBAFL_CMPLOG_MAP_W");
    println!("cargo:rerun-if-env-changed=LIBAFL_CMPLOG_MAP_H");
    println!("cargo:rerun-if-env-changed=LIBAFL_ACCOUNTING_MAP_SIZE");

    #[cfg(any(feature = "sancov_value_profile", feature = "sancov_cmplog"))]
    {
        println!("cargo:rerun-if-changed=src/sancov_cmp.c");

        let mut sancov_cmp = cc::Build::new();

        #[cfg(feature = "sancov_value_profile")]
        {
            sancov_cmp.define("SANCOV_VALUE_PROFILE", "1");
            println!("cargo:rerun-if-changed=src/value_profile.h");
        }

        #[cfg(feature = "sancov_cmplog")]
        {
            sancov_cmp.define("SANCOV_CMPLOG", "1");

            println!("cargo:rustc-link-arg=--undefined=__sanitizer_weak_hook_memcmp");
            println!("cargo:rustc-link-arg=--undefined=__sanitizer_weak_hook_strncmp");
            println!("cargo:rustc-link-arg=--undefined=__sanitizer_weak_hook_strncasecmp");
            println!("cargo:rustc-link-arg=--undefined=__sanitizer_weak_hook_strcmp");
            println!("cargo:rustc-link-arg=--undefined=__sanitizer_weak_hook_strcasecmp");
        }

        sancov_cmp
            .define("CMP_MAP_SIZE", Some(&*format!("{cmp_map_size}")))
            .define(
                "AFLPP_CMPLOG_MAP_W",
                Some(&*format!("{aflpp_cmplog_map_w}")),
            )
            .define(
                "AFLPP_CMPLOG_MAP_H",
                Some(&*format!("{aflpp_cmplog_map_h}")),
            )
            .define("CMPLOG_MAP_W", Some(&*format!("{cmplog_map_w}")))
            .define("CMPLOG_MAP_H", Some(&*format!("{cmplog_map_h}")))
            .file(src_dir.join("sancov_cmp.c"))
            .compile("sancov_cmp");

        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_cmp1");
        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_cmp2");
        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_cmp4");
        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_cmp8");

        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_const_cmp1");
        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_const_cmp2");
        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_const_cmp4");
        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_const_cmp8");

        println!("cargo:rustc-link-arg=--undefined=__sanitizer_cov_trace_switch");
    }

    #[cfg(feature = "libfuzzer")]
    {
        println!("cargo:rerun-if-changed=src/libfuzzer.c");

        let mut libfuzzer = cc::Build::new();
        libfuzzer.file(src_dir.join("libfuzzer.c"));

        #[cfg(feature = "libfuzzer_no_link_main")]
        libfuzzer.define("FUZZER_NO_LINK_MAIN", "1");
        #[cfg(feature = "libfuzzer_define_run_driver")]
        libfuzzer.define("FUZZER_DEFINE_RUN_DRIVER", "1");

        libfuzzer.compile("libfuzzer");
    }

    println!("cargo:rerun-if-changed=src/common.h");
    println!("cargo:rerun-if-changed=src/common.c");

    #[cfg(feature = "sanitizer_interfaces")]
    {
        println!("cargo:rerun-if-changed=src/sanitizer_interfaces.h");

        let build = bindgen::builder()
            .header("src/sanitizer_interfaces.h")
            .use_core()
            .generate_comments(true)
            .parse_callbacks(Box::new(bindgen::CargoCallbacks))
            .generate()
            .expect("Couldn't generate the sanitizer headers!");

        build
            .write_to_file(Path::new(&out_dir).join("sanitizer_interfaces.rs"))
            .expect("Couldn't write the sanitizer headers!");
    }

    let mut common = cc::Build::new();

    #[cfg(feature = "sanitizers_flags")]
    {
        common.define("DEFAULT_SANITIZERS_OPTIONS", "1");
    }

    common.file(src_dir.join("common.c")).compile("common");

    println!("cargo:rerun-if-changed=src/coverage.c");

    cc::Build::new()
        .file(src_dir.join("coverage.c"))
        .define("EDGES_MAP_SIZE", Some(&*format!("{edges_map_size}")))
        .define("ACCOUNTING_MAP_SIZE", Some(&*format!("{acc_map_size}")))
        .compile("coverage");

    println!("cargo:rerun-if-changed=src/cmplog.h");
    println!("cargo:rerun-if-changed=src/cmplog.c");

    cc::Build::new()
        .define("CMP_MAP_SIZE", Some(&*format!("{cmp_map_size}")))
        .define(
            "AFLPP_CMPLOG_MAP_W",
            Some(&*format!("{aflpp_cmplog_map_w}")),
        )
        .define(
            "AFLPP_CMPLOG_MAP_H",
            Some(&*format!("{aflpp_cmplog_map_h}")),
        )
        .define("CMPLOG_MAP_W", Some(&*format!("{cmplog_map_w}")))
        .define("CMPLOG_MAP_H", Some(&*format!("{cmplog_map_h}")))
        .file(src_dir.join("cmplog.c"))
        .compile("cmplog");

    #[cfg(unix)]
    {
        println!("cargo:rerun-if-changed=src/forkserver.c");

        cc::Build::new()
            .file(src_dir.join("forkserver.c"))
            .compile("forkserver");
    }

    #[cfg(windows)]
    {
        println!("cargo:rerun-if-changed=src/windows_asan.c");

        cc::Build::new()
            .file(src_dir.join("windows_asan.c"))
            .compile("windows_asan");
    }

    println!("cargo:rustc-link-search=native={}", &out_dir);

    println!("cargo:rerun-if-changed=build.rs");
}
