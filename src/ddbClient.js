
// Create service client module using ES6 syntax.
import { DynamoDBClient } from "@aws-sdk/client-dynamodb";
//import pkg from '@aws-sdk/client-dynamodb';
//const { DynamoDBClient  } = pkg;
//import pkg from "@aws-sdk/types";
//const {EndpointV2}= pkg;
// Set the AWS Region.
//const REGION = "REGION"; //e.g. "us-east-1"
// Create an Amazon DynamoDB service client object.

import { Config } from './configuration.js';

const config = Config();

let dynamodb;
if (process.env.NODE_ENV === "development")
    dynamodb = new DynamoDBClient({ endpoint: config.dbURL, region: config.dbRegion });
else if (process.env.NODE_ENV === "production")
    dynamodb = new DynamoDBClient();
else
    throw 'No NODE_ENV flag enabled'

export { dynamodb }

