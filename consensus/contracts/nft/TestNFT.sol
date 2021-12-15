// SPDX-License-Identifier: MIT

pragma solidity 0.8.9;

import "@openzeppelin/contracts/token/ERC721/extensions/ERC721URIStorage.sol";
import "@openzeppelin/contracts/utils/Counters.sol";

/**
* @notice Simple NFT smart contract with URI storage
*/
contract TestNFT is ERC721URIStorage {
    using Counters for Counters.Counter;
    Counters.Counter private _tokenIDs;

    constructor() ERC721("TestNFT", "TN") {}

    /**
    * @notice Mints a new token
    */
    function mintNFT(
        address _to,
        string memory _tokenURI
    ) public returns (uint256) {
        _tokenIDs.increment();

        uint256 newTokenID = _tokenIDs.current();

        _mint(_to, newTokenID);
        _setTokenURI(newTokenID, _tokenURI);

        return newTokenID;
    }
}