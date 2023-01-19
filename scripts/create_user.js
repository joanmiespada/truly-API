import dotenv from 'dotenv';
dotenv.config();

import { Command } from 'commander';
const program = new Command();
import { dynamodb } from './ddbClient.js';
import { Config } from './configuration.js';
import { PutItemCommand } from "@aws-sdk/client-dynamodb";
const config = Config();

async function create_user(options) {

    let user_id = options.id; 
    let user_email = options.email; 
    let user_device = options.device; 
    let user_walletAddress = options.wallet; 


    await Promise.all([new Promise(async (resolve, error) => {
        var params = {
            TableName: config.dbSchemaNames.users ,
            Item: {
                'userID': { S: user_id },
                'creationTime': { S: new Date().toISOString() },
                'walletAddress': { S: user_walletAddress },
                'email': { S: user_email },
                'password': {S: 'NULL' },
                'device': { S: user_device  },
                'userRoles': { SS: ['Basic', 'Admin'] },
                'userStatus': { S: 'Disabled' },
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

program
    .name('create admin user')
    .description('create admin user')
    .version('0.0.1')
    .option('-e, --email <email>','user email')
    //.option('-p, --password <password>','user pass')
    .option('-d, --device <device>', 'user device')
    .option('-w, --wallet <wallet address>', 'user wallet address')
    .option('-i, --id <user identifier>', 'user identifier')


program.parse(process.argv);

const options= program.opts();

create_user(options);
