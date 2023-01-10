//require('dotenv').config();
import dotenv from 'dotenv';
dotenv.config();

const config = {
    common: {
        dbSchemaNames:{
            users:'truly_users',
            tokens:'truly_tokens',
            owners:'truly_owners',
            tagsByToken:'truly_tagsByToken',
        } 
    },
    development: {
        blockchainURL: 'ws://localhost:8545',
        //dbURL: 'http://localhost:8000',
        awsURL:'http://localhost:4566',
        //dbRegion: 'local',
        //secretsURL: 'http://localhost:4566', // check localstack is up and running
        networkID: 1669986775736, // got it from ganache-cli and migration command
    },
    production: {
        blockchainURL: '',
        //dynamodbURL: 'https://dynamodb.eu-central-1.amazonaws.com',
        awsRegion: 'eu-central-1',
        networkID: 1
    }
};


function Config() {
    if (process.env.NODE_ENV === "development")
        return Object.assign({}, config.common, config.development);
    else if (process.env.NODE_ENV === "production")
        return Object.assign({}, config.common, config.production);
    else
        throw "not config for this environment";

}

export { Config }

