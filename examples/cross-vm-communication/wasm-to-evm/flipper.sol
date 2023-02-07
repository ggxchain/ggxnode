pragma solidity ^0.8.0;

contract Flipper {
    bool public data;

    constructor(bool initial_value) {
        data = initial_value;
    }

    function flip() public {
        data = !data;
    }
}
