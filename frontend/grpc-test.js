const caller = require('grpc-caller');
const path = require("path");
const PROTO_PATH = path.resolve(__dirname, "./src/protos/node_rpc.proto");

const client = caller('127.0.0.1:50051', PROTO_PATH, 'NodeRpc');

async function main() {
    const response = await client.CreateOrderCommitment({
        signer: "da", 
        taker: "da",
        contractAddress: "0x00", 
        tokenAddress: "0x01", 
        nftId: "dasdasd", 
        gossiper: "dasdasddasd", 
        price: 100000,
        orderType: 1,
        orderId: "1231a"
    });

    // console.log(response);

    const orders = await client.GetOrderCommitments({});
    console.log(orders);
}

main().then(() => {
    console.log("Done");
    process.exit(0);
}).catch((err) => {
    console.log(err);
});
