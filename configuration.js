//require('dotenv').config();
import dotenv from 'dotenv';
dotenv.config();

const config = {
    common: {
        dbSchemaNames:{
            users:'users',
            tokens:'tokens',
            owners:'owners',
            tagsByToken:'tagsByToken',
        } 
    },
    development: {
        blockchainURL: 'ws://localhost:8545',
        dbURL: 'http://localhost:8000',
        dbRegion: 'local',
        networkID: 1669986775736, // got it from ganache-cli and migration command
    },
    production: {
        blockchainURL: '',
        dbURL: '', //not needed in prod
        dbRegion: 'eu-west-1',
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

