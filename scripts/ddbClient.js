
// Create service client module using ES6 syntax.
import { DynamoDBClient } from "@aws-sdk/client-dynamodb";
import { Config } from './configuration.js';

const config = Config();

let dynamodb;
if (process.env.NODE_ENV === "development")
    dynamodb = new DynamoDBClient({ endpoint: config.awsURL });
else if (process.env.NODE_ENV === "production")
    dynamodb = new DynamoDBClient( { region: config.awsRegion });
else
    throw 'No NODE_ENV flag enabled'

export { dynamodb }

