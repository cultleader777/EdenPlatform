use crate::{static_analysis::CheckedDB, codegen::nixplan::NixAllServerPlans};

pub fn provision_aws(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    // add nftables rules for forwarding to nat forward to internet for hosts
    for dc in db.db.datacenter().rows_iter() {
        if db.db.datacenter().c_implementation(dc) == "aws" {
            for server in db.db.datacenter().c_referrers_server__dc(dc) {
                let plan = plans.fetch_plan(*server);
                plan.add_nix_package("awscli2");
            }
        }
    }
}
