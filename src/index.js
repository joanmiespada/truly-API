
//let Web3 = require('web3')
//import { Web3 } from 'web3';
import Web3 from 'web3';
import { dynamodb } from '../ddbClient.js';
//const config = require('./configuration')();
import { Config } from '../configuration.js';
import fs from 'fs';
import { PutItemCommand } from "@aws-sdk/client-dynamodb";

import { Command } from 'commander';
const program = new Command();

const config = Config();

const BACK_FILL_EVENTS = 0;
const LATEST_EVENTS = 'latest';

let web3;

async function start(networkID, block, contractJsonPath) {

    web3 = new Web3(new Web3.providers.WebsocketProvider(config.blockchainURL));

    const file = fs.readFileSync(contractJsonPath).toString();
    const MyPureNFTContract = JSON.parse(file);
    const myContract = new web3.eth.Contract(
        MyPureNFTContract.abi,
        MyPureNFTContract.networks[networkID].address,
        //{from: MY_COINBASE, gas: MY_DEFAULT_GAS, gasPrice: MY_DEFAULT_GAS_PRICE}
    );

    console.log('reading from: ',MyPureNFTContract.networks[networkID].address  )

    myContract.events.allEvents({
        fromBlock: block // or BACK_FILL_EVENTS or LATEST_EVENTS
    }, async function (error, event) {
        if (error) {
            console.log(error); return;
        }
        console.log(event)
        let params;
        if (event.token)
            params = {
                TableName: config.dbSchemaNames.eventsByToken,// "eventsByToken",
                Item: {
                    "token": { S: event.token },
                    "eventID": { N: generateRowId(0) },
                    "eventName": { S: event.event },
                    "transaction": { S: event.transactionHash },
                    "creationTime": { S: new Date().toISOString() },
                    "eventInfo": {S: event.evenInfo }
                }
            };
        else
            params = {
                TableName: config.dbSchemaNames.eventsSystem,
                Item: {
                    "eventID": { N: generateRowId(0) },
                    "eventName": { S: event.event },
                    "transaction": { S: event.transactionHash },
                    "creationTime": { S: new Date().toISOString() }
                }
            };
        try {
            const res = await dynamodb.send(new PutItemCommand(params));
            console.log("PutItem succeeded.: " + event.event);
        } catch (ex) {
            console.error(ex);
        };
    });

}

const CUSTOMEPOCH = 1300000000000; // artificial epoch
function generateRowId(shardId /* range 0-64 for shard/slot */) {
    var ts = new Date().getTime() - CUSTOMEPOCH; // limit to recent
    var randid = Math.floor(Math.random() * 512);
    ts = (ts * 64);   // bit-shift << 6
    ts = ts + shardId;
    return (ts * 512) + randid;
}


async function stop() {
    console.log('Shutting down...')

    if (process.env.DEBUG) console.log(process._getActiveHandles())

    web3.currentProvider.disconnect(); // web3.currentProvider.connection.close()

    process.exit(0)
}

process.on('SIGTERM', async () => {
    console.log('Received SIGTERM')
    await stop()
})

process.on('SIGINT', async () => {
    console.log('Received SIGINT')
    await stop()
})
/*
program
    .name('blockchain event listener')
    .description('listening events from blockchain and store them at DynamoDB')
    .version('0.0.1')
    //.argument('<string>', 'string with ')
//program.command('starting')
    .option('-f,--first', 'since the begining, ideal for backfilling')
    .option('-l,--latest', 'default value. Read starting since the last event. Only news.')

//program.command('contract')
//    .description('Relative path to contract definition. Json file.')
//    .argument('<string>', 'path eg ../contracts/build/contract1.json')

program.parse(process.argv);

const options = program.opts();

*/
//let fromBlock = LATEST_EVENTS; // default value

//if (options.first)
//    fromBlock = BACK_FILL_EVENTS;
const networkId = process.env.NETWORK_ID || config.networkID
start(networkId, LATEST_EVENTS, '../PureNFT/build/contracts/LightNFT.json');
//start(BACK_FILL_EVENTS, '../PureNFT/build/contracts/LightNFT.json');


//if (process.env.NODE_ENV === "development")
console.log('listening events...')
