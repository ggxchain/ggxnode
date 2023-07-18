// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title ZKGroth16Verify Interface
 *
 * The interface through which solidity contracts will interact with ZKGroth16Verify
 * Address :    0x0000000000000000000000000000000000008888
 */
// struct Proof {
//     uint[2] a;
//     uint[2][2] b;
//     uint[2] c;
// }

// struct VerificationKey {
//     uint[2] alpha;
//     uint[2][2] beta;
//     uint[2][2] gamma;
//     uint[2][2] delta;
//     uint[2][] ic;
// }

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
