#!/bin/bash -e

# ==============================================================================
# This script fetches blockchain metadata from specified network endpoints using
# JSON-RPC. The script can be run with predefined network names ('sydney' or
# 'brooklyn') or with a 'custom' RPC endpoint specified by the user.
#
# Usage:
#   ./fetch_metadata.sh <network> [<custom RPC>]
#
# Parameters:
#   <network>     The network name from which to fetch metadata. Supported values:
#                 - sydney   : Use the Sydney network RPC endpoint.
#                 - brooklyn : Use the Brooklyn network RPC endpoint.
#                 - custom   : Use a custom RPC endpoint. Must provide the RPC URL
#                              as the second argument.
#   [<custom RPC>]: (Optional) The RPC endpoint URL when using the 'custom' network.
#                   This argument is required when <network> is set to 'custom'.
#
# Example:
#   ./fetch_metadata.sh sydney
#   ./fetch_metadata.sh brooklyn
#   ./fetch_metadata.sh custom "http://127.0.0.1:5100"
#
# Output:
#   The script will generate a file named "eth_light_client_<network>.scale"
#   containing the metadata retrieved from the specified RPC endpoint.
#
# Prerequisites:
#   - jq : Command-line JSON processor
#   - xxd: Hex dump tool
#   - curl: Command-line tool for transferring data with URLs
#
# Notes:
#   - Ensure the required tools (jq, xxd, curl) are installed and available in
#     the system's PATH.
#   - For the 'custom' network option, a valid RPC URL must be provided as the
#     second argument.
# ==============================================================================

# Define default RPC endpoints
SYDNEY_RPC="https://gate.ggxchain.net/sydney-archive"
BROOKLYN_RPC="https://brooklyn-archive.ggxchain.net/dev-brooklyn"

# Get network and custom RPC (if provided)
NETWORK="$1"
CUSTOM_RPC="$2"

function get_metadata() {

    local rpc_url="$1"
    local output_file="$2"
    local json_rpc_request='{"jsonrpc":"2.0","method":"state_getMetadata", "id": 1}'

    curl -sX POST -H "Content-Type: application/json" \
        --data "${json_rpc_request}" "${rpc_url}" | \
        jq -r .result | xxd -r -p > "${output_file}"
}

# Display usage if no arguments are provided
if [[ -z "${NETWORK}" ]]; then
    echo "Usage: fetch_metadata.sh <network> [<custom RPC>]"
    echo "Networks:"
    echo "  sydney     - Use Sydney network RPC"
    echo "  brooklyn   - Use Brooklyn network RPC"
    echo "  custom     - Use a custom RPC endpoint (provide RPC URL as second argument)"
    echo "Example:"
    echo "  fetch_metadata.sh sydney"
    echo "  fetch_metadata.sh custom \"http://127.0.0.1:5100\""
    exit 1
fi

# Determine the appropriate RPC URL based on the network name
if [[ "${NETWORK}" == "sydney" ]]; then
    RPC="${SYDNEY_RPC}"
elif [[ "${NETWORK}" == "brooklyn" ]]; then
    RPC="${BROOKLYN_RPC}"
elif [[ "${NETWORK}" == "custom" ]]; then
    if [[ -n "${CUSTOM_RPC}" ]]; then
        RPC="${CUSTOM_RPC}"
    else
        echo "Error: For custom network, please provide the RPC endpoint."
        exit 1
    fi
else
    echo "Error: Unknown network '${NETWORK}'. Supported networks: sydney, brooklyn, custom."
    exit 1
fi

# Set output filename based on network
OUTPUT_FILE="eth_light_client_${NETWORK}.scale"

# Fetch metadata
get_metadata "${RPC}" "${OUTPUT_FILE}"