// SPDX-License-Identifier: GPL-3.0
/*
    Copyright 2021 0KIMS association.

    This file is generated with [snarkJS](https://github.com/iden3/snarkjs).

    snarkJS is a free software: you can redistribute it and/or modify it
    under the terms of the GNU General Public License as published by
    the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    snarkJS is distributed in the hope that it will be useful, but WITHOUT
    ANY WARRANTY; without even the implied warranty of MERCHANTABILITY
    or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public
    License for more details.

    You should have received a copy of the GNU General Public License
    along with snarkJS. If not, see <https://www.gnu.org/licenses/>.
*/

pragma solidity >=0.7.0 <0.9.0;

/**
* @title ZKGroth16Verify Interface
*
* The interface of snarkjs groth16 solidity verifier.
*/
interface IGroth16Verifier {
    /**
    * @notice Verifies a Groth16 zkSNARK proof.
    *
    * @param _pA The first element of the zkSNARK proof.
    * @param _pB The second element of the zkSNARK proof.
    * @param _pC The third element of the zkSNARK proof.
    * @param _pubSignals The array of public inputs to the zkSNARK.
    *
    * @return valid A boolean value representing whether the proof is valid or not.
    */
    function verifyProof(
        uint[2] calldata _pA,
        uint[2][2] calldata _pB,
        uint[2] calldata _pC,
        uint[1] calldata _pubSignals
    ) external view returns (bool valid);
}
