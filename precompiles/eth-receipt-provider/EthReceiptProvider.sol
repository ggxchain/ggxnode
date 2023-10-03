// SPDX-License-Identifier: GPL-3.0-only
pragma solidity >=0.8.0;

/**
 * @title EthReceiptProvider Interface
 *
 * The interface through which solidity contracts will interact with EthReceiptProvider
 * Address :    0x0000000000000000000000000000000000009999
 */

interface EthReceiptProvider {
    /**
	 * set keys 
     * Selector: 0xd8be245a
	 *
	 * @param receipt_hash receipt hash
     * @param contract_addr contract address
	 *
     */
    function logs_for_receipt(bytes calldata receipt_hash, bytes calldata contract_addr) external returns (bytes calldata);
}