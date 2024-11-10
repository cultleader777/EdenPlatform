use std::fmt::Write;
use crate::{
    database::{Database, TableRowPointerRegion},
    static_analysis::server_runtime::{NomadJob, NomadJobStage, ServerRuntime, ReplacementMacro, JobUpdateStrategy, NomadJobUpdateExplicitStrategy, NomadUpdateHealthCheckStrategy},
};

pub fn schedule_nomad_jobs(
    database: &Database,
    region: TableRowPointerRegion,
    runtime: &mut ServerRuntime,
    stage: NomadJobStage,
) -> String {
    let mut res = String::with_capacity(256);
    res += r#"#!/bin/sh

# if token is passed in somehow, unset it
unset VAULT_TOKEN

# wait for nomad hosts to be available
while ! ping -c 1 nomad-servers.service.consul
do
  sleep 1
done

# wait for nomad rpc port to open
while ! nc -z nomad-servers.service.consul 4646
do
  sleep 1
done

"#;

    struct Arg {
        vp_name: Option<String>,
        job_file_name: String,
        job_src: String,
        replacement_macros: Vec<ReplacementMacro>,
    }

    let mut args = Vec::new();
    for (nj_name, job_data) in runtime.nomad_jobs(region) {
        if job_data.job_stage() == stage {
            let namespace = database.nomad_namespace().c_namespace(job_data.job_namespace());
            args.push(Arg {
                vp_name: job_data
                    .vault_policy()
                    .as_ref()
                    .map(|i| i.vault_policy_name().to_string()),
                job_file_name: format!("{namespace}_{nj_name}.hcl"),
                job_src: generate_single_nomad_job(database, job_data),
                replacement_macros: job_data.replacement_macros().clone(),
            });
        }
    }

    for arg in args {
        let job_file_path = runtime.add_provisioning_resource(
            region,
            "nomad-jobs",
            arg.job_file_name.clone(),
            arg.job_src.clone(),
            false,
            arg.replacement_macros,
        );

        let is_app = job_file_path.path().contains("_app-");

        if is_app {
            res += "if ! grep '@@EPL_' ";
            res += job_file_path.path();
            res += "\nthen\n  ";
        }
        for vp in &arg.vp_name {
            res += "VAULT_TOKEN=$( cat /run/secdir/epl-job-tokens/";
            res += vp;
            res += " ) ";
        }
        res += "nomad job run -detach ";
        res += job_file_path.path();
        res += "\n";
        if is_app {
            res += "else\n";
            res += "  echo Nomad job ";
            res += job_file_path.path();
            res += " image build failed\n";
            res += "fi\n";
        }
    }

    res
}

fn job_update_data_from_strat(strat: &JobUpdateStrategy) -> NomadJobUpdateExplicitStrategy {
    let res =
        match strat {
            // uncomment if will ever be needed
            //JobUpdateStrategy::Custom(strat) => {
            //    strat.clone()
            //}
            // do everything at once for minio
            JobUpdateStrategy::InstantAllAtOnce => {
                NomadJobUpdateExplicitStrategy {
                    max_parallel: 0,
                    health_check: crate::static_analysis::server_runtime::NomadUpdateHealthCheckStrategy::Checks,
                    min_healthy_time_seconds: 0,
                    auto_revert: false,
                    healthy_deadline_seconds: 300,
                    progress_deadline_seconds: 600,
                    stagger_seconds: 30,
                }
            }
            // go slowly and avoid downtime
            JobUpdateStrategy::RollingDefault => {
                NomadJobUpdateExplicitStrategy {
                    max_parallel: 1,
                    health_check: crate::static_analysis::server_runtime::NomadUpdateHealthCheckStrategy::Checks,
                    min_healthy_time_seconds: 30,
                    auto_revert: false,
                    healthy_deadline_seconds: 300,
                    progress_deadline_seconds: 600,
                    stagger_seconds: 30,
                }
            }
        };

    assert!(res.stagger_seconds > 0);

    res
}

fn generate_job_update_block(strat: &JobUpdateStrategy, res: &mut String) {
    let update_data = job_update_data_from_strat(strat);

    let health_check =
        match &update_data.health_check {
            NomadUpdateHealthCheckStrategy::Checks => "checks",
            //NomadUpdateHealthCheckStrategy::TaskStates => "task_states",
        };

    *res += "  update {\n";
    writeln!(res, "    auto_revert = {}", update_data.auto_revert).unwrap();
    writeln!(res, "    max_parallel = {}", update_data.max_parallel).unwrap();
    writeln!(res, "    health_check = \"{}\"", health_check).unwrap();
    writeln!(res, "    min_healthy_time = \"{}s\"", update_data.min_healthy_time_seconds).unwrap();
    writeln!(res, "    stagger = \"{}s\"", update_data.stagger_seconds).unwrap();
    writeln!(res, "    healthy_deadline = \"{}s\"", update_data.healthy_deadline_seconds).unwrap();
    writeln!(res, "    progress_deadline = \"{}s\"", update_data.progress_deadline_seconds).unwrap();
    *res += "  }\n";
}

fn generate_single_nomad_job(database: &Database, job_data: &NomadJob) -> String {
    let mut res = String::with_capacity(256);

    res += "job \"";
    res += job_data.job_name();
    res += "\" {\n";

    match job_data.job_kind() {
        crate::static_analysis::server_runtime::NomadJobKind::BoundStateful => {
            res += "  type = \"service\"\n";
        }
        crate::static_analysis::server_runtime::NomadJobKind::Stateless => {
            res += "  type = \"service\"\n";
        }
        crate::static_analysis::server_runtime::NomadJobKind::SystemStateless => {
            res += "  type = \"system\"\n";
        }
    }
    let ns = job_data.job_namespace();
    res += "  namespace = \"";
    res += database.nomad_namespace().c_namespace(ns);
    res += "\"\n";
    // default until DCs are simulated

    let region = job_data.region();
    res += "  region = \""; res += database.region().c_region_name(region); res += "\"\n";
    res += "  datacenters = [";
    let dc_count = database.region().c_referrers_datacenter__region(region).len();
    for (idx, dc) in database.region().c_referrers_datacenter__region(region).iter().enumerate() {
        let is_last = idx + 1 >= dc_count;
        res += "\"";
        res += database.datacenter().c_dc_name(*dc);
        res += "\"";
        if !is_last {
            res += ", ";
        }
    }
    res += "]\n";

    for vp in job_data.vault_policy() {
        res += &format!(
            r#"
  vault {{
    policies = ["{}"]
  }}
"#,
            vp.vault_policy_name()
        );
    }

    let update_strategy = job_data.update_strategy().as_ref().unwrap_or(&JobUpdateStrategy::RollingDefault);
    generate_job_update_block(update_strategy, &mut res);
    res += "\n";

    for (tgn, tg) in job_data.task_groups() {
        res += "  group \"";
        res += tgn;
        res += "\" {\n";

        writeln!(&mut res, "    count = {}", tg.count()).unwrap();
        writeln!(&mut res, "    shutdown_delay = \"{}s\"", tg.shutdown_delay_seconds()).unwrap();

        if let Some(ac) = tg.architecture_constraint() {
            let n = ac.to_nomad_name();
            write!(&mut res, r#"
    constraint {{
        attribute = "${{attr.kernel.arch}}"
        value     = "{n}"
    }}
"#).unwrap()
        }

        // Make it default to run on distinct hosts
        // if group count more than 1
        if tg.count() > 1 {
            res += r#"
    constraint {
        operator  = "distinct_hosts"
        value     = "true"
    }

"#;
        }

        if let Some(llock) = tg.label_lock() {
            res += "    constraint {\n";
            res += "      attribute = \"${meta.lock_";
            res += llock.label_name();
            res += "}\"\n";
            res += "      operator  = \">\"\n";
            res += "      value     = \"0\"\n";
            res += "    }\n";
        }

        if let Some(placement) = tg.placement() {
            for (key, value) in &placement.match_keys_and_values {
                res += "    constraint {\n";
                res += "      attribute = \"${meta.label_";
                res += key;
                res += "}\"\n";
                res += "      value     = \"";
                res += value;
                res += "\"\n";
                res += "    }\n";
            }
        } else {
            // if job is not assigned to run anywhere
            // and has no label lock only schedule on nodes
            // where unassigned workload is allowed
            if tg.label_lock().is_none() {
                res += "    constraint {\n";
                res += "      attribute = \"${meta.run_unassigned_workloads}\"\n";
                res += "      operator  = \">\"\n";
                res += "      value     = \"0\"\n";
                res += "    }\n";
            }
        }

        if !tg.ports().is_empty() {
            res += "    network {
      mode = \"host\"
";
            for (pv, lp) in tg.ports() {
                res += "      port \"";
                res += pv;
                res += "\" {\n";
                res += "        static = ";
                res += &lp.value().to_string();
                res += "\n";
                // TODO: host network generate
                res += "        host_network = \"lan\"\n";
                res += "      }\n";
            }

            res += "    }\n";
        }

        let mut vol_count = 0;
        for task in tg.tasks().values() {
            for hv in task.used_host_volumes() {
                vol_count += 1;

                res += "\n";
                res += "    volume \"v_";
                res += &vol_count.to_string();
                res += "\" {\n";
                res += "      type = \"host\"\n";
                res += "      source = \"";
                res += hv.lock().nomad_host_volume_name();
                res += "\"\n";
                res += "      read_only = ";
                res += &hv.lock().is_read_only().to_string();
                res += "\n";
                res += "    }\n";
            }
        }

        if !tg.services().is_empty() {
            for service in tg.services().values() {
                res += "\n";
                res += "    service {\n";
                res += "      name = \"";
                res += service.handle().service_name();
                res += "\"\n";
                res += "      port = \"";
                res += service.port();
                res += "\"\n";
                res += "      address = \"${meta.private_ip}\"\n";
                if let Some(mc) = service.metrics_collector() {
                    res += "      tags = [\"epl-mon-";
                    res += database.monitoring_cluster().c_cluster_name(mc.cluster);
                    res += "\"]\n";
                    res += "      meta {\n";
                    res += "        metrics_path = \"";
                    res += &mc.path;
                    res += "\"\n";
                    res += "      }\n";
                }
                match service.healthcheck() {
                    crate::static_analysis::server_runtime::ServiceHealthcheck::Tcp => {
                        res += "      check {\n";
                        res += "        type = \"tcp\"\n";
                        res += "        port = \"";
                        res += service.port();
                        res += "\"\n";
                        res += "        interval = \"10s\"\n";
                        res += "        timeout = \"2s\"\n";
                        res += "      }\n";
                    }
                    crate::static_analysis::server_runtime::ServiceHealthcheck::Http { path } => {
                        res += "      check {\n";
                        res += "        type = \"http\"\n";
                        res += "        port = \"";
                        res += service.port();
                        res += "\"\n";
                        res += "        path = \"";
                        res += path;
                        res += "\"\n";
                        res += "        interval = \"10s\"\n";
                        res += "        timeout = \"2s\"\n";
                        res += "      }\n";
                    }
                }
                res += "    }\n";
            }
        }

        vol_count = 0;
        for (tn, task) in tg.tasks() {
            res += "\n";

            res += "    task \"";
            res += tn;
            res += "\" {\n";
            res += "      driver = \"docker\"\n";
            res += "      resources {\n";
            res += "        memory = ";
            res += &task.used_memory_mb().to_string();
            res += "\n";
            res += "        memory_max = ";
            res += &(task.used_memory_mb() + task.memory_oversubscription_mb() as u64).to_string();
            res += "\n";
            res += "      }\n";

            // TODO: will be needed since nomad 1.9 as consul identities appear
            //if !task.consul_configs().is_empty() {
            //    res += "      identity {\n";
            //    res += "        name = \"consul_default\"\n";
            //    res += "      }\n";
            //}

            for lc in task.lifecycle() {
                let phase = match lc.phase() {
                    crate::static_analysis::server_runtime::NomadTaskLifecyclePhase::PostStart => {
                        "poststart"
                    }
                };
                res += "      lifecycle {\n";
                res += &format!("        sidecar = {}\n", lc.is_sidecar());
                res += &format!("        hook = \"{}\"\n", phase);
                res += "      }\n";
            }

            if !task.env_variables().is_empty() {
                res += "      env {\n";
                for (evn, evl) in task.env_variables() {
                    res += "        ";
                    res += evn;
                    res += " = \"";
                    res += evl;
                    res += "\"\n";
                }
                res += "      }\n";
            }

            res += "      config {\n";
            res += "        image = \"";
            res += task.docker_image();
            res += "\"\n";
            res += "        network_mode = \"host\"\n";
            if !task.entrypoint().is_empty() {
                res += "        entrypoint = [\n";
                for ep in task.entrypoint() {
                    assert!(!ep.contains('\"'));
                    res += "          \"";
                    res += ep;
                    res += "\",\n";
                }
                res += "        ]\n";
            }
            if !task.arguments().is_empty() {
                res += "        args = [\n";
                for ta in task.arguments() {
                    assert!(!ta.contains('\"'));
                    res += "          \"";
                    res += ta;
                    res += "\",\n";
                }
                res += "        ]\n";
            }

            if let Some(cluster) = job_data.loki_cluster() {
                res += "        labels {\n";
                res += "          epl_loki_cluster = \"";
                res += database.loki_cluster().c_cluster_name(cluster);
                res += "\"\n";
                res += "        }\n";
            }

            res += "      }\n";

            for hv in task.used_host_volumes() {
                vol_count += 1;
                res += "\n";
                res += "      volume_mount {\n";
                res += "        volume = \"v_";
                res += &vol_count.to_string();
                res += "\"\n";
                res += "        destination = \"";
                res += hv.target_path();
                res += "\"\n";
                res += "      }\n";
            }

            for (scn, sc) in task.consul_configs() {
                res += "\n";
                res += "      template {\n";
                res += "        destination = \"secrets/";
                res += scn;
                res += "\"\n";
                res += "        perms = \"";
                res += sc.perms();
                res += "\"\n";
                if let Some(cs) = sc.change_script() {
                    res += "        change_mode = \"script\"\n";
                    res += "        change_script {\n";
                    res += "          command = \"";
                    res += cs;
                    res += "\"\n";
                    res += "        }\n";
                }
                res += "        data = \"";
                res += "{{ key \\\"";
                res += sc.kv_path();
                res += "\\\" }}\"\n";
                res += "      }\n";
            }

            for (scn, sc) in task.secure_configs() {
                assert!(!sc.contents().contains("EOL"));

                res += "\n";
                res += "      template {\n";
                res += "        destination = \"secrets/";
                res += scn;
                res += "\"\n";
                res += "        perms = \"";
                res += sc.perms();
                res += "\"\n";
                if sc.eval_as_env() {
                    res += "        env = true\n";
                }
                match sc.file_change_mode() {
                    crate::static_analysis::server_runtime::TaskFileChangeMode::RestartTask => {}
                    crate::static_analysis::server_runtime::TaskFileChangeMode::ReloadSignal { signal } => {
                        res += "        change_mode = \"signal\"\n";
                        let signal = match signal {
                            crate::static_analysis::server_runtime::ChangeSignal::SIGHUP => "SIGHUP",
                        };
                        res += "        change_signal = \"";
                        res += signal;
                        res += "\"\n";
                    }
                }
                res += "        data = <<EOL\n";
                res += sc.contents();
                res += "EOL\n";
                res += "      }\n";
            }

            for (lfn, lfc) in task.local_files() {
                assert!(!lfc.contents().contains("EOF"));

                res += "\n";
                res += "      template {\n";
                res += "        destination = \"local/";
                res += lfn;
                res += "\"\n";
                res += "        perms = \"";
                res += lfc.perms();
                res += "\"\n";
                res += "        data = <<EOL\n";
                res += lfc.contents();
                res += "EOL\n";
                res += "      }\n";
            }

            res += "    }\n";
        }

        res += "  }\n";
        res += "\n";
    }

    res += "}\n";

    if res.contains("{{ with secret") && job_data.vault_policy().is_none() {
        panic!(
            "Job {} uses secrets but does not have vault policy assigned",
            job_data.job_name()
        )
    }
    res
}
