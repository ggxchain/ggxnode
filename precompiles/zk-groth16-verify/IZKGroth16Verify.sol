// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title ZKGroth16Verify Interface
 *
 * The interface through which solidity contracts will interact with ZKGroth16Verify
 * Address :    0x0000000000000000000000000000000000008888
 */
interface IZKGroth16Verify {
    function verify(
        uint[2] memory proof_a,
        uint[2][2] memory proof_b,
        uint[2] memory proof_c,
        uint[2] memory vk_alpha,
        uint[2][2] memory vk_beta,
        uint[2][2] memory vk_gamma,
        uint[2][2] memory vk_delta,
        uint[2][] memory vk_ic,
        uint[] memory input
    ) external returns (bool);
}
