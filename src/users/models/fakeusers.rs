use uuid::Uuid;

fn list_fake_users() -> HashMap<String, User> {
    let mut map = HashMap::new();

    let mut id = String::from(Uuid::new_v4());
    map.insert(
        id,
        User {
            user_id: id,
            creation_time: Utc::now(),
            wallet_address: "0xWalletFakeAddress4239AEF9",
            email: "foo@foo.com",
            device: "deviceid-hwdjh2374-dfkjkjh23e-dkb",
            roles: vec![UserRoles::Admin],
        },
    );
    
    id = String::from(Uuid::new_v4());
    map.insert(
        id,
        User {
            user_id: id,
            creation_time: Utc::now(),
            wallet_address: "0xWalletFakeAddressFFFFFFF",
            email: "foo1@foo1.com",
            device: "deviceid-hw123123123a-dsf-sdfjkjh23e-dkb",
            roles: vec![],
        },
    );
    map
}
