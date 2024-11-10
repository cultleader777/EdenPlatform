use std::{collections::{BTreeMap, HashMap, BTreeSet}, mem::swap};

use serde::{Deserialize, Serialize};

use crate::database::{TableRowPointerServer, Database, TableRowPointerRegion, TableRowPointerValidServerLabels};

use super::PlatformValidationError;

#[derive(Debug)]
pub struct RegionData {
    server_count: usize,
    region_index: BTreeMap<TableRowPointerValidServerLabels, BTreeMap<String, BTreeSet<TableRowPointerServer>>>,
}

#[derive(Debug)]
pub struct LabelDatabase {
    server_index: BTreeMap<TableRowPointerRegion, RegionData>,
    label_index: HashMap<String, TableRowPointerValidServerLabels>,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LabelQuery {
    pub match_keys_and_values: BTreeMap<String, String>,
}

impl LabelDatabase {
    pub fn find_servers(&self, region: TableRowPointerRegion, lq: &LabelQuery, original_query: Option<&str>) -> Result<BTreeSet<TableRowPointerServer>, PlatformValidationError> {
        let mut res: BTreeSet<TableRowPointerServer> = BTreeSet::new();
        let mut is_initial_run = true;

        let index = self.server_index.get(&region).unwrap();

        for (k, v) in &lq.match_keys_and_values {
            if let Some(lv) = self.label_index.get(k) {
                if let Some(key_idx) = index.region_index.get(lv) {
                    if let Some(servers) = key_idx.get(v) {
                        if !is_initial_run {
                            let mut new_set: BTreeSet<TableRowPointerServer> =
                                servers.intersection(&res).map(|i| *i).collect::<BTreeSet<_>>();
                            swap(&mut res, &mut new_set);
                        } else {
                            res = servers.clone();
                        }
                    }
                }
            } else {
                let placement_query =
                    original_query.map(|i| i.to_string()).unwrap_or_else(|| {
                        serde_yaml::to_string(lq).unwrap()
                    });
                return Err(PlatformValidationError::InvalidServerLabelInQuery {
                    invalid_label_key: k.clone(),
                    label_value: v.clone(),
                    placement_query,
                });
            }
            is_initial_run = false;
            if res.is_empty() {
                break;
            }
        }

        Ok(res)
    }

    pub fn try_to_find_placements(&self, db: &Database, context: &str, region: TableRowPointerRegion, query: &str, need_at_least: usize) -> Result<Option<LabelQuery>, PlatformValidationError> {
        if query.is_empty() {
            let region_data = self.server_index.get(&region).unwrap();
            // check is fast but error generation might be slower, we don't care about it
            if region_data.server_count < need_at_least {
                let found_servers =
                    db.region().c_referrers_datacenter__region(region)
                    .iter()
                    .map(|dc| db.datacenter().c_referrers_server__dc(*dc))
                    .flat_map(|srvs| {
                        srvs.iter().map(|srv| {
                            db.server().c_hostname(*srv).clone()
                        })
                    })
                    .collect::<Vec<String>>();

                return Err(PlatformValidationError::FailedToFindPlacements {
                    context: context.to_string(),
                    need_at_least,
                    found_servers_count: region_data.server_count,
                    found_servers,
                    placement_query: query.to_string(),
                });
            } else {
                return Ok(None);
            }
        }

        assert!(need_at_least > 0, "{need_at_least} should be at least more than 0?");

        let parsed = parse_label_query(query)?;

        let servers = self.find_servers(region, &parsed, Some(query))?;
        if servers.len() < need_at_least {
            return Err(PlatformValidationError::FailedToFindPlacements {
                context: context.to_string(),
                need_at_least,
                found_servers_count: servers.len(),
                found_servers: servers.iter().map(|i| db.server().c_hostname(*i).clone()).collect(),
                placement_query: query.to_string(),
            });
        }

        Ok(Some(parsed))
    }
}

fn parse_label_query(input: &str) -> Result<LabelQuery, PlatformValidationError> {
    serde_yaml::from_str::<LabelQuery>(input).map_err(|e| {
        PlatformValidationError::LabelQueryParseError {
            placement_query: input.to_string(),
            parsing_error: e.to_string()
        }
    })
}

pub fn build_label_database(db: &Database) -> Result<LabelDatabase, PlatformValidationError> {
    let label_index = build_label_index(db);
    let mut server_index = BTreeMap::new();

    for region in db.region().rows_iter() {
        let mut region_index: BTreeMap<TableRowPointerValidServerLabels, BTreeMap<String, BTreeSet<TableRowPointerServer>>> = BTreeMap::new();
        let mut server_count = 0;
        for dc in db.region().c_referrers_datacenter__region(region) {
            server_count += db.datacenter().c_referrers_server__dc(*dc).len();
            for server in db.datacenter().c_referrers_server__dc(*dc) {
                for label in db.server().c_children_server_label(*server) {
                    let valid_label = db.server_label().c_label_name(*label);
                    let value = db.server_label().c_label_value(*label);
                    let this_label_idx = region_index.entry(valid_label).or_default();
                    let srvs = this_label_idx.entry(value.clone()).or_default();
                    assert!(srvs.insert(*server));
                }
            }
        }

        assert!(server_index.insert(region, RegionData { server_count, region_index }).is_none());
    }

    Ok(LabelDatabase { server_index, label_index })
}

fn build_label_index(db: &Database) -> HashMap<String, TableRowPointerValidServerLabels> {
    let mut label_index = HashMap::new();

    for vsl in db.valid_server_labels().rows_iter() {
        assert!(label_index.insert(db.valid_server_labels().c_label_name(vsl).clone(), vsl).is_none());
    }

    label_index
}
