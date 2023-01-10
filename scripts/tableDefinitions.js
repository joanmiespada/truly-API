
import { Config } from './configuration.js';
const config = Config();

var schemas = [
    {

        "TableName": config.dbSchemaNames.users,
        "BillingMode": "PAY_PER_REQUEST",
        "AttributeDefinitions": [
            {
                "AttributeName": "userID",
                "AttributeType": "S"
            },
            {
                "AttributeName": "walletAddress",
                "AttributeType": "S"
            },
            {
                "AttributeName": "email",
                "AttributeType": "S"
            },
            {
                "AttributeName": "device",
                "AttributeType": "S"
            },
        ],
        "KeySchema": [
            {
                "AttributeName": "userID",
                "KeyType": "HASH"
            },
        ],
        "GlobalSecondaryIndexes": [
            {
                "Projection": {
                    "ProjectionType": "KEYS_ONLY"
                },
                "IndexName": "walletAddress",
                "KeySchema": [
                    {
                        "AttributeName": "walletAddress",
                        "KeyType": "HASH"
                    }
                ]
            },
            {
                "Projection": {
                    "ProjectionType": "KEYS_ONLY"
                },
                "IndexName": "email",
                "KeySchema": [
                    {
                        "AttributeName": "email",
                        "KeyType": "HASH"
                    },
                ]
            },
            {
                "Projection": {
                    "ProjectionType": "KEYS_ONLY"
                },
                "IndexName": "device",
                "KeySchema": [
                    {
                        "AttributeName": "device",
                        "KeyType": "HASH"
                    },
                ]
            },

        ],
        "Tags": [
            {
                "Key": "project",
                "Value": "truly"
            }
        ]

    },
    {

        "TableName": config.dbSchemaNames.tokens,
        "BillingMode": "PAY_PER_REQUEST",
        "AttributeDefinitions": [
            {
                "AttributeName": "tokenID",
                "AttributeType": "S"
            },
            {
                "AttributeName": "creationTime",
                "AttributeType": "S"
            },
        ],
        "KeySchema": [
            {
                "AttributeName": "tokenID",
                "KeyType": "HASH"
            },
            {
                "AttributeName": "creationTime",
                "KeyType": "RANGE"
            },
        ],
        
        "Tags": [
            {
                "Key": "project",
                "Value": "truly"
            }
        ]

    },{

        "TableName": config.dbSchemaNames.tagsByToken,
        "BillingMode": "PAY_PER_REQUEST",
        "AttributeDefinitions": [
            {
                "AttributeName": "tag",
                "AttributeType": "S"
            },
            {
                "AttributeName": "tokenID",
                "AttributeType": "S"
            },
        ],
        "KeySchema": [
            {
                "AttributeName": "tag",
                "KeyType": "HASH"
            },
            {
                "AttributeName": "tokenID",
                "KeyType": "RANGE"
            },
        ],
        
        "Tags": [
            {
                "Key": "project",
                "Value": "truly"
            }
        ]

    },{

        "TableName": config.dbSchemaNames.owners,
        "BillingMode": "PAY_PER_REQUEST",
        "AttributeDefinitions": [
            {
                "AttributeName": "userID",
                "AttributeType": "S"
            },
            {
                "AttributeName": "tokenID",
                "AttributeType": "S"
            },
        ],
        "KeySchema": [
            {
                "AttributeName": "userID",
                "KeyType": "HASH"
            },
            {
                "AttributeName": "tokenID",
                "KeyType": "RANGE"
            },
        ],
        
        "Tags": [
            {
                "Key": "project",
                "Value": "truly"
            }
        ]

    }
]

export { schemas }