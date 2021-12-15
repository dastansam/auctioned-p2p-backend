import React from "react";

// We'll use ethers to interact with the Ethereum network and our contract
import { ethers } from "ethers";

// We import the contract's artifacts and address here, as we are going to be
// using them with ethers
import AuctionArtifact from "../contracts/AuctionProtocol.json";
import MarketplaceArtifact from "../contracts/Marketplace.json";
import NFTArtifact from "../contracts/TestNFT.json";
import WUSDArtifact from "../contracts/WrappedUSD.json";

// All the logic of this dapp is contained in the Dapp component.
// These other components are just presentational ones: they don't have any
// logic. They just render HTML.
import { NoWalletDetected } from "./NoWalletDetected";
import { ConnectWallet } from "./ConnectWallet";
import { Loading } from "./Loading";
import { Bid } from "./Bid";
import { TransactionErrorMessage } from "./TransactionErrorMessage";
import { WaitingForTransactionMessage } from "./WaitingForTransactionMessage";
import { NoTokensMessage } from "./NoTokensMessage";
import { RegisterValidator } from "./RegisterValidator";

// This is the Hardhat Network id, you might change it in the hardhat.config.js
// Here's a list of network ids https://docs.metamask.io/guide/ethereum-provider.html#properties
// to use when deploying to other networks.
const HARDHAT_NETWORK_ID = '0x539';

// This is an error code that indicates that the user canceled a transaction
const ERROR_CODE_TX_REJECTED_BY_USER = 4001;

// Path to the protobuf file
const PROTO_PATH = "../protos/node_rpc.proto";

export class Marketplace extends React.Component {
  constructor(props) {
    super(props);
    this.initialState = {
      // sell orders in the marketplace
      orders: [],
      // The user's address and balance
      selectedAddress: undefined,
      balance: undefined,
      nodeUrl: undefined,
      // The ID about transactions being sent, and any possible error with them
      txBeingSent: undefined,
      transactionError: undefined,
      networkError: undefined,
      grpcPort: "50051",
    };

    this.state = this.initialState;
  }

  render() {
    // Ethereum wallets inject the window.ethereum object. If it hasn't been
    // injected, we instruct the user to install MetaMask.
    if (window.ethereum === undefined) {
      return <NoWalletDetected />;
    }

    // The next thing we need to do, is to ask the user to connect their wallet.
    // When the wallet gets connected, we are going to save the users's address
    // in the component's state. So, if it hasn't been saved yet, we have
    // to show the ConnectWallet component.
    //
    // Note that we pass it a callback that is going to be called when the user
    // clicks a button. This callback just calls the _connectWallet method.
    if (!this.state.selectedAddress) {
      return (
        <ConnectWallet 
          connectWallet={() => this._connectWallet()} 
          networkError={this.state.networkError}
          dismiss={() => this._dismissNetworkError()}
        />
      );
    }

    // If the token data or the user's balance hasn't loaded yet, we show
    // a loading component.
    if (!this.state.balance) {
    console.log(this.state.balance);
      return <Loading />;
    }

    // If everything is loaded, we render the application.
    return (
      <div className="container p-4">
        <div className="row">
          <div className="col-12">
            <h1>
              Marketplace
            </h1>
            <div className="container">
              Welcome <b>{this.state.selectedAddress}</b>, here's the list of orders in sale:
                <br/>
                <br/>
                {this.state.orders.map((order, index) => (
                    //for each order create new row with order details
                    <div className="row" key={index}>
                        <div className="col-12">
                            <h4><b>Order: </b> {index}</h4>
                        </div>
                        <div className="col-12">
                            <p><b>Signer</b>: {order.signer}</p>
                        </div>
                        <div className="col-12">
                            <p><b>Gossiper</b> {order.gossiper}</p>
                        </div>
                        <div className="col-12">
                            <p><b>Nft id: </b> {order.nftId}</p>
                        </div>
                        <div className="col-12">
                            <p><b>Contract address:</b> {order.contractAddress} </p>
                        </div>
                        <div className="col-12">
                            <p><b>Price: </b> {order.price}</p>
                        </div>
                        <div className="col-12">
                            <button 
                                className="btn btn-primary" 
                                onClick={() => this._purchaseOrder(order)}
                            >Purchase
                            </button>
                        </div>
                    </div>
                ))}
            </div>
          </div>
        </div>

        <hr />

        <div className="row">
            <h2> This part of the page shows nfts that you own: </h2>

          <div className="col-12">
          </div>
        </div>

        <div className="row">
          <div className="col-12">
            {/*
              If the user has no tokens, we don't show the Tranfer form
            */}
            {this.state.balance.eq(0) && (
              <NoTokensMessage selectedAddress={this.state.selectedAddress} />
            )}

            {/*
              This component displays a form that the user can use to send a 
              transaction and transfer some tokens.
              The component doesn't have logic, it just calls the transferTokens
              callback.
            */}
            {this.state.balance.gt(0) && (
              <Bid
                bidForSlot={(slotNumber, amount, bidAmount) =>
                  this._bidForSlot(slotNumber, amount, bidAmount)
                }
              />
            )}
          </div>
        </div>
      </div>
    );
  }

  componentWillUnmount() {
    // We poll the user's balance, so we have to stop doing that when Dapp
    // gets unmounted
    this._stopPollingData();
  }

  async _connectWallet() {
    const [selectedAddress] = await window.ethereum.enable();

    // Once we have the address, we can initialize the application.

    if (!this._checkNetwork()) {
      return;
    }

    this._initialize(selectedAddress);

    // We reinitialize it whenever the user changes their account.
    window.ethereum.on("accountsChanged", ([newAddress]) => {
      this._stopPollingData();
      if (newAddress === undefined) {
        return this._resetState();
      }
      
      this._initialize(newAddress);
    });
    
    // We reset the dapp state if the network is changed
    window.ethereum.on("networkChanged", ([networkId]) => {
      this._stopPollingData();
      this._resetState();
    });
  }

  _initialize(userAddress) {
    // This method initializes the dapp

    // We first store the user's address in the component's state
    this.setState({
      selectedAddress: userAddress,
    });

    this._intializeEthers();
    this._getTokenData();
    this._startPollingData();
  }

  async _intializeEthers() {
    // We first initialize ethers by creating a provider using window.ethereum
    this._provider = new ethers.providers.Web3Provider(window.ethereum);

    // When, we initialize the contract using that provider and the token's
    // artifact. You can do this same thing with your contracts.
    this._auction = new ethers.Contract(
      "0x1536383CE8E7c70fCb8A4AFF2275E0c7D71D7519",
      AuctionArtifact.abi,
      this._provider.getSigner(0)
    );

    this._marketplace = new ethers.Contract(
        "0xdC937D0CfdE76144bcfc1336630Ad8854566C2F4",
        MarketplaceArtifact.abi,
        this._provider.getSigner(0)
    );

    this._nftContract = new ethers.Contract(
        "0x5d3a536E4D6DbD6114cc1Ead35777bAB948E3643",
        NFTArtifact.abi,
        this._provider.getSigner(0)
    );

    this._wusdContract = new ethers.Contract(
        "0x6B175474E89094C44Da98b954EedeAC495271d0F",
        WUSDArtifact.abi,
        this._provider.getSigner(0)
    );
  }

  // The next two methods are needed to start and stop polling data
  _startPollingData() {
    this._pollDataInterval = setInterval(() => {
      this._updateMarketplace();
      this._updateSlotState();
      this._updateBalance();
    }, 60000);

    // We run it once immediately so we don't have to wait for it
    this._updateBalance();
    this._updateMarketplace();

  }

  _stopPollingData() {
    clearInterval(this._pollDataInterval);
    this._pollDataInterval = undefined;
  }

  // The next two methods just read from the contract and store the results
  // in the component state.
  _getTokenData() {
    const name = "Auction Protocol";
    const symbol = "PRO";

    this.setState({ tokenData: { name, symbol } });
  }

  async _updateMarketplace() {
    const orders = await fetch('/50051/orders');
    const ordersJson = await orders.json();
    console.log(ordersJson);
    this.setState({orders: ordersJson?.orderCommitments || []});
  }

  async _updateSlotState() {
    const validator = await this._auction.getCurrentValidator();
    console.log(validator[1]?.slice(5));

    this.setState({ validator: validator[0], grpcPort: validator[1]?.slice(5) });
  }

  async _updateBalance() {
    const balance = await this._provider.getBalance(this.state.selectedAddress);
    console.log(await this._provider.getBlockNumber());
    this.setState({ balance });
  }

  // register new node
  async _createOrderCommitment(orderCommitment) {
    try{
      const tx = await this._auction.registerValidator(
        this.state.selectedAddress
      );
      this.setState({ txBeingSent: tx.hash });
      
      this._dismissTransactionError();
      
      const receipt = await tx.wait();

      // The receipt, contains a status flag, which is 0 to indicate an error.
      if (receipt.status === 0) {
        // We can't know the exact error that made the transaction fail when it
        // was mined, so we throw this generic one.
        throw new Error("Transaction failed");
      }

      // If we got here, the transaction was successful, so you may want to
      // update your state. Here, we update the user's balance.
      await this._updateSlotState();

    } catch (error) {
      // We check the error code to see if this error was produced because the
      // user rejected a tx. If that's the case, we do nothing.
      if (error.code === ERROR_CODE_TX_REJECTED_BY_USER) {
        return;
      }

      // Other errors are logged and stored in the Dapp's state. This is used to
      // show them to the user, and for debugging.
      console.error(error);
      this.setState({ transactionError: error });
    } finally {
      // If we leave the try/catch, we aren't sending a tx anymore, so we clear
      // this part of the state.
      this.setState({ txBeingSent: undefined });
    }
  }
  // This method sends an ethereum transaction to bid for slot
  async _purchaseOrder(slotNumber, amount, bidAmount) {
    try {
      // If a transaction fails, we save that error in the component's state.
      // We only save one such error, so before sending a second transaction, we
      // clear it.
      this._dismissTransactionError();

      // We send the transaction, and save its hash in the Dapp's state. This
      // way we can indicate that we are waiting for it to be mined.
      const tx = await this._auction.bid(
        slotNumber, 
        bidAmount, 
        {
          value: amount,
          from: this.state.selectedAddress,
        }
      );
      this.setState({ txBeingSent: tx.hash });

      // We use .wait() to wait for the transaction to be mined. This method
      // returns the transaction's receipt.
      const receipt = await tx.wait();

      // The receipt, contains a status flag, which is 0 to indicate an error.
      if (receipt.status === 0) {
        // We can't know the exact error that made the transaction fail when it
        // was mined, so we throw this generic one.
        throw new Error("Transaction failed");
      }

      // If we got here, the transaction was successful, so you may want to
      // update your state. Here, we update the user's balance.
      await this._updateSlotState();
    } catch (error) {
      // We check the error code to see if this error was produced because the
      // user rejected a tx. If that's the case, we do nothing.
      if (error.code === ERROR_CODE_TX_REJECTED_BY_USER) {
        return;
      }

      // Other errors are logged and stored in the Dapp's state. This is used to
      // show them to the user, and for debugging.
      console.error(error);
      this.setState({ transactionError: error });
    } finally {
      // If we leave the try/catch, we aren't sending a tx anymore, so we clear
      // this part of the state.
      this.setState({ txBeingSent: undefined });
    }
  }

  // This method just clears part of the state.
  _dismissTransactionError() {
    this.setState({ transactionError: undefined });
  }

  // This method just clears part of the state.
  _dismissNetworkError() {
    this.setState({ networkError: undefined });
  }

  // This is an utility method that turns an RPC error into a human readable
  // message.
  _getRpcErrorMessage(error) {
    if (error.data) {
      return error.data.message;
    }

    return error.message;
  }

  // This method resets the state
  _resetState() {
    this.setState(this.initialState);
  }

  // This method checks if Metamask selected network is Localhost:8545 
  _checkNetwork() {
    console.log(window.ethereum.chainId);

    if (window.ethereum.chainId === HARDHAT_NETWORK_ID) {
      return true;
    }

    this.setState({ 
      networkError: 'Please connect Metamask to Localhost:8545'
    });

    return false;
  }
}
