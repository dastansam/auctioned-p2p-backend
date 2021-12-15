const {expect} = require('chai');
const client = require('./_grpc-client');
const testOrders = require('./test-orders.json');

// GRPC ports of the nodes
const NODE1_PORT = '50051';
const NODE2_PORT = '50052';
const NODE3_PORT = '50053';

describe('Network integration tests', function() {
    this.timeout(100000);
    before(async function() {
        // wait 15 seconds for the network to start
        console.log('Waiting for network to start...');
        await new Promise(resolve => setTimeout(resolve, 15000));
    })

    it('Initially the network does not have any orders', async function() {
        const client1 = await client(NODE1_PORT);
        const client2 = await client(NODE2_PORT);
        const client3 = await client(NODE3_PORT);

        const orders1 = await client1.GetOrderCommitments({});
        const orders2 = await client2.GetOrderCommitments({});
        const orders3 = await client3.GetOrderCommitments({});
        console.log('Initially the network does not have any orders');
        expect(orders1).to.deep.equal({});
        expect(orders2).to.deep.equal({});
        expect(orders3).to.deep.equal({});
    })

    it('Create order commitment works', async function(){
        const client1 = await client(NODE1_PORT);

        const orderCommitment = await client1.CreateOrderCommitment(testOrders[0]);

        const { gossiper, ...orderCommitmentWithoutGossiper } = orderCommitment;
        const { gossiper: nullGossiper, ...expectedOrderCommitmentWithoutGossiper } = testOrders[0];
        
        console.log("Node should populate gossiper field with its address");

        expect(gossiper !== nullGossiper).to.equal(true);
        expect(expectedOrderCommitmentWithoutGossiper).to.deep.equal(orderCommitmentWithoutGossiper);
    })

    it('Create order commitment and gossip', async function(){
        const client2 = await client(NODE2_PORT);
        const client3 = await client(NODE3_PORT);

        await client2.CreateOrderCommitment(testOrders[1]);

        const orders1 = await client2.GetOrderCommitments({});
        const orders2 = await client2.GetOrderCommitments({});
        const orders3 = await client3.GetOrderCommitments({});

        expect(orders2).to.deep.equal(orders3);
        expect(orders2).to.deep.equal(orders1);
    })

    it('Should create multiple orders and gossip all', async function() {
        const client1 = await client(NODE1_PORT);
        const client2 = await client(NODE2_PORT);
        const client3 = await client(NODE3_PORT);

        // add orders, just randomly choosing clients
        for (i = 0; i < testOrders.length; i++) {
            if (i % 3 === 0) {
                await client1.CreateOrderCommitment(testOrders[i]);
            }
            else if (i % 3 === 1) {
                await client2.CreateOrderCommitment(testOrders[i]);
            }
            else {
                await client3.CreateOrderCommitment(testOrders[i]);
            }
        }

        const orders1 = await client1.GetOrderCommitments({});
        const orders2 = await client2.GetOrderCommitments({});
        const orders3 = await client3.GetOrderCommitments({});

        expect(orders1).to.deep.equal(orders2);
        expect(orders1).to.deep.equal(orders3);
        expect(orders2).to.deep.equal(orders3);
        expect(orders1.orderCommitments.length).to.deep.equal(testOrders.length);
    })

    it("should cancel an order commitment", async function() {
        const client1 = await client(NODE1_PORT);

        const orderCommitment = await client1.CreateOrderCommitment(testOrders[0]);
        
        const orderCommitmentsBefore = await client1.GetOrderCommitments({});

        await client1.CancelOrderCommitment(orderCommitment);

        const orderCommitmentsAfter = await client1.GetOrderCommitments({});
        
        expect(orderCommitmentsBefore.orderCommitments.length).to.deep.equal(orderCommitmentsAfter.orderCommitments.length + 1);
    })
})