use crate::{static_analysis::server_runtime::{ServerRuntime, ProvisioningScriptTag}, database::TableRowPointerRegion};


pub fn generate_consul_resources(runtime: &mut ServerRuntime, region: TableRowPointerRegion) {
    let mut res = String::new();

    res += "#!/bin/sh\n";
    res += "\n";

    let entries = runtime.consul_kv_entries(region);

    for (k, v) in entries {
        let enc = base64::encode(v.content());
        let enc_bytes = zstd::encode_all(v.content().as_bytes(), 7).unwrap();
        let comp_len = enc_bytes.len();
        assert!(comp_len < 450 * 1024, "Consul KV max size compressed is 512KB, got [{comp_len}] which is close or above limit, shard your KV entry!");
        res += "ORIGINAL_B64=$( echo -n ";
        res += &enc;
        res += " )\n";
        res += "CURRENT_B64=$( consul kv get ";
        res += k;
        res += " | gunzip | base64 -w 0 || true )\n";
        res += "if [ \"$ORIGINAL_B64\" != \"$CURRENT_B64\" ]\n";
        res += "then\n";
        // no gzip library in rust, zstd might not be available in docker container.
        // eventually we'll build our docker containers with standard utilities.
        res += "  THE_VALUE=$( echo -n $ORIGINAL_B64 | base64 -d | gzip -9 | base64 -w 0 )\n";
        res += "  consul kv put -base64 ";
        res += k;
        res += " $THE_VALUE";
        res += "\n";
        res += "else\n";
        res += "  echo ";
        res += k;
        res += " is up to date\n";
        res += "fi\n";
    }

    runtime.add_provisioning_script(
        region,
        ProvisioningScriptTag::L1Resources,
        "provision-consul-resources.sh",
        res,
    );
}
