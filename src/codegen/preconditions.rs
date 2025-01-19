use std::borrow::Cow;

use crate::{static_analysis::{CheckedDB, networking::DcParameters, server_disks::{pick_disk_id_policy, DiskIdsPolicy, pick_absolute_disk_path_by_policy}}, database::{TableRowPointerServer, Database, TableRowPointerServerDisk}, codegen::l1_provisioning::utils::epl_arch_to_linux_arch};

pub fn generate_server_preconditions(checked: &CheckedDB, server: TableRowPointerServer) -> String {
    use std::fmt::Write;

    let dc = checked.db.server().c_dc(server);
    let dc_impl = checked.db.datacenter().c_implementation(dc);
    let disk_id_policy = pick_disk_id_policy(&checked.db, dc);
    // damn aws... hard disk can be seen as nvme, wtf wtf wtf...
    let is_aws = dc_impl == "aws";
    let is_coprocessor = dc_impl == "coprocessor";

    // TODO: add CPU/RAM checks
    let mut res = "set -e\n".to_string();
    let server_kind = checked.projections.server_kinds.value(server);
    let memory_bytes = checked.db.server_kind().c_memory_bytes(*server_kind);
    // at least 95% advertised memory, or at least 512MB difference memory specs not precise in vms
    let need_at_least = ((memory_bytes * 9) / 10).min(memory_bytes - 512 * 1024 * 1024);
    let core_count = checked.db.server_kind().c_cores(*server_kind);
    let arch = checked.db.server_kind().c_architecture(*server_kind);
    let checked_server_arch = epl_arch_to_linux_arch(&arch);
    let root_disk = checked.db.server().c_root_disk(server);
    let root_disk_id = checked.db.server_disk().c_disk_id(root_disk);
    let nixos_env = checked.db.nixpkgs_environment().c_version(checked.db.server().c_nixpkgs_environment(server));
    let nixos_v = checked.db.nixpkgs_version().c_version(nixos_env);
    // we don't need to check because we set channel in the beginning
    // and this allows for upgrades
    let check_nixos_version = false;
    res.push_str(preconditions_utility_functions());
    res.push_str("\n");
    res.push_str("# preconditions\n");

    let required_commands = [
        "tmux", // run session
        "git", // version control
        "sqlite3", // provisioning state
        "lockfile", // lockfile
        "gzip", // unzip command stream
    ];

    if check_nixos_version {
        writeln!(&mut res, "check_nixos_version {nixos_v}").unwrap();
    }

    for rc in required_commands {
        writeln!(&mut res, "check_required_command {rc}").unwrap();
    }

    writeln!(&mut res, "check_zfs_root {root_disk_id}").unwrap();
    writeln!(&mut res, "check_server_disabled_swap").unwrap();
    writeln!(&mut res, "check_server_architecture {checked_server_arch}").unwrap();
    writeln!(&mut res, "check_core_count {core_count}").unwrap();
    writeln!(&mut res, "check_at_least_memory_bytes {need_at_least}").unwrap();

    let dc = checked.db.server().c_dc(server);
    let dc_net = checked.sync_res.network.networking_answers.dcs.get(&dc).unwrap();
    let hostname = checked.db.server().c_hostname(server);
    for ni in checked.db.server().c_children_network_interface(server) {
        let iface_name = if checked.db.network_interface().c_if_vlan(*ni) < 0 {
           Cow::from(checked.db.network_interface().c_if_name(*ni))
        } else { Cow::from(format!("vlan{}", checked.db.network_interface().c_if_vlan(*ni))) };
        let is_subinterface = iface_name.contains(":");
        if is_subinterface || is_coprocessor {
            // we configure subinterfaces but don't expect them to be there on first run
            continue;
        }
        let network = checked.db.network_interface().c_if_network(*ni);
        let iface_ip = checked.db.network_interface().c_if_ip(*ni);
        let iface_cidr = checked.db.network_interface().c_if_prefix(*ni);
        let addr_full = format!("{iface_ip}/{iface_cidr}");
        let network_name = checked.db.network().c_network_name(network).as_str();
        match network_name {
            "internet" if !dc_net.params.are_public_ips_hidden => {
                writeln!(&mut res, r#"check_iface_has_ip {iface_name} {addr_full} {hostname} {network_name}"#).unwrap();
            }
            "internet" if dc_net.params.are_public_ips_hidden => {}
            "lan" => {
                writeln!(&mut res, r#"check_iface_has_ip {iface_name} {addr_full} {hostname} {network_name}"#).unwrap();
            }
            "vpn" => {}
            "dcrouter" => {
                writeln!(&mut res, r#"check_iface_has_ip {iface_name} {addr_full} {hostname} {network_name}"#).unwrap();
            }
            other => panic!("Unexpected network interface {other}")
        }
    }

    for disk in checked.db.server().c_children_server_disk(server) {
        let disk_id = checked.db.server_disk().c_disk_id(*disk);
        let disk_kind = checked.db.server_disk().c_disk_kind(*disk);
        let disk_size = checked.projections.server_disk_sizes.get(disk).unwrap();
        let disk_medium = checked.db.disk_kind().c_medium(disk_kind);
        let disk_label = disk_id_label(&checked.db, *disk, &dc_net.params, &disk_id_policy);
        // damn aws...
        if is_aws && disk_medium == "hdd" && disk_id.starts_with("nvme") {
            writeln!(&mut res, "check_expected_disk {disk_label} {disk_size} nvme").unwrap();
        } else {
            // normal path
            writeln!(&mut res, "check_expected_disk {disk_label} {disk_size} {disk_medium}").unwrap();
        }
    }

    for disk in checked.db.server().c_children_server_disk(server) {
        if checked.db.server_disk().c_xfs_format(*disk) {
            let disk_label = disk_id_label(&checked.db, *disk, &dc_net.params, &disk_id_policy);
            writeln!(&mut res, "check_xfs_disk {disk_label}").unwrap();
        }
    }

    res
}

pub fn disk_id_label(db: &Database, disk: TableRowPointerServerDisk, dc_params: &DcParameters, pol: &DiskIdsPolicy) -> String {
    let disk_id = db.server_disk().c_disk_id(disk);
    if let Some(tr) = &dc_params.disk_id_transform {
        tr.replace("DISK_ID", &disk_id)
    } else {
        pick_absolute_disk_path_by_policy(db, disk, pol)
    }
}

fn preconditions_utility_functions() -> &'static str {
    r#"
# check required packages for the very first provision
function check_nixos_version() {
    EXPECTED_VERSION=$1
    if ! which nixos-version > /dev/null
    then
      echo nixos-version command not found, not running NixOS?
      exit 7
    fi

    ACTUAL_VERSION=$( nixos-version )
    if ! echo "$ACTUAL_VERSION" | grep -E "^$EXPECTED_VERSION" > /dev/null
    then
      echo Unexpected NixOS version, expected $EXPECTED_VERSION, found $ACTUAL_VERSION for this server
      exit 7
    fi
}

# check required packages for the very first provision
function check_required_command() {
    if ! which $1 > /dev/null
    then
      echo Required command $1 not found in path, exiting
      exit 7
    fi
}

# in this day and age swap has literally no value
function check_zfs_root() {
    ZFS_ROOT_DISK_ID=$1

    EXPECTED_DATASET='rpool/root'
    MOUNTPOINT=$( df | grep -E "^$EXPECTED_DATASET " | awk '{print $6}' )
    if [ "$MOUNTPOINT" != "/" ]
    then
        echo "Expected dataset $EXPECTED_DATASET is not mounted at root mountpoint / printing df output"
        df -h
        exit 7
    fi

    ROOT_ZPOOL=rpool
    ZPOOL_STATUS=$( zpool status -P $ROOT_ZPOOL )
    ROOT_DISK_DEV_COUNT=$( echo "$ZPOOL_STATUS" | grep '/dev/' | wc -l )
    if [ "$ROOT_DISK_DEV_COUNT" != "1" ]
    then
        echo "Expected only single disk to be in root zpool, found $ROOT_DISK_DEV_COUNT"
        zpool status -P $ROOT_ZPOOL
        exit 7
    fi

    ROOT_ZPOOL='rpool'
    ROOT_DISK=$( echo "$ZPOOL_STATUS" |\
       grep '/dev/' |\
       awk '{print $1}' |\
       sed -E 's/-part[0-9]+$//' |\
       xargs readlink -f |\
       xargs lsblk -no pkname |\
       sort | uniq | sed '/^$/d' ) # remove dupes and empty lines, lsblk returns three rows?
    DEREFFED_ROOT_DISK=$( dereference_disk_link $ROOT_DISK )
    DEREFFED_ZFS_ROOT_DISK=$( dereference_disk_link $ZFS_ROOT_DISK_ID )
    if [ "$DEREFFED_ROOT_DISK" != "$DEREFFED_ZFS_ROOT_DISK" ]
    then
        echo "Unexpected root disk id for root zpool of $ROOT_ZPOOL, expected [$ZFS_ROOT_DISK_ID] actual [$ROOT_DISK]"
        zpool status -P $ROOT_ZPOOL
        exit 7
    fi
}

function check_core_count() {
    EXPECTED_COUNT=$1
    ACTUAL_COUNT=$( nproc )
    if [ "$EXPECTED_COUNT" -ne "$ACTUAL_COUNT" ]
    then
        echo "Machine cores expected to be $EXPECTED_COUNT, actual $ACTUAL_COUNT"
        exit 7
    fi
}

function check_at_least_memory_bytes() {
    AT_LEAST=$1
    MACHINE_MEMORY_BYTES=$( free -b | grep -E 'Mem:' | awk '{print $2}' )
    if [ "$MACHINE_MEMORY_BYTES" -lt "$AT_LEAST" ]
    then
        echo "Machine RAM expected to be at least $AT_LEAST bytes, actual $MACHINE_MEMORY_BYTES"
        free -b
        exit 7
    fi
}

# needed because in aws we get symlinks for devices
# we intended but they're actually differently named
# devices as nvme
function dereference_disk_link() {
    DEV_NAME=$1
    if [ -e /dev/disk/by-id/$DEV_NAME ]
    then
        DEV_NAME=/dev/disk/by-id/$DEV_NAME
    elif [[ ! "$DEV_NAME" == /dev/* ]]
    then
        DEV_NAME=/dev/$DEV_NAME
    fi
    readlink -f $DEV_NAME
}

function check_expected_disk() {
    DEV_ORIGINAL=$1
    EXPECTED_DEV_SIZE=$2
    EXPECTED_DEV_MEDIUM=$3

    DEV_NAME=$( dereference_disk_link $DEV_ORIGINAL )

    DISK_INFO=$( lsblk -b -n --nodeps -o NAME,SIZE,ROTA $DEV_NAME )
    if [ -z "$DISK_INFO" ]
    then
        echo "Device $DEV_NAME not found in server"
        lsblk -b
        exit 7
    fi

    DISK_SIZE=$( echo "$DISK_INFO" | awk '{print $2}' )
    IS_ROTATIONAL=$( echo "$DISK_INFO" | awk '{print $3}' )

    if [ "$DISK_SIZE" != "$EXPECTED_DEV_SIZE" ]
    then
        echo "Device $DEV_NAME expected to have $EXPECTED_DEV_SIZE, actual $DISK_SIZE"
        lsblk -b
        exit 7
    fi

    if [ "$EXPECTED_DEV_MEDIUM" == "hdd" ] && [ "$IS_ROTATIONAL" != "1" ]
    then
        echo "Device $DEV_NAME is expected to be rotational disk device, but it is not"
        lsblk -o NAME,ROTA --nodeps $DEV_NAME
        exit 7
    elif [ "$EXPECTED_DEV_MEDIUM" == "ssd" ] || [ "$EXPECTED_DEV_MEDIUM" == "nvme" ]
    then
        if [ "$IS_ROTATIONAL" == "1" ]
        then
            # TODO: ssd on libvirt detected as HDD in testvms setup, comment until I figure it out
            #echo "Device $DEV_NAME is expected to be not rotational disk device, but it is"
            #lsblk -o NAME,ROTA --nodeps $DEV_NAME
            #exit 7
            true
        fi
    fi
}

# run this function to see if disk is not xfs formatted at all
# or it is already xfs formatted
function check_xfs_disk() {
    DEV_NAME=$( dereference_disk_link $1 )

    CURR_FS_TYPE=$( lsblk -n -o FSTYPE $DEV_NAME )
    if [ -z "$CURR_FS_TYPE" ]
    then
        # all is good, disk not formatted yet
        true
    elif [ "$CURR_FS_TYPE" != "xfs" ]
    then
        echo Current disk is formatted but fs type for expected disk is not xfs, what is happening?
        lsblk -f $DEV_NAME
        exit 7
    fi
}

# in this day and age swap has literally no value
function check_server_disabled_swap() {
    if ! free -h | grep -E "^Swap:.*0B.*0B.*0B$"
    then
        exit 7
    fi
}

function check_iface_has_ip() {
    IFACE_NAME=$1
    IFACE_IP=$2
    SERVER_NAME=$3
    NETWORK_NAME=$4

    for I in $(seq 1 5);
    do
        ip -f inet addr show $IFACE_NAME | grep -F "inet $IFACE_IP" && return
        sleep 1
    done

    echo Interface $IFACE_NAME for network $NETWORK_NAME with address $IFACE_IP not found on server $SERVER_NAME
    exit 7
}

function check_server_architecture() {
    EXPECTED_ARCH=$1
    ACTUAL_ARCH="$(uname -m)"

    if [ "$ACTUAL_ARCH" != "$EXPECTED_ARCH" ];
    then
        echo Server architecture is expected to be $EXPECTED_ARCH found $ACTUAL_ARCH
        exit 7
    fi
}
"#
}
