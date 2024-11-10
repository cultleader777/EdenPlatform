
set -e
# pass root vault token in to access all secrets
[ -n "$VAULT_TOKEN" ] || { echo VAULT_TOKEN environment variable is required; exit 7; }
