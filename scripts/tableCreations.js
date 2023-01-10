
import { Command } from 'commander';
const program = new Command();
import { dynamodb } from './ddbClient.js';
import { Config } from './configuration.js';
import { CreateTableCommand, DeleteTableCommand, DescribeTableCommand } from "@aws-sdk/client-dynamodb";
const config = Config();

import { schemas } from './tableDefinitions.js';

async function createSchema(tablefilter) {

    await Promise.all(schemas
        .filter( (f) => { 
            if( tablefilter == undefined) 
                return true
            else if (f.TableName == tablefilter) 
                return true
            else
                return false   
        }) 
        .map (async (schm) => {

            try {
                const data = await dynamodb.send(new DescribeTableCommand({ TableName: schm.TableName }));
                console.log(`table ${schm.TableName} already exists`)
            } catch (ex) {

                console.log(`Creating table ${schm.TableName}...`);
                try {
                    const data = await dynamodb.send(new CreateTableCommand(schm));
                    console.log(`Table ${schm.TableName} created successfully`);
                } catch (ex) {
                    console.log(`table ${schm.TableName} creation failed`)
                    console.log(ex.message)
                }
            }

    }));

};

async function deleteSchema() {
    await Promise.all(schemas.map(async (schm) => {
        try {
            // Call DynamoDB to delete the specified table
            const res = await dynamodb.send(new DeleteTableCommand({ TableName: schm.TableName }));

            console.log(`Successfully deleted ${schm.TableName}.`);
        } catch (ex) {
            /*if (err && err.code === 'ResourceNotFoundException') {
                console.log("Error: Table not found");
            } else if (err && err.code === 'ResourceInUseException') {
                console.log("Error: Table in use");
            } else*/
            console.log(`table ${schm.TableName} deletion failed`)
            console.log(ex.message)

        }
    }));
}

//module.exports = {createSchema, deleteSchema};

//createSchema();
//deleteSchema();
program
    .name('create dynamodb tables for event listener')
    .description('create tables at Dynmodb for listening events')
    .version('0.0.1')
    .option('-d, --delete','delete tables/schemas from Dynomodb')
    .option('-c, --create','create tables/schemas to Dynomodb')
    .option('-t, --table <table name>', 'specific table name')


program.parse(process.argv);

const options= program.opts();
if(options.delete)
    deleteSchema()
if(options.create){
    createSchema(options.table)
}

