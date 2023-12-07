pragma solidity ^0.8.0;

/**
 * @title XVM interface.
 */
interface XVM {
    /**
     * @dev Execute external VM call
     * @param vm_id - vm id, select the Evm or Wasm
     * @param to - call recepient
     * @param input - SCALE-encoded call arguments
     * @param value - the amount of native token to transfer, used for payable calls
     * @param storage_deposit_limit - The maximum amount of storage space that can be used
     */
    function xvm_call(
        uint8 vm_id,
        bytes calldata to,
        bytes calldata input,
        uint256 value,
        uint256 storage_deposit_limit
    ) external payable returns (bool success, bytes memory data);
}
