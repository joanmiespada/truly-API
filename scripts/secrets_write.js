
//import { SecretsManagerClient, CancelRotateSecretCommand } from "@aws-sdk/client-secrets-manager";
var AWS = require("aws-sdk");
import { Config } from './configuration.js';

const config = Config();

let client;
if (process.env.NODE_ENV === "development"){
    //client = new SecretsManagerClient({ endpoint: config.awsURL}); //, region: config.dbRegion }); //{ region: "REGION" });
    client = new AWS.SecretsManager({endpoint: config.awsURL});
}else if (process.env.NODE_ENV === "production")
    throw 'forbidden!! go to AWS console'
    //client = new SecretsManagerClient({ region: config.awsRegion }); //, region: config.dbRegion }); //{ region: "REGION" });
else
    throw 'No NODE_ENV flag enabled'

const params = require('fake_secret.json');


/*const command = new CancelRotateSecretCommand(params);

try {
    const data = await client.send(command);
    console.log('succsessfully stored');
    console.log(data);
  } catch (error) {
    console.log('error');
    console.log(error)
  }
*/

/* 
aws --endpoint-url=http://localhost:4566 --region=eu-central-1 secretsmanager create-secret --name "truly/api/secrets" --description "My test database secret created with the CLI" --secret-string file://fake_secret.json
aws --endpoint-url=http://localhost:4566 --region=eu-central-1 secretsmanager get-secret-value  --secret-id "truly/api/secrets"
*/
