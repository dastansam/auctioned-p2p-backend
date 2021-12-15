// SPDX-License-Identifier: MIT

pragma solidity 0.8.9;

import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/**
* @notice Token we will use to pay for NFTs
*/
contract WrappedUSD is ERC20 {
    constructor () ERC20("Wrapped USD", "WUSD") {
        _mint(msg.sender, 10000000000);
    }

    // override decimals function to 2
    function decimals() public view virtual override returns (uint8) {
        return 2;
    }
}
