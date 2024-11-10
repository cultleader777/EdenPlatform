pub mod aws;
pub mod gcloud;
pub mod bm_simple;

pub fn system_reserved_memory_bytes() -> i64 {
    // 1.0GB
    // real usage on nodes for system.slice (measured by 'systemd-cgtop system.slice') command
    // is around 770MB but we add little extr
    // TODO: docker uses ~256MB memory, can we lower that?
    1024 * 1024 * 1024
}

fn node_eligibility_calculation(node_memory_bytes: i64) -> String {
    assert!(node_memory_bytes > 0);
    let system_reserved_memory_bytes = system_reserved_memory_bytes();
    let minimum_extra_memory = 512 * 1024 * 1024;
    let minimum_total_memory = system_reserved_memory_bytes + minimum_extra_memory;
    if minimum_total_memory > node_memory_bytes {
        return format!(
            "Too little memory to run Eden platform node, node size is {:.1}GB, minimum required is {:.1}GB ({:.1}GB system reserved + {:.1}GB for workloads)",
            bytes_to_gb(node_memory_bytes),
            bytes_to_gb(minimum_total_memory),
            bytes_to_gb(system_reserved_memory_bytes),
            bytes_to_gb(minimum_extra_memory)
        );
    } else { "".to_string() }
}

fn bytes_to_gb(input: i64) -> f64 {
    (input as f64) / 1024.0 / 1024.0 / 1024.0
}
