set -e

# in this day and age swap has literally no value
function check_zfs_root() {
    ZFS_ROOT_DISK_ID=$1

    EXPECTED_DATASET='tank/system/root'
    MOUNTPOINT=$( df | grep -E "^$EXPECTED_DATASET " | awk '{print $6}' )
    if [ "$MOUNTPOINT" != "/" ]
    then
        echo "Expected dataset $EXPECTED_DATASET is not mounted at root mountpoint / printing df output"
        df -h
        exit 7
    fi

    ROOT_ZPOOL=tank
    ZPOOL_STATUS=$( zpool status -P $ROOT_ZPOOL )
    ROOT_DISK_DEV_COUNT=$( echo "$ZPOOL_STATUS" | grep '/dev/' | wc -l )
    if [ "$ROOT_DISK_DEV_COUNT" != "1" ]
    then
        echo "Expected only single disk to be in root zpool, found $ROOT_DISK_DEV_COUNT"
        zpool status -P $ROOT_ZPOOL
        exit 7
    fi

    ROOT_ZPOOL='tank'
    ROOT_DISK=$( echo "$ZPOOL_STATUS" |\
       grep '/dev/' |\
       awk '{print $1}' |\
       sed -E 's/-part[0-9]+$//' |\
       xargs readlink -f |\
       xargs lsblk -no pkname |\
       sort | uniq | sed '/^$/d' ) # remove dupes and empty lines, lsblk returns three rows?
    if [ "$ZFS_ROOT_DISK_ID" != "$ROOT_DISK" ]
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
    if [[ ! "$DEV_NAME" == /dev/* ]]
    then
        DEV_NAME=/dev/$DEV_NAME
    fi
    readlink -f $DEV_NAME
}

function check_expected_disk() {
    DEV_NAME=$( dereference_disk_link $1 )
    EXPECTED_DEV_SIZE=$2
    EXPECTED_DEV_MEDIUM=$3

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

# preconditions
check_zfs_root vda
check_server_disabled_swap
check_server_architecture x86_64
check_core_count 2
check_at_least_memory_bytes 2684354560
check_iface_has_ip enp1s0 10.17.1.12/24 server-l lan
check_expected_disk vda 21474836480 ssd