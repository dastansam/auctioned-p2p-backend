const caller = require('grpc-caller');
const path = require("path");
const PROTO_PATH = path.resolve(__dirname, "./node_rpc.proto");

const client = (port) => caller('127.0.0.1:' + port, PROTO_PATH, 'NodeRpc');

module.exports = client;
