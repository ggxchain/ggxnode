pragma solidity ^0.8.0;

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
