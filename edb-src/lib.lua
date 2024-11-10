function standard_24_disk_setup(hostname)
  data('server_disk', { hostname = hostname, disk_id = "vda" })
  for i = string.byte('b'), string.byte('z') do
    data('server_disk', { hostname = hostname, disk_id = "vd" .. string.char(i) })
  end
end

function stringStarts(input,start)
   return string.sub(input,1,string.len(start))==start
end

function stringEnds(str, ending)
   return ending == "" or str:sub(-#ending) == ending
end

function isSnakeCase(input)
  return string.match(input,"^[a-z0-9_]+$") ~= nil
end

function isPascalCase(input)
  return string.match(input,"^[A-Z][A-Za-z0-9]+$") ~= nil
end

function isKebabCase(input)
  return string.match(input,"^[a-z0-9-]+$") ~= nil
end

function isValidPort(port)
  return port >= 1 and port < 65536
end

function isValidTld(domain)
  return ( string.match(domain,"^[a-z0-9-]+%.[a-z]+$") and true ) or false
end

function isValidNixVersion(input)
  return string.match(input,"^[1-9][1-9]%.[0-9][0-9]$") ~= nil
end

function stringContains(input,other)
  return string.find(input, other) ~= nil
end

nixChecksumRegex = "^" .. ("[0-9a-f]"):rep(40) .. "$"

function isValidNixChecksum(input)
  return string.match(input, nixChecksumRegex) ~= nil
end

function noEOF(input)
  return string.find(input, 'EOF') == nil
end

function areNumbersInSequence(...)
  local arg = {...}
  for idx, current in ipairs(arg) do
    if current == nil then
      return false
    end
    if idx > 1 then
      if arg[idx - 1] + 1 ~= current then
        return false
      end
    end
  end

  return true
end

-- call like setContains({1, 2, 3}, 3)
function setContains(set, theValue)
  for _, value in ipairs(set) do
    if value == theValue then
      return true
    end
  end

  return false
end

function arrIntoSet(...)
  local values = {...}
  local output = {}

  for _, value in ipairs(values) do
    output[value] = true
	end

  return output
end

function isInSet(set, value)
  return set[value] or false
end

function stringHasNoFunnyValues(input)
  return string.match(input,"^[._A-Za-z0-9-+ ]+$") ~= nil
end

function isValidSemver(value)
  if type(value) ~= "string" then
    return false
  end
  if #value == 0 then
    return false
  end
  local first = string.sub(value, 1, 1)
  local last = string.sub(value, #value, #value)
  if first ~= '=' and tonumber(first) == nil then
    return false
  end
  if tonumber(last) == nil then
    return false
  end
  local segmentCount = 0
  for part in string.gmatch(value, '([^.]+)') do
    -- check locked version
    if segmentCount == 0 and string.sub(part, 1, 1) == '=' then
      local tail = string.sub(part, 2, #part)
      return tonumber(tail) ~= nil
    end
    if not string.match(part, '^[0-9]+$') then
      return false
    end
    if tonumber(part) == nil then
      return false
    end
    segmentCount = segmentCount + 1
  end
  return segmentCount == 3
end

function isValidGitHash(value)
  return #value == 40 and not ( string.match(value, '^[0-9a-f]+$') == nil )
end

function isValidBase32Sha256Hash(value)
  -- yeah this regex includes forbidden letters e o t but I don't care
  return #value == 52 and not ( string.match(value, '^[0-9a-z]+$') == nil )
end

function isValidRustEdition(value)
  return VALID_RUST_EDITIONS[value] or false
end

function isValidRustCompilerEnvKind(value)
  return VALID_RUST_COMPILE_ENVIRONMENT_KINDS[value] or false
end

function pathFilename(path)
  local split = {}
  for substr in string.gmatch(path, "[^/]+") do
    table.insert(split, substr)
  end

  return split[#split]
end

function readFileToString(file)
    local f = assert(io.open(file, "rb"))
    local content = f:read("*all")
    f:close()
    return content
end

-- returns filename and contents
function globFilesInDirectory(directory, extension)
  local result = {}
  -- we assume user is not malicious in this context
  -- and will not abuse command interface
  local query = 'find ' .. directory .. ' -type f'
  for file in io.popen(query):lines() do
    if stringEnds(file, extension) then
      local fname = pathFilename(file)
      local contents = readFileToString(file)
      table.insert(result, {filename = fname, contents = contents})
    end
  end

  return result
end

function loadGrafanaDashboards(directory)
  assert(directory ~= nil)

  local files = globFilesInDirectory(directory, '.json')

  for _, v in ipairs(files) do
    -- print('Adding dashboard: ' .. v.filename .. ' len: ' .. string.len(v.contents))
    data('grafana_dashboard', v)
  end
end

function xor(a, b)
  assert(type(a) == 'boolean');
  assert(type(b) == 'boolean');
  return a ~= b
end

-- server can be dns master or slave
-- but never both at once
function server_dns_check(is_dns_master, is_dns_slave)
  if not is_dns_master and not is_dns_slave then
    return true
  end

  return xor(is_dns_master, is_dns_slave)
end

function isValidDiskId(input)
  -- vda, sda, hda
  if string.match(input, '^[a-zA-Z0-9_-]+$') ~= nil then
    return true
  end

  return false
end

RESERVED_METRICS_NAMES = arrIntoSet('vault_sealed_clusters', 'epl_l1_provisioning_last_hash', 'node_boot_time_seconds')

function checkValidScrapedMetricsNames(metric_name)
  if RESERVED_METRICS_NAMES[metric_name] then
    return false
  end

  return true
end

function isValidDiskSerial(input)
  if input == '' then
    return true
  end

  -- vda, sda, hda
  if string.match(input, '^[a-zA-Z0-9_-]+$') ~= nil then
    return true
  end

  return false
end

-- auto: for bm datacenters by default require serial numbers of disks
DISK_ID_POLICIES = arrIntoSet('auto', 'require_serial', 'require_devname')

function isValidDiskIdPolicy(value)
  return DISK_ID_POLICIES[value] or false
end

ALLOWED_SUBNET_NAMES = arrIntoSet('lan', 'vpn', 'internet', 'dcrouter')

assert(setContains({1, 2, 3}, 2))
assert(not setContains({1, 2, 3}, 7))
assert(areNumbersInSequence(1, 2, 3))
assert(not areNumbersInSequence(1, 3, 4))
assert(not isValidSemver(' 1.2.3'))
assert(not isValidSemver('1.2.3 '))
assert(not isValidSemver('1.2'))
assert(isValidSemver('1.2.3'))
assert(isValidSemver('=1.2.3'))
assert(not isValidSemver(nil))
assert(not isValidSemver(''))
assert(not isValidSemver(' =1.2.3'))
assert(not isValidSemver('=1.2.3 '))
assert(isValidTld("epl-infra.net"))
assert(isValidTld("example.com"))
assert(not isValidTld("subdomain.example.com"))
assert(not isValidTld("henlo"))
assert(pathFilename("/salookie/dookie/mookie.txt") == "mookie.txt")
assert(isPascalCase("PascalCase"))
assert(not isPascalCase("pascalCase"))
assert(isValidNixVersion("22.05"))
assert(not isValidNixVersion(" 22.05"))
assert(not isValidNixVersion("22.05 "))
assert(not isValidNixVersion("1.2"))
assert(stringHasNoFunnyValues("1.2"))
assert(stringHasNoFunnyValues("1 2"))
assert(stringHasNoFunnyValues("1 abc A. 2"))
assert(stringHasNoFunnyValues("1_abc A. 2"))
assert(stringHasNoFunnyValues("1_abc A. 2"))
assert(stringHasNoFunnyValues("1_abc+A. 2"))
assert(stringHasNoFunnyValues("1_abc+A.-2"))
assert(not stringHasNoFunnyValues("1.2\n"))
assert(not stringHasNoFunnyValues("1\t2"))
assert(not stringHasNoFunnyValues("1\r2"))
assert(not stringHasNoFunnyValues("1'2"))
assert(not stringHasNoFunnyValues('1"2'))
assert(isValidDiskId('mookie1'))
assert(isValidDiskId('vda'))
assert(isValidDiskId('vdb'))
assert(not isValidDiskId(' vda'))
assert(not isValidDiskId('vda '))
assert(isValidDiskId('sdb'))
assert(isValidDiskId('nvme0n1'))
assert(isValidDiskId('hdz'))

VALID_RUST_EDITIONS = arrIntoSet('2018', '2021')

VALID_RUST_COMPILE_ENVIRONMENT_KINDS = arrIntoSet('backend_app', 'frontend_app')

-- will need to be overriden for every infra
ROOT_INFRA_DOMAIN = "epl-infra.net"

-- SOURCE_DIR - variable defined by edendb compiler
-- of directory in which current source file resides
-- download new dashboard to target directory from grafana with
-- curl -k https://<grafana external server dns>/api/dashboards/uid/<dashboard uid> | python3 -m json.tool > <dashboard name>.json
-- example:
-- curl -k https://adm-grafana-main.epl-infra.net/api/dashboards/uid/MQHVDmtWk | python3 -m json.tool > loki.json
loadGrafanaDashboards(SOURCE_DIR .. '/grafana-dashboards')

VALID_REGION_AVAILABILITY_MODES = arrIntoSet('single_dc', 'multi_dc')

VALID_DC_IMPLEMENTATIONS = arrIntoSet('manual', 'aws', 'gcloud', 'testvms', 'coprocessor', 'bm_simple')

VALID_ARCHITECTURES = arrIntoSet('x86_64', 'arm64')

function isValidArchitecture(architecture)
  return isInSet(VALID_ARCHITECTURES, architecture)
end

RESERVED_ZFS_DATASETS = arrIntoSet('consul', 'nomad', 'vault', 'docker', 'acme')

function isValidZfsDatasetName(dataset)
  return isInSet(RESERVED_ZFS_DATASETS, dataset)
end

ZFS_VALID_RECORDSIZE = arrIntoSet('4k', '8k', '16k', '32k', '64k', '128k', '256k', '512k', '1M')

function isValidZfsRecordSize(input)
  return isInSet(ZFS_VALID_RECORDSIZE, input)
end

VALID_DISK_MEDIUMS = arrIntoSet('hdd', 'ssd', 'nvme')

function isValidDiskMedium(input)
  return isInSet(VALID_DISK_MEDIUMS, input)
end

VALID_ZFS_VDEV_TYPES = arrIntoSet('mirror', 'raidz1', 'raidz2', 'raidz3')

function isValidZfsVdevType(input)
  return isInSet(VALID_ZFS_VDEV_TYPES, input)
end

VALID_CONFIG_TYPES = arrIntoSet('int', 'bool', 'string', 'float')

function isValidConfigType(input)
  return isInSet(VALID_CONFIG_TYPES, input)
end

VALID_CH_MAX_MERGE_TREE_VALUES = arrIntoSet(1024, 2048, 4096, 8192)

function isValidMergeMaxBlockSize(input)
  return isInSet(VALID_CH_MAX_MERGE_TREE_VALUES, input)
end

CH_FORBIDDEN_DB_NAMES = arrIntoSet(
  'system',
  'default',
  'information_schema',
  'INFORMATION_SCHEMA'
)

VALID_SERVICE_PROTOCOLS = arrIntoSet(
  'http',
  'tcp'
)

function isValidBlackboxPortProtocol(input)
  return isInSet(VALID_SERVICE_PROTOCOLS, input)
end
