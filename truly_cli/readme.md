
# Create dependencies not terraformed

At /res folder there is all conf files. Ending with _development is for local dev.
Create files ending with_prod_stage.json to adjust values for those envs.

## Compile

```bash
cargo build -p truly_cli
```

Environment flag can have three values: **development**, **stage** or **production**
Example:

```bash
export ENVIRONMENT=development
```

## Create tables

```bash
cargo run -p truly_cli -- --table all --create
```

## Create Blockchain and Contract datasets

In local dev environment (docker localstack)

Deloy contracts manually using blockchain tools and copy addresses in json file

```bash
cargo run -p truly_cli -- --blockchain <file_json> --create
cargo run -p truly_cli -- --contract  <file_json> --create
```

## Create Secrets

```bash
cargo run -p truly_cli -- --store_secret <file_path_json> --create
```

## Create Key

With this key we'll be able to cypher information

```bash
cargo run -p truly_cli -- --key true --create
```

Save the id for next step.

## Store the secret key

With the id creted with the previous step, use it here to upload owner's secret key, encrypt with the key id and stored the data on a new secret at secrets manager.

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --store_key <key_id> --create --path <to_file>
```

## Create admin user

To signup as admin user we need a user with admin privilegies.

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --adminuser <email> --password <pass> --create
ENVIRONMENT=stage cargo run -p truly_cli -- --adminuser <email> --password <pass> --create
ENVIRONMENT=production cargo run -p truly_cli -- --adminuser <email> --password <pass> --create
```

## Additional infrastructure

All other dependencies such as queues, topics, etc... have been terraformed. Use terraform commands to deploy it.
