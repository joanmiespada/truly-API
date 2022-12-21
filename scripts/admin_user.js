import { Command } from 'commander';
const program = new Command();
import { dynamodb } from './ddbClient.js';
import { Config } from './configuration.js';
import { PutItemCommand } from "@aws-sdk/client-dynamodb";
const config = Config();

import { schemas } from './tableDefinitions.js';

async function create_adminuser() {

    await Promise.all([new Promise(async (resolve, error) => {
        var params = {
            TableName: 'users',
            Item: {
                'userID': { S: '1234-1234-1234-1234-1234-1234' },
                'creationTime': { S: new Date().toISOString() },
                'walletAddress': { S: '0x1234123412341234123412341234' },
                'email': { S: 'admin@admin.com' },
                'device': { S: 'device-12341234-12341234-12341234-1234' },
                'roles': { SS: ['Basic', 'Admin'] },
            }
        };
        try {
            const data = await dynamodb.send(new PutItemCommand(params));

            console.log("Success", data);
            resolve();
        } catch (err) {
            console.log("Error", err);
            error();
        }
    })]);
};




create_adminuser();
