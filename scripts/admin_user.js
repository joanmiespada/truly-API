import dotenv from 'dotenv';
dotenv.config();

import crypto from "crypto";
import { Command } from 'commander';
const program = new Command();
import { dynamodb } from './ddbClient.js';
import { Config } from './configuration.js';
import { PutItemCommand } from "@aws-sdk/client-dynamodb";
const config = Config();

import { schemas } from './tableDefinitions.js';

async function create_adminuser() {

    let secret = process.env.HMAC_SECRET;
    let user_admin_email = process.env.ADMIN_USER_EMAIL;
    //let user_admin_passw = process.env.ADMIN_USER_PASSWORD;
    let user_admin_device = process.env.ADMIN_USER_DEVICE;
    let user_admin_walletAddress = process.env.ADMIN_TRON_ADDRESS;

    //let cyphered = crypto.createHmac('sha256', secret)
    //var encrypt = cyphered.update(user_admin_passw); //, 'utf8', 'hex'); //.digest("hex");
    //let gen_hmac = '';
    //console.log("password admin: "+ user_admin_passw +" cyphered: " + gen_hmac );

    await Promise.all([new Promise(async (resolve, error) => {
        var params = {
            TableName: 'users',
            Item: {
                'userID': { S: 'e8c9284b-a8e9-466d-87de-d2d3d8e1ab8f' },
                'creationTime': { S: new Date().toISOString() },
                'walletAddress': { S: user_admin_walletAddress },
                'email': { S: user_admin_email },
                'password': {S: 'NULL' },
                'device': { S: user_admin_device  },
                'userRoles': { SS: ['Basic', 'Admin'] },
                'userStatus': { S: 'Enabled' },
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
