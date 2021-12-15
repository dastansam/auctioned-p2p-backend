// SPDX-License-Identifier: MIT

pragma solidity 0.8.9;

import "../interfaces/IAuction.sol";
import "../math/SafeMathU128.sol";

contract AuctionProtocol is IAuction {
    using SafeMath for uint128;

    // Node in the protocol
    struct Node {
        address validator;
        string nodeUrl;
    }

    // State of the slot
    struct SlotState {
        address bidder;
        string bidderUrl;
        uint128 bidAmount;
        uint128 minBid;
    }

    // Blocks per slot
    uint8 public constant BLOCKS_PER_SLOT = 20;
    // Closed slot
    uint8 public constant CLOSED_SLOTS = 2;
    // Number of slots
    // auction slots: 40000 * 10 = 400000 blocks (around 70 days)
    uint128 public constant OPEN_SLOTS = 40000;
    // Treasury address
    address private _treasury;
    // Initail full node (main processor)
    address private _defaultProcessor;
    // Initial node url
    string public defaultProcessorUrl;
    // initail block number
    uint128 public initialBlockNumber;
    // how many percent of the bid amount is the minimum outbid amount
    uint128 private _minOutbid;
    // slot deadline (when the bidding for next slot is closed)
    uint16 private _slotDeadline;

    // slot states
    mapping(uint128 => SlotState) public slots;

    // pending balances
    mapping(address => uint128) public pendingBalances;

    // address to node mapping
    mapping(address => Node) public nodes;

    // constructor event 
    event AuctionProtocolCreated(
        address treasury, 
        address defaultProcessor, 
        string defaultProcessorUrl, 
        uint128 initialBlockNumber, 
        uint128 minOutbid, 
        uint16 slotDeadline
    );

    // event for new bid
    event NewBid(uint128 indexed slot, uint128 bidAmount, address indexed bidder);

    // event for new refund 
    event NewRefund(uint128 indexed slotNumber, uint128 refundAmount, address indexed bidder);

    // event for new slot deadline
    event NewSlotDeadline(uint16 slotDeadline);

    // event for new registered node
    event NewNode(address indexed node, string nodeUrl);

    // event for new slot
    event NewValidator(uint128 slotNumber, address validator, string nodeUrl);

    // modifier that makes sure only treasury can call this function
    modifier onlyTreasury {
        require(msg.sender == _treasury);
        _;
    }

    // event on initialization    
    constructor (
        address treasury, 
        address initialNodeAddress, 
        string memory initNodeUrl, 
        uint128 genesis
    ){
        require(treasury != address(0), "Treasury address cannot be 0");

        _treasury = treasury;
        _defaultProcessor = initialNodeAddress;
        defaultProcessorUrl = initNodeUrl;

        require(genesis >= block.number, 
            "Genesis block must be greater than or equal to current block"
        );

        initialBlockNumber = genesis;
        // how many percent of the bid amount is the minimum outbid amount
        _minOutbid = 1000;
        // bidding to slot ends in 10 blocks before the slot start
        _slotDeadline = 10;
        
        // emit auctionprotocolcreated event
        emit AuctionProtocolCreated(
            _treasury, 
            _defaultProcessor, 
            defaultProcessorUrl, 
            initialBlockNumber, 
            _minOutbid, 
            _slotDeadline
        );
    }

    // get slot deadline
    function getSlotDeadline() external override view returns (uint16) {
        return _slotDeadline;
    }

    // set slot deadline
    function setSlotDeadline(uint16 newDeadline) 
        external 
        override
        onlyTreasury 
    {
        require(newDeadline <= BLOCKS_PER_SLOT, "Slot deadline must be less than blocks per slot");
        _slotDeadline = newDeadline;
        emit NewSlotDeadline(_slotDeadline);
    }

    // get closed slots
    function getClosedSlots() external override pure returns (uint128) {
        return CLOSED_SLOTS;
    }

    // get treasury address
    function getTreasury() external override view returns(address) {
        return _treasury;
    }
    
    // set new treasury address
    function setTreasury(address newTreasury) external override onlyTreasury {
        _treasury = newTreasury;
    }

    // get default processor address
    function getDefaultProcessor() 
        external 
        override 
        view 
        returns(address) 
    {
        return _defaultProcessor;
    }

    // set new default processor address
    function setDefaultProcessor(address newTreasury) 
        external 
        override 
        onlyTreasury 
    {
        _defaultProcessor = newTreasury;
    }

    // get default processor url
    function getDefaultProcessorUrl() public view returns (string memory) {
        return defaultProcessorUrl;
    }

    // set new default processor url
    function setDefaultProcessorUrl(string memory processorUrl) 
        public onlyTreasury {
        defaultProcessorUrl = processorUrl;
    }

    // check if the node is registered in the protocol
    function isNodeRegistered(address node) public view returns (bool) {
        return nodes[node].validator != address(0);
    }

    // register node
    function registerValidator(address validator, string memory nodeUrl)
        external payable override 
    {
        require(
            keccak256(abi.encodePacked(nodeUrl)) != keccak256(abi.encodePacked("")), 
            "Node url cannot be empty"
        );
        require(validator != address(0), "Validator address cannot be 0");

        // add value to sender's balance
        pendingBalances[msg.sender] = uint128(msg.value);

        nodes[msg.sender].validator = validator;
        nodes[msg.sender].nodeUrl = nodeUrl;
        
        emit NewNode(msg.sender, nodeUrl);
    }

    // get initial block number
    function getInitialBlockNumber() public view returns(uint128) {
        return initialBlockNumber;
    }

    // set new initial block number
    function setInitialBlockNumber(uint128 genesis) public onlyTreasury {
        require(genesis >= block.number, 
            "Genesis block must be greater than or equal to current block"
        );

        initialBlockNumber = genesis;
    }

    // get current slot number
    function getCurrentSlotNumber() public view returns(uint128) {
        return getSlotNumber(uint128(block.number));
    }

    // get slot number for a given block number
    function getSlotNumber(uint128 blockNumber) public view returns(uint128) {
        return (blockNumber >= initialBlockNumber) ?
            (blockNumber - initialBlockNumber) / BLOCKS_PER_SLOT
            : uint128(0);
    }

    // get minimum bid for a given slot
    function getMinBid(uint16 slotNumber) public view returns(uint128) {
        require(
            slotNumber > (getCurrentSlotNumber() + CLOSED_SLOTS), 
            "Slot number must be greater than current slot number"
        );
        return slots[slotNumber].minBid;
    }

    // get current slot validator
    // Returns the address
    // and the url of the node that is the validator
    // e.g /ip4/p2p/nft/<PeerID>
    function getCurrentValidator() 
        public 
        view 
        returns(string memory, address) 
    {
        string memory nodeUrl = slots[getCurrentSlotNumber()].bidder != address(0) ?
                slots[getCurrentSlotNumber()].bidderUrl
                : getDefaultProcessorUrl();
    
        address validator = slots[getCurrentSlotNumber()].bidder != address(0) ?
                slots[getCurrentSlotNumber()].bidder
                : _defaultProcessor;
    
        return (nodeUrl, validator);
    }

    /**
    * @notice Bid on a slot, msg.value is the bid amount
    * @param slotNumber slot number
    */
    function bid(
        uint16 slotNumber,
        uint128 bidAmount
    ) external payable override {
        require(
            nodes[msg.sender].validator != address(0),
            "Validator needs to be registered"
        );

        require(
            slotNumber > (getCurrentSlotNumber() + CLOSED_SLOTS ), 
            "Slot number must be greater than current slot number"
        );

        require(msg.value > getMinBid(slotNumber), "Bid amount must be greater than 0");
        
        require(
            slotNumber <= (
                getCurrentSlotNumber() + CLOSED_SLOTS + OPEN_SLOTS
            ),
            "Auction is not open"
        );

        pendingBalances[msg.sender] = pendingBalances[msg.sender].add(uint128(msg.value));

        require(
            pendingBalances[msg.sender] >= bidAmount, 
            "Balance must be greater than bid amount"
        );

        _doBid(slotNumber, bidAmount, msg.sender);       
    }

    /**
    * @notice Claim refund
    * @param slotNumber slot number
    * @param refundAmount refund amount
    */
    function refund(uint16 slotNumber, uint128 refundAmount)
        external 
        override
        payable
    {
        require(
            slots[slotNumber].bidder != msg.sender,
            "Bidder must not be the winner of the slot"
        );

        require(
            pendingBalances[msg.sender] >= refundAmount,
            "Refund amount must be less than or equal to pending balance"
        );

        _doRefund(slotNumber, refundAmount, msg.sender);
    }

    /**
    * @notice Internal function for bid
    * @param slotNumber slot number
    * @param bidAmount bid amount
    * @param bidder bidder address
    */   
    function _doBid(
        uint16 slotNumber,
        uint128 bidAmount,
        address bidder
    ) private {
        address prevBidder = slots[slotNumber].bidder;
        uint128 prevBidAmount = slots[slotNumber].bidAmount;

        require(bidAmount > prevBidAmount, "Bid amount must be greater than previous bid amount");

        pendingBalances[bidder] = pendingBalances[bidder].sub(bidAmount);

        slots[slotNumber].bidder = bidder;
        slots[slotNumber].bidAmount = bidAmount;
        slots[slotNumber].bidderUrl = nodes[bidder].nodeUrl;

        // if there is a prevous bid we must return their bid amount to the pending balance
        if (prevBidder != address(0) && prevBidAmount != uint128(0)) {
            pendingBalances[prevBidder] = pendingBalances[prevBidder].add(prevBidAmount);
        }

        emit NewBid(slotNumber, bidAmount, bidder);
        emit NewValidator(slotNumber, bidder, nodes[bidder].nodeUrl);
    }

    /**
    * @notice Internal function for refund
    * @param slotNumber slot number
    * @param refundAmount refund amount
    * @param bidder bidder address
    */
    function _doRefund(
        uint16 slotNumber,
        uint128 refundAmount,
        address bidder
    ) private {
        pendingBalances[bidder] = pendingBalances[bidder].sub(refundAmount);

        emit NewRefund(slotNumber, refundAmount, bidder);
    }

}