const xvm_abi = [
    {
        "inputs": [
            {
                "internalType": "uint8",
                "name": "vm_id",
                "type": "uint8"
            },
            {
                "internalType": "bytes",
                "name": "to",
                "type": "bytes"
            },
            {
                "internalType": "bytes",
                "name": "input",
                "type": "bytes"
            },
            {
                "internalType": "uint256",
                "name": "value",
                "type": "uint256"
            },
            {
                "internalType": "uint256",
                "name": "storage_deposit_limit",
                "type": "uint256"
            }
        ],
        "name": "xvm_call",
        "outputs": [
            {
                "internalType": "bool",
                "name": "success",
                "type": "bool"
            },
            {
                "internalType": "bytes",
                "name": "data",
                "type": "bytes"
            }
        ],
        "stateMutability": "payable",
        "type": "function"
    }
]

export default xvm_abi;
