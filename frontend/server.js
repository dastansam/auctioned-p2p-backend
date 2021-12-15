// import client from './grpc/client';
const express = require('express');
const app = express();
const path = require('path');
const appPort = process.env.PORT || 3001;
const cors = require('cors');

const PROTO_PATH = path.resolve(__dirname, "./src/protos/node_rpc.proto");
const caller = require('grpc-caller');

app.options('*', cors())

// add json parser
app.use(express.json());

async function getOrders(port) {
    console.log("getOrders");
    const client = caller('127.0.0.1:' + port, PROTO_PATH, 'NodeRpc');
    const getOrders = await client.GetOrderCommitments({});
    console.log(getOrders);
    return getOrders;
}

async function createOrder(order, port) {
    const client = caller('127.0.0.1:' + port, PROTO_PATH, 'NodeRpc');
    const createOrder = await client.CreateOrderCommitment(order);
    return createOrder;
}

async function cancelOrder(order, port) {
    const client = caller('127.0.0.1:' + port, PROTO_PATH, 'NodeRpc');
    const cancelOrder = await client.CancelOrderCommitment(order);
    return cancelOrder;
}

app.get('/:port/orders', (req, res) => {
    getOrders(req.params.port).then(result => {
        res.json(result);
    })
    .catch(err => {
        console.log(err);
        res.json([]);
    });
})

app.post('/:port/create', (req, res) => {
    createOrder(req.body, req.params.port).then(result => {
        res.json(result);
    })
    .catch(err => {
        console.log(err);
        res.json({});
    });
})

app.post('/:port/cancel', (req, res) => {
    cancelOrder(req.body, req.params.port).then(result => {
        res.json(result);
    })
    .catch(err => {
        console.log(err);
        res.json({});
    });
})

app.listen(appPort, () => {
    console.log(`Server listening on port ${appPort}`);
})