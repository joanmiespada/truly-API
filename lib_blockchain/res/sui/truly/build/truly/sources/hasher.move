module truly::hasher {

    use sui::object::{Self, UID};
    use sui::transfer;
    //use sui::tx_context::{Self};
    use std::string::{Self, String};
    use sui::event;

    struct OwnerCap has key { id: UID }

    struct HashKeeper has key, store {
        id: UID,
        hash: String,// vector<u8>,
        algorithm: String, // vector<u8>,
        truly_id: String, // vector<u8>,
        //creator: address,
    }

    // ====== Errors ======
    const EHashEmpty: u64 = 1;
    const EAlgorithmEmpty: u64 = 2;
    const ETrulyIdEmpty: u64 = 3;

    // ====== Events ======
    /// For when hash has been added.
    struct HashAdded has copy, drop {
        //id: ID,
        truly_id: String,
        //creator: address,
    }


    //fun init(ctx: &mut TxContext) {
        //let admin = HashKeeper {
        //    id: object::new(ctx),
        //    
        //};
        //transfer::transfer(admin, tx_context::sender(ctx));
      //  owner = tx_context::sender(ctx);
    //}

    public fun hash(self: &HashKeeper): &String{ //vector<u8> {
        &self.hash
    }
    public fun algorithm(self: &HashKeeper): &String{ //&vector<u8> {
        &self.algorithm
    }
    public fun truly_id(self: &HashKeeper): &String{ //&vector<u8> {
        &self.truly_id
    }
    // public fun creator(self: &HashKeeper): &address {
    //     &self.creator
    // }

    entry fun add_hash(hash_value: vector<u8>, algorithm: vector<u8>, truly_id: vector<u8>, ctx: &mut sui::tx_context::TxContext) {


        assert!( std::vector::length(&hash_value) != 0  , EHashEmpty );
        assert!( std::vector::length(&algorithm) != 0  , EHashEmpty );
        assert!( std::vector::length(&truly_id) != 0  , EHashEmpty );

        let id_new = object::new(ctx);
        //let sender = tx_context::sender(ctx);


        let new_hash = HashKeeper {
            id: id_new,
            hash: string::utf8(hash_value),
            algorithm: string::utf8(algorithm),
            truly_id: string::utf8(truly_id), 
        };


         event::emit( HashAdded { 
             truly_id: string::utf8(truly_id), 
         });
        transfer::freeze_object(new_hash);
        //transfer::transfer(new_hash,sender);

    }



    #[test]
    public fun test_keep_create() {

        use sui::test_scenario;

        let admin = @0xABC;
        //let other = @0xA00;
        let hash1: vector<u8> = b"HashHashHash";
        //let hash2: String = std::string::utf8(hash1);
        let algo1: vector<u8> = b"MD5";
        //let algo2: String = std::string::utf8(algo1);
        let truly1: vector<u8> = b"1234-1234-123-1";
        //let truly2: String = std::string::utf8(truly1);

        let scenario_val = test_scenario::begin(admin);
        let scenario = &mut scenario_val;
        {
            add_hash(hash1, algo1, truly1, test_scenario::ctx(scenario));
        };

        // test_scenario::next_tx(scenario,admin);
        // {
        //      assert!(test_scenario::has_most_recent_for_address<HashKeeper>(admin),1);
        // };

        // test_scenario::next_tx(scenario, admin);
        // {
        //     let keep:HashKeeper =  test_scenario::take_from_address<HashKeeper>(scenario, admin);
        //     std::debug::print(&keep);
        //     assert!(*hash(&keep) == hash2, 1 );
        //     assert!(*algorithm(&keep) == algo2, 1);
        //     assert!(*truly_id(&keep) == truly2, 1);
        //     transfer::transfer(keep, admin);
        // };

        // test_scenario::next_tx(scenario, other);
        // {
        //     assert!(!test_scenario::has_most_recent_for_address<HashKeeper>(other),1);
        //     assert!(test_scenario::has_most_recent_for_address<HashKeeper>(admin),1);
        // };

        test_scenario::end(scenario_val);

    }

    #[test]
    public fun test_multiple_creates() {

        //use sui::transfer;
        use sui::test_scenario;
        use sui::object::ID;
        
        let admin = @0xABC;
        let scenario_val = test_scenario::begin(admin);
        let scenario = &mut scenario_val;
        {
            let hash1: vector<u8> = b"HashHashHash111111";
            let algo1: vector<u8> = b"MD5";
            let truly1: vector<u8> = b"1234-1234-123-1";
            add_hash(hash1, algo1, truly1, test_scenario::ctx(scenario));
        };
        test_scenario::next_tx(scenario, admin);
        {
            let hash1: vector<u8> = b"HashHashHash222222";
            let algo1: vector<u8> = b"MD4";
            let truly1: vector<u8> = b"444554-1234-123-1";

            add_hash(hash1, algo1, truly1, test_scenario::ctx(scenario));
        };
        test_scenario::next_tx(scenario, admin);
        {
            let res: vector<ID> = test_scenario::ids_for_address<HashKeeper>(admin);
            assert!( std::vector::length(&res) == 0, 1);
        };

        test_scenario::end(scenario_val);


    }


}