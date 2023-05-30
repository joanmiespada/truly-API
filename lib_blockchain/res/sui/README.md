# Contrats on SUI blockchain

## Deploy contracts

Run sui-test-validator

```bash
RUST_LOG=error cargo run -p sui-test-validator
```

Check my addresses:

```bash
sui client envs
sui client new-env --alias local2 --rpc http://127.0.0.1:9000
sui client switch --env local2
sui client active-env
sui client addresses
```

Adding sui coins to one of your addresses:

```bash
curl --location --request POST 'http://127.0.0.1:9123/gas' \
--header 'Content-Type: application/json' \
--data-raw '{
    "FixedAmountRequest": {
        "recipient": "0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"
    }
}'
```

Check your gas coin addresses:

```bash
sui client gas
```

Deploy the contract

```bash
sui client publish ./ --gas 0x3572473506bd5fa3983ab0845dccbe88ed80f14e97459e104cea9321a0e9c819  --gas-budget=12500000
```

result:

```bash
INCLUDING DEPENDENCY Sui
INCLUDING DEPENDENCY MoveStdlib
BUILDING truly
Successfully verified dependencies on-chain against source.
----- Transaction Digest ----
6fa88PPankQiKAEBgJMMRZDR3FMm8xGfK5YSq7RVvKge
----- Transaction Data ----
Transaction Signature: [Signature(Ed25519SuiSignature(Ed25519SuiSignature([0, 68, 216, 148, 59, 205, 143, 182, 97, 121, 2, 157, 220, 219, 63, 237, 125, 68, 43, 214, 208, 44, 152, 195, 129, 182, 158, 31, 119, 249, 115, 97, 150, 86, 194, 201, 218, 131, 215, 74, 103, 97, 44, 214, 150, 56, 192, 43, 64, 118, 92, 222, 205, 105, 112, 120, 61, 164, 128, 132, 70, 35, 178, 242, 15, 177, 41, 74, 15, 138, 110, 150, 17, 122, 185, 234, 19, 205, 147, 61, 204, 66, 56, 196, 243, 27, 0, 21, 182, 145, 246, 188, 242, 47, 180, 177, 35])))]
Transaction Kind : Programmable
Inputs: [Pure(SuiPureValue { value_type: Some(Address), value: "0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30" })]
Commands: [
  Publish(<modules>,0x0000000000000000000000000000000000000000000000000000000000000001,0x0000000000000000000000000000000000000000000000000000000000000002),
  TransferObjects([Result(0)],Input(0)),
]

Sender: 0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30
Gas Payment: Object ID: 0x3572473506bd5fa3983ab0845dccbe88ed80f14e97459e104cea9321a0e9c819, version: 0xb, digest: 3RJj2msU7wbrJgzm4wsQMBUw6hZ4qmTksrTKihb9MsjL 
Gas Owner: 0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30
Gas Price: 1000
Gas Budget: 12500000

----- Transaction Effects ----
Status : Success
Created Objects:
  - ID: 0x101919030142d959573ea2826efc4b7439bf32c30fb72f4b92f102c44d5d1775 , Owner: Account Address ( 0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30 )
  - ID: 0x788c167e4a430615954221b2469abae1f77ae25eaddcfea11007269db9b6b4b5 , Owner: Immutable
Mutated Objects:
  - ID: 0x3572473506bd5fa3983ab0845dccbe88ed80f14e97459e104cea9321a0e9c819 , Owner: Account Address ( 0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30 )

----- Events ----
Array []
----- Object changes ----
Array [
    Object {
        "type": String("mutated"),
        "sender": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        "owner": Object {
            "AddressOwner": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        },
        "objectType": String("0x2::coin::Coin<0x2::sui::SUI>"),
        "objectId": String("0x3572473506bd5fa3983ab0845dccbe88ed80f14e97459e104cea9321a0e9c819"),
        "version": String("12"),
        "previousVersion": String("11"),
        "digest": String("HQEd9ek2vJqgsWwSjAUCoVKC5EepCU6FeQd2g8xnXCYe"),
    },
    Object {
        "type": String("created"),
        "sender": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        "owner": Object {
            "AddressOwner": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        },
        "objectType": String("0x2::package::UpgradeCap"),
        "objectId": String("0x101919030142d959573ea2826efc4b7439bf32c30fb72f4b92f102c44d5d1775"),
        "version": String("12"),
        "digest": String("E457NWw3jtcrDdKApGvcPaaSyUR6os3N7hdL1T2M4uia"),
    },
    Object {
        "type": String("published"),
        "packageId": String("0x788c167e4a430615954221b2469abae1f77ae25eaddcfea11007269db9b6b4b5"),
        "version": String("1"),
        "digest": String("2zBAdzdsvsvd3dRZAFFm7EFiGa5QXtHS5Ur31tFaSBWq"),
        "modules": Array [
            String("hasher"),
        ],
    },
]
----- Balance changes ----
Array [
    Object {
        "owner": Object {
            "AddressOwner": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        },
        "coinType": String("0x2::sui::SUI"),
        "amount": String("-10388280"),
    },
]
```

Call our function:

```bash
sui client call --function add_hash --module hasher --package 0x788c167e4a430615954221b2469abae1f77ae25eaddcfea11007269db9b6b4b5 --args hashhash1 md5 12342134  --gas-budget 10000000
```

Result:

```bash
----- Transaction Digest ----
3BsQ1k1anHc1XyT1tFn3ugDu2qpwt1wwaMJcSKxkhVbC
----- Transaction Data ----
Transaction Signature: [Signature(Ed25519SuiSignature(Ed25519SuiSignature([0, 92, 93, 252, 6, 141, 88, 225, 88, 38, 42, 59, 243, 198, 125, 44, 63, 101, 210, 96, 213, 238, 98, 236, 84, 67, 4, 43, 196, 226, 97, 107, 48, 156, 37, 135, 69, 90, 35, 162, 54, 134, 161, 143, 178, 1, 246, 73, 40, 185, 175, 81, 208, 59, 143, 137, 183, 184, 4, 204, 60, 210, 127, 158, 5, 177, 41, 74, 15, 138, 110, 150, 17, 122, 185, 234, 19, 205, 147, 61, 204, 66, 56, 196, 243, 27, 0, 21, 182, 145, 246, 188, 242, 47, 180, 177, 35])))]
Transaction Kind : Programmable
Inputs: [Pure(SuiPureValue { value_type: Some(Vector(U8)), value: "hashhash1" }), Pure(SuiPureValue { value_type: Some(Vector(U8)), value: "md5" }), Pure(SuiPureValue { value_type: Some(Vector(U8)), value: "12342134" })]
Commands: [
  MoveCall(0x788c167e4a430615954221b2469abae1f77ae25eaddcfea11007269db9b6b4b5::hasher::add_hash(Input(0),Input(1),Input(2))),
]

Sender: 0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30
Gas Payment: Object ID: 0x3572473506bd5fa3983ab0845dccbe88ed80f14e97459e104cea9321a0e9c819, version: 0xf, digest: 948xETxn49ziDRFnsFrHNNULxDymgmDHbPLXAkxLrWv4 
Gas Owner: 0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30
Gas Price: 1000
Gas Budget: 10000000

----- Transaction Effects ----
Status : Success
Created Objects:
  - ID: 0x5a74c8db4151a21884c73e71d2d9c1b4fab4f21240b296d78692cb198666eec6 , Owner: Immutable
Mutated Objects:
  - ID: 0x3572473506bd5fa3983ab0845dccbe88ed80f14e97459e104cea9321a0e9c819 , Owner: Account Address ( 0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30 )

----- Events ----
Array [
    Object {
        "id": Object {
            "txDigest": String("3BsQ1k1anHc1XyT1tFn3ugDu2qpwt1wwaMJcSKxkhVbC"),
            "eventSeq": String("0"),
        },
        "packageId": String("0x788c167e4a430615954221b2469abae1f77ae25eaddcfea11007269db9b6b4b5"),
        "transactionModule": String("hasher"),
        "sender": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        "type": String("0x788c167e4a430615954221b2469abae1f77ae25eaddcfea11007269db9b6b4b5::hasher::HashAdded"),
        "parsedJson": Object {
            "truly_id": String("12342134"),
        },
        "bcs": String("73nRti9Xe5dM"),
    },
]
----- Object changes ----
Array [
    Object {
        "type": String("mutated"),
        "sender": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        "owner": Object {
            "AddressOwner": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        },
        "objectType": String("0x2::coin::Coin<0x2::sui::SUI>"),
        "objectId": String("0x3572473506bd5fa3983ab0845dccbe88ed80f14e97459e104cea9321a0e9c819"),
        "version": String("16"),
        "previousVersion": String("15"),
        "digest": String("Dn8ejmzDeihTiTqDz2LdfdAbM9mvrmSb8tL1qzmGwx2w"),
    },
    Object {
        "type": String("created"),
        "sender": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        "owner": String("Immutable"),
        "objectType": String("0x788c167e4a430615954221b2469abae1f77ae25eaddcfea11007269db9b6b4b5::hasher::HashKeeper"),
        "objectId": String("0x5a74c8db4151a21884c73e71d2d9c1b4fab4f21240b296d78692cb198666eec6"),
        "version": String("16"),
        "digest": String("E7YS9PgsfhP7685jCfHDHe9UePsKFUzzWZ5ZS6SJEs6a"),
    },
]
----- Balance changes ----
Array [
    Object {
        "owner": Object {
            "AddressOwner": String("0x042d9857b31cdec48b00332fec4a7adf8bf8e2a5a1561ef7778ce1abf7b91f30"),
        },
        "coinType": String("0x2::sui::SUI"),
        "amount": String("-2499480"),
    },
]
```
