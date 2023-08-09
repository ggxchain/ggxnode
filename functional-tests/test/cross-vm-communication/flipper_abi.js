const flipper_abi = [
    {
        "inputs": [],
        "name": "flip",
        "outputs": [],
        "stateMutability": "nonpayable",
        "type": "function"
    },
    {
        "inputs": [
            {
                "internalType": "bool",
                "name": "initial_value",
                "type": "bool"
            }
        ],
        "stateMutability": "nonpayable",
        "type": "constructor"
    },
    {
        "anonymous": false,
        "inputs": [
            {
                "indexed": false,
                "internalType": "bool",
                "name": "value",
                "type": "bool"
            }
        ],
        "name": "Flipped",
        "type": "event"
    },
    {
        "inputs": [],
        "name": "data",
        "outputs": [
            {
                "internalType": "bool",
                "name": "",
                "type": "bool"
            }
        ],
        "stateMutability": "view",
        "type": "function"
    }
]

export default flipper_abi;
