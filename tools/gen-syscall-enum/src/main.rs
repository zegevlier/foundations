use regex::RegexBuilder;
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;

const GLIBC_REPO: &str = "https://sourceware.org/git/glibc.git";
const DST_DIR_REL_PATH: &str = "../../bedrock/src/seccomp/syscalls";

const SYSCALL_SRC_DST: &[(&str, &str)] = &[
    (
        "sysdeps/unix/sysv/linux/aarch64/arch-syscall.h",
        "aarch64.rs",
    ),
    (
        "sysdeps/unix/sysv/linux/x86_64/64/arch-syscall.h",
        "x86_64.rs",
    ),
];

fn main() {
    let dst_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join(DST_DIR_REL_PATH)
        .canonicalize()
        .unwrap();

    let glibc_dir = fetch_glibc_sources();

    for (glibc_header, out_rs) in SYSCALL_SRC_DST {
        let syscall_list = get_syscall_list(glibc_dir.as_path().join(glibc_header));

        gen_rs_file(dst_dir.as_path().join(out_rs), syscall_list);
    }
}

fn fetch_glibc_sources() -> PathBuf {
    println!("Fetching glibc sources...");
    println!("=========================");

    let target_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("target");
    let glibc_dir = target_dir.as_path().join("glibc");

    let _ = fs::remove_dir_all(&glibc_dir);

    Command::new("git")
        .args(["clone", "--depth=1", GLIBC_REPO])
        .current_dir(target_dir)
        .spawn()
        .unwrap()
        .wait()
        .unwrap();

    glibc_dir
}

fn get_syscall_list(header_file: PathBuf) -> Vec<(String, String)> {
    let mut header_data = "".into();

    fs::File::open(header_file)
        .unwrap()
        .read_to_string(&mut header_data)
        .unwrap();

    RegexBuilder::new(r"^#define\s+__NR_(?P<syscall>\S+)\s+(?P<id>\d+)$")
        .multi_line(true)
        .build()
        .unwrap()
        .captures_iter(&header_data)
        .map(|c| {
            (
                c.name("syscall").unwrap().as_str().to_string(),
                c.name("id").unwrap().as_str().to_string(),
            )
        })
        .collect()
}

fn gen_rs_file(out_rs: PathBuf, syscall_list: Vec<(String, String)>) {
    let mut out = fs::File::create(out_rs).unwrap();

    writeln!(out, "// AUTOGENERATED by tools/gen-syscall-enum\n").unwrap();
    writeln!(out, "/// Linux syscalls.").unwrap();
    writeln!(out, "#[allow(non_camel_case_types)]").unwrap();
    writeln!(out, "pub enum Syscall {{").unwrap();

    for (syscall, id) in syscall_list {
        writeln!(
            out,
            "    /// <https://man7.org/linux/man-pages/man2/{syscall}.2.html>"
        )
        .unwrap();

        writeln!(out, "    {syscall} = {id},").unwrap();
    }

    writeln!(out, "}}").unwrap();
}
