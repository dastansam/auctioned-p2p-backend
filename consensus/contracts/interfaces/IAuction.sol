// SPDX-License-Identifier: MIT

pragma solidity 0.8.9;

/**
* @title Auction Protocol Interface
* @dev A protocol for auctioning time slots for backends
* This is a proof of concept for a protocol for auctioning time slots for backends.
* Each node bids an amount of bonds they are ready to pay 
* for the time slot they are ready to run. Protocal simply selects 
* the highest bid and assigns the time slot to that node.
* Loser nodes will be refunded their bond.
* Winner node's bond will be burned and stored in the treasury.
 */

interface IAuction {
    /**
    * @dev Get current slot's deadline
    * @return uint16 deadline 
    */
    function getSlotDeadline() external view returns (uint16);

    /**
    * @notice Set slot's deadline
     */
    function setSlotDeadline(uint16 newDeadline) external;

    /**
    @notice Get closed auction slots
    @return uint16 closedSlots
    */
    function getClosedSlots() external view returns (uint128);

    /**
    * @notice Get treasury address
    * @return address treasury
    */
    function getTreasury() external view returns (address);

    /**
    * @notice Set treasury address
    * @param newTreasury address
    */
    function setTreasury(address newTreasury) external;

    /**
    * @notice Get get initial full node
    * @return address initialFullNode
    */
    function getDefaultProcessor() external view returns (address);

    /**
    * @notice Set initial full node
    * @param newMainProcessor new default processor
    */
    function setDefaultProcessor(address newMainProcessor) external;

    /**
    * @notice register node
    * @param nodeUrl url of node
    * @param validator address of validator
    */
    function registerValidator(address validator, string memory nodeUrl) external payable;

    /**
    * @notice process bid, msg.value is the bid amount
    * @param slotNumber slotNumber
    * @param bidAmount bid amount
    */
    function bid(uint16 slotNumber, uint128 bidAmount) external payable;

    /**
    * @notice process refund
    * @param slotNumber slot
    * @param refundAmount amount of bonds
    */
    function refund(uint16 slotNumber, uint128 refundAmount) external payable;
}