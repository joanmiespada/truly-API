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

    let user_admin_id = 'e8c9284b-a8e9-466d-87de-d2d3d8e1ab8f'
    let user_admin_email = "dfcgw345f03ger@truly.camera"
    let user_admin_device = "device-12DFFG34-12YUIO34-12QWE34-8790"
    let user_admin_walletAddress = "00x00";

    //let cyphered = crypto.createHmac('sha256', secret)
    //var encrypt = cyphered.update(user_admin_passw); //, 'utf8', 'hex'); //.digest("hex");
    //let gen_hmac = '';
    //console.log("password admin: "+ user_admin_passw +" cyphered: " + gen_hmac );

    await Promise.all([new Promise(async (resolve, error) => {
        var params = {
            TableName: config.dbSchemaNames.users ,
            Item: {
                'userID': { S: user_admin_id },
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
