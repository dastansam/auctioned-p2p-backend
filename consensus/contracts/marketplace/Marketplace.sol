// SPDX-License-Identifier: MIT

pragma solidity 0.8.9;

import "../math/SafeMathU128.sol";
import "@openzeppelin/contracts/token/ERC721/ERC721.sol";
import "@openzeppelin/contracts/token/ERC20/ERC20.sol";

/**
Writes a simple match function
This function needs to accept as params:
All the raw params of the order commitment
(what NFT is sold - identified by contract and id, how much it is sold for - identified 
by ERC20 contract and amount, gossiper and anything else you've decided to include)
seller signature of keccak256 hash of all the params (when hashing use encode not encodePacked)
matcher signature of keccak256 hash of all the params
The flow of operations is going to be:

- The seller approves the marketplace contract to move their NFT (ERC721 approve)
- The seller signs the order commitment and sends it to the gossiping node
- The buyer requests the information from the matching node 
 (getting the matcher signature, allongside all the other info)
- The buyer approves the sell amount (ERC20 approve)
- The buyer sends a buy tx where
- The contract checks all the params and requirements
- The contract checks if the order commitment is signed by the seller
- The contract tries to transferFrom the NFT from the seller to the buyer 
- (might fail if the signer has sold it already or trasnferred it away)
- The contract tries to charge the buyer for the amount via transferFrom
- The contract deducts the commission fee that is distributed between the gossiper, 
  matcher and main processor (some/all of these might be the same addresses)
 */

contract Marketplace {
    using SafeMath for uint128;

    // Commission allocation ratio in percentages
    // 1 - to the gossiping node
    // 2 - to the treasury
    // 3 - to the main processor
    uint8[3] public allocationRatio;

    // Treasury address
    address public treasury;

    // Stores canceled or matched orders
    mapping (bytes32 => bool) public cancelledOrMatched;

    constructor(uint8[3] memory _allocationRatio, address _treasury) public {
        require(
            _allocationRatio[0] + _allocationRatio[1] + _allocationRatio[2] <= 100,
            "Allocation ratio must sum up to 100"
        );
        allocationRatio = _allocationRatio;
        treasury = _treasury;
    }

    enum OrderType {
        BUY,
        SELL
    }

    /**
    /* @notice An ECDSA signature
    */
    struct MarketplaceSignature {
        uint8 v;
        bytes32 r;
        bytes32 s;
    }

    /**
    * @notice An order in the exchange
    */
    struct Order {
        address signer; // Someone who signed this order commitment, both buyer and seller can sign
        address taker; // Someone who is taking this order commitment, i.e buyer
        address contractAddress; // The contract of NFT
        address tokenAddress; // Payment token's address
        uint128 nftId; 
        address gossiper; // the node that first gossiped the order
        uint128 price; // the price of the NFT in payment tokens
        OrderType order_type; // BUY or SELL
    }

    event Match (
        address indexed signer,
        address indexed taker,
        address indexed contractAddress,
        uint128 nftId,
        address gossiper,
        uint128 price
    );

    function getAllocationRatio() public view returns (uint8[3] memory) {
        return allocationRatio;
    }

    function setAllocationRatio(uint8[3] memory _allocationRatio) public {
        // make sure the sum of the ratios is 100
        require(
            _allocationRatio[0] + _allocationRatio[1] + _allocationRatio[2] <= 100,
            "The sum of the allocation ratios must be 100"
        );
        allocationRatio = _allocationRatio;
    }

    /**
    * @notice Prepare Order for signature
    */
    function hashForSignature(Order memory order)
        internal
        pure
        returns (bytes32) 
    {
        return keccak256(
            abi.encode(
                order.signer,
                order.taker,
                order.contractAddress,
                order.tokenAddress,
                order.nftId, 
                order.gossiper, 
                order.price,
                order.order_type
            )
        );
    }

    /**
    * @notice Validate order, given signature and order
    */
    function validateOrder(bytes32 hash, Order memory order, MarketplaceSignature memory signature)
        internal
        view
        returns (bool) 
    {
        // make sure order is not already matched
        if (cancelledOrMatched[hash]) {
            return false;
        }

        if (ecrecover(hash, signature.v, signature.r, signature.s) == order.signer) {
            return true;
        }

        return false;
    }

    /**
    * @notice Validate order and create hash
    */
    function validateOrderAndHash(Order memory order, MarketplaceSignature memory signature)
        internal
        view
        returns (bytes32)
    {
        bytes32 hash = hashForSignature(order);

        require(validateOrder(hash, order, signature), "Invalid order");

        return hash;
    }

    /**
    * @notice Do orders match? Do basic checks
    */
    function ordersMatch(Order memory buy, Order memory sell)
        internal
        pure
        returns (bool)
    {
        // taker address can be 0x00 or the address of the taker
        return (
            buy.contractAddress == sell.contractAddress &&
            buy.nftId == sell.nftId &&
            buy.tokenAddress == sell.tokenAddress &&
            buy.gossiper == sell.gossiper &&
            buy.price == sell.price &&
            (buy.order_type == OrderType.BUY || sell.order_type == OrderType.SELL) &&
            (sell.taker == address(0) || sell.taker == buy.signer) &&
            (buy.taker == address(0) || buy.taker == sell.signer)  
        );
    }

    /**
    * @notice Process commisions for matched order
    * @dev This function is called when the order is matched
    * @dev The commission is calculated as a percentage of the fee
    */
    function processCommissions(
        uint128 amount,
        address payable gossiper,
        address payable matcher
    )
        public
        payable
        returns (bool)
    {
        uint128 gossiperCommission = amount * allocationRatio[0] / 100;
        uint128 treasuryCommission = amount * allocationRatio[1] / 100;
        uint128 matcherCommission = amount * allocationRatio[2] / 100;
        
        // Gossiper commission
        (bool gossiperSent, bytes memory gossiperData) = gossiper.call{value: gossiperCommission}("");
        require(gossiperSent, "Could not send gossiper commission");

        // Main processor commission
        (bool matcherSent, bytes memory matcherData) = matcher.call{value: matcherCommission}("");
        require(matcherSent, "Could not send matcher commission");

        // Treasury commission
        (bool treasurySent, bytes memory treasuryData) = treasury.call{value: treasuryCommission}("");
        require(treasurySent, "Could not send treasury commission");

        return gossiperSent && matcherSent && treasurySent;
    }

    /**
    * @notice A bid in the exchange
    * @dev Buyer should give approval to this contract for the ERC20 tokens
    * @dev Seller should give approval to this contract to move the NFT
    */
    function matchOrder(
        Order memory buy,
        MarketplaceSignature memory buySignature,
        Order memory sell,
        MarketplaceSignature memory sellSignature,
        address payable matcher
    ) public payable returns (bool) {
        // make sure the order is valid
        bytes32 buyHash;
        if (buy.signer != msg.sender) {
            buyHash = validateOrderAndHash(buy, buySignature);
        }

        // make sure the sell order is valid
        bytes32 sellHash;
        if (sell.signer != msg.sender) {
            sellHash = validateOrderAndHash(sell, sellSignature);
        }

        require(ordersMatch(buy, sell), "Orders do not match");

        // mark the orders as matched
        if (msg.sender != buy.signer) {
            cancelledOrMatched[buyHash] = true;
        }

        if (msg.sender != sell.signer) {
            cancelledOrMatched[sellHash] = true;
        }
        
        /** Process transfer */
        
        // transfer the NFT from the signer to the taker
        ERC721 nftContract = ERC721(buy.contractAddress);

        nftContract.safeTransferFrom(sell.signer, buy.taker, buy.nftId);

        // process payment
        ERC20 tokenContract = ERC20(buy.tokenAddress);

        // transfer erc20 token from taker to signer
        require(
            tokenContract.transferFrom(buy.taker, sell.signer, buy.price),
            "Could not transfer payment token"
        );

        // process commissions
        require(
            processCommissions(uint128(msg.value), payable(buy.gossiper), matcher), 
            "Could not process commissions"
        );
        
        // emit match event
        emit Match(
            sell.signer,
            buy.taker,
            buy.contractAddress,
            buy.nftId,
            buy.gossiper,
            buy.price
        );
    }
}