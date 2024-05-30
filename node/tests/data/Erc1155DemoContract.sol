// SPDX-License-Identifier: MIT
pragma solidity ^0.8.20;

import "@openzeppelin/contracts/token/ERC1155/ERC1155.sol";

contract Erc1155DemoContract is ERC1155 {
    uint256 public constant GOLD = 0;
    uint256 public constant SILVER = 1;
    uint256 public constant THORS_HAMMER = 2;
    uint256 public constant SWORD = 3;
    uint256 public constant SHIELD = 4;

    constructor() public ERC1155("https://game.example/api/item/{id}.json") {
        _mint(0x1000000000000000000000000000000000000001, GOLD, 10**18, "");
        _mint(0x1000000000000000000000000000000000000001, SILVER, 10**27, "");
        _mint(0x1000000000000000000000000000000000000001, THORS_HAMMER, 1, "");
        _mint(0x1000000000000000000000000000000000000001, SWORD, 10**9, "");
        _mint(0x1000000000000000000000000000000000000001, SHIELD, 10**9, "");
    }
}