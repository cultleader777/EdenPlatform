use crate::{static_analysis::CheckedDB, codegen::nixplan::NixAllServerPlans};

mod aws;
mod gcloud;

pub fn provision_cloud_specific(db: &CheckedDB, plans: &mut NixAllServerPlans) {
    aws::provision_aws(db, plans);
    gcloud::provision_gcloud(db, plans);
}
