use std::env;
use std::fs;

pub fn read_meminfo_fields(fields: &[&str]) -> Vec<u64> {
    let mut results = vec![0; fields.len()];
    for line in fs::read_to_string("/proc/meminfo").unwrap().lines() {
        for (i, &field) in fields.iter().enumerate() {
            if line.starts_with(field) {
                results[i] = line.split_whitespace().nth(1).unwrap().parse().unwrap();
            }
        }
    }
    results
}

pub fn format_bytes(b: u64) -> String {
    let b = b as f64;
    if b >= (1u64 << 40) as f64 {
        format!("{:.2} TiB", b / (1u64 << 40) as f64)
    } else if b >= (1u64 << 30) as f64 {
        format!("{:.2} GiB", b / (1u64 << 30) as f64)
    } else if b >= (1u64 << 20) as f64 {
        format!("{:.2} MiB", b / (1u64 << 20) as f64)
    } else if b >= (1u64 << 10) as f64 {
        format!("{:.2} KiB", b / (1u64 << 10) as f64)
    } else {
        format!("{} B", b as u64)
    }
}

pub fn read_file_trim(path: &str) -> String {
    fs::read_to_string(path).unwrap().trim().to_string()
}

pub fn make_separator(len: usize, color: &str) -> String {
    format!("{}{}{}", color, "-".repeat(len), "\x1b[0m")
}

pub fn evod(var: &str, default: &str) -> String {
    env::var(var).unwrap_or(default.to_string())
}
