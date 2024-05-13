//SPDX-License-Identifier:MIT
pragma solidity 0.8.25;

contract Flipper {
    bool public data;

    event Flipped(bool value);

    constructor(bool initial_value) {
        data = initial_value;
    }

    function flip() public {
        data = !data;
        emit Flipped(data);
    }
}

contract Storage {
    uint256 number;
    
    function store(uint256 num) public {
        number = num;
    }
    function retrieve() public view returns (uint256) {
        return number;
    }
}
