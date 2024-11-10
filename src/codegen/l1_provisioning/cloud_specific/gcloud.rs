use crate::{codegen::nixplan::{NixServerFeatures, NixAllServerPlans}, static_analysis::CheckedDB};


pub fn provision_gcloud(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    // add nftables rules for forwarding to nat forward to internet for hosts
    for dc in db.db.datacenter().rows_iter() {
        if db.db.datacenter().c_implementation(dc) == "gcloud" {
            for server in db.db.datacenter().c_referrers_server__dc(dc) {
                if db.projections.internet_network_iface.get(server).is_some() {
                    let plan = plans.fetch_plan(*server);
                    plan.add_server_feature(NixServerFeatures::Nftables);
                    plan.add_nix_package("google-cloud-sdk");
                }
            }
        }
    }
}
