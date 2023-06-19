
# Create dependencies not terraformed

At /res folder there is all conf files. Ending with _development is for local dev.
Create files ending with_prod_stage.json to adjust values for those envs.

## Compile

```bash
cargo build -p truly_cli
```

## Create tables

In local dev environment (docker localstack)

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --table all --create
```

In stage environment (eu-west-1)

```bash
ENVIRONMENT=stage cargo run -p truly_cli -- --table all --create
```

In stage *production* (eu-central-1)

```bash
ENVIRONMENT=production cargo run -p truly_cli -- --table all --create
```

## Create Blockchain and Contract datasets

In local dev environment (docker localstack)

Deloy contracts manually using blockchain tools and copy addresses in json file

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --blockchain <file_json> --create
ENVIRONMENT=development cargo run -p truly_cli -- --contract  <file_json> --create
```

## Create Secrets

In local dev environment (docker localstack)

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --store_secret true --create
```

In stage environment (eu-west-1)

```bash
ENVIRONMENT=stage cargo run -p truly_cli -- --store_secret true --create
```

In stage *production* (eu-central-1)

```bash
ENVIRONMENT=production cargo run -p truly_cli -- --store_secret true --create
```

## Create Key

In local dev environment (docker localstack)

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --key true --create
```

In stage environment (eu-west-1)

```bash
ENVIRONMENT=stage cargo run -p truly_cli -- --key true --create
```

In stage *production* (eu-central-1)

```bash
ENVIRONMENT=production cargo run -p truly_cli -- --key true --create
```

Save the id for next step.

## Store the secret key

With the id creted with the previous step, use it here to upload owner's secret key, encrypt with the key id and stored the data on a new secret at secrets manager.

In local dev environment (docker localstack)

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --store_key <key_id> --create
```

In stage environment (eu-west-1)

```bash
ENVIRONMENT=stage cargo run -p truly_cli -- --store_key <key_id> --create
```

In stage *production* (eu-central-1)

```bash
ENVIRONMENT=production cargo run -p truly_cli -- --store_key <key_id> --create
```

## Create async infra

Only in development env, at stage and prod envs it is terraformed. It creates minting queue, deadletter queue and minting topic.

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --async true --create
```

## Create admin user

To signup as admin user we need a user with admin privilegies.

```bash
ENVIRONMENT=development cargo run -p truly_cli -- --adminuser <email> --password <pass> --create
ENVIRONMENT=stage cargo run -p truly_cli -- --adminuser <email> --password <pass> --create
ENVIRONMENT=production cargo run -p truly_cli -- --adminuser <email> --password <pass> --create
```
