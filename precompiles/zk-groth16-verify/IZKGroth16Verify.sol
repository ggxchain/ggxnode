// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title ZKGroth16Verify Interface
 *
 * The interface through which solidity contracts will interact with ZKGroth16Verify
 * Address :    0x0000000000000000000000000000000000008888
 */
interface IZKGroth16Verify {
    /**
     * @notice Verifies a Groth16 zkSNARK proof.
     *
     * @param proof_a The first element of the zkSNARK proof.
     * @param proof_b The second element of the zkSNARK proof.
     * @param proof_c The third element of the zkSNARK proof.
     * @param vk_alpha The first element of the verification key.
     * @param vk_beta The second element of the verification key.
     * @param vk_gamma The third element of the verification key.
     * @param vk_delta The fourth element of the verification key.
     * @param vk_ic The array of the rest of the elements of the verification key.
     * @param input The array of public inputs to the zkSNARK.
     *
     * @return valid A boolean value representing whether the proof is valid or not.
     */
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
    ) external returns (bool valid);
}
