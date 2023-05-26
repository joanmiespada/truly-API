// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.8 <0.9.0;

//import "./IterableMapping.sol";

contract LightNFT {
    //using IterableMapping for itmap;

    bool public contractPaused;
    string private urlBase;
    string private contract_version = "0.0.1";
    address private constant ADDRESS_NULL = address(0x0);
    address private _contractOwner;

    enum nNftState {
        Active,
        Inactive
    }

    struct nNftOwner {
        //address owner;
        uint256 percentatge;
        uint256 amountToWithdraw;
    }

    struct nNft {
        //  itmap owners; //mapping between address and nfts
        mapping(address => nNftOwner) owners;
        address[] _owners;
        string hashFile;
        //  string uriFile;
        //  string hashMetaFile;
        //  string uriMetaFile;
        uint256 price;
        // string uriLicense;
        // string copyright;
        nNftState state;
        bool exist;
    }

    mapping(string => nNft) private _UsersWithNfts;

    uint256 private _counter;

    event ContractSetupCompleted(address owner);
    event Minted(address owner, string token);
    event Transfered(
        string token,
        uint256 percentatge,
        address from,
        uint256 newFromPercentatge,
        address to,
        uint256 newToPercentate
    );
    event Sold(address buyer, string token, uint256 amount);

    event SoldOne(address buyer, string token, uint256 amount);
    event DisabledToken(string token);
    event EnabledToken(string token);
    event Withdrawn(address seller, uint256 amount);
    event WithdrawnRemainFail(address, uint256);
    event FoundsReceived(address, uint256);
    event ContractPaused();
    event ContractEnabled();
    /// the funds send don't cover the price
    error NotEnoughMoney(string);
    // // The function cannot be called at the current state.
    //error InvalidState();
    /// Not an owner of a token
    error NoOwner();
    /// No funds to be withdrawn
    error NoMoneyToWithdraw();
    /// Error getting funds
    error WithdrawCancelled(address, uint256);

    /*
    mapping(address => uint256) private userLastAction;
    uint256 throttleTime = 1; // block - 30 seconds;


    modifier throttling() {
        if (userLastAction[msg.sender] == 0x0) {
            userLastAction[msg.sender] = 0;
        }
        require(block.number - throttleTime >= userLastAction[msg.sender], "you're calling super fast");
        userLastAction[msg.sender] = block.number;
        _;
    }
*/

    constructor() {
        //require(_contractOwner != address(0), "Master address cannot be a zero address");

        _contractOwner = msg.sender; // 'msg.sender' is sender of current call, contract deployer for a constructor
        _counter = 0;
        contractPaused = false;
        urlBase = "https://truly.camera";

        emit ContractSetupCompleted(_contractOwner);
    }

    modifier isOwner() {
        require(msg.sender == address(_contractOwner), "Caller is not owner");
        _;
    }

    function circuitBreaker() public isOwner {
        // onlyOwner can call
        if (contractPaused == false) {
            contractPaused = true;
            emit ContractPaused();
        } else {
            contractPaused = false;
            emit ContractEnabled();
        }
    }

    function contractIsPaused() public view returns(bool){
        return contractPaused;
    }

    modifier checkIfPaused() {
        require(contractPaused == false);
        _;
    }

    function mint(
        address to,
        string memory token,
        //string memory uriFile,
        string memory hashFile,
        //string memory uriMetaInfo,
        //string memory hashMetaInfo,
        uint256 price
    )
        public
        isOwner
    {
        require(to != ADDRESS_NULL, "No null address is allowed");
        require(bytes(token).length != 0, "token is mandatory");
        //require(bytes(uriFile).length != 0, "uriFile is mandatory");
        require(bytes(hashFile).length != 0, "hashFile is mandatory");
        //require(bytes(uriMetaInfo).length != 0, "uriMetaInfo is mandatory");
        //require(bytes(hashMetaInfo).length != 0, "hashMetaInfo is mandatory");
        //require(bytes(uriLicense).length != 0, "uriLicense is mandatory");
        //require(bytes(copyright).length != 0, "copyright is mandatory");
        require(price >= 100, "token price must be >=100 wei");
        require(price % 2 == 0, "token price must be even");
        require(_UsersWithNfts[token].exist == false, "token is already in use");

        _UsersWithNfts[token].owners[_contractOwner].percentatge = 30;
        _UsersWithNfts[token].owners[_contractOwner].amountToWithdraw = 0;
        _UsersWithNfts[token]._owners.push(_contractOwner);
        _UsersWithNfts[token].owners[to].percentatge = 70;
        _UsersWithNfts[token].owners[to].amountToWithdraw = 0;
        _UsersWithNfts[token]._owners.push(to);

        _UsersWithNfts[token].hashFile = hashFile;
        //_UsersWithNfts[token].uriFile = uriFile;
        //_UsersWithNfts[token].hashMetaFile = hashMetaInfo;
        //_UsersWithNfts[token].uriMetaFile = uriMetaInfo;
        //_UsersWithNfts[token].uriLicense = uriLicense;
        //_UsersWithNfts[token].copyright = copyright;
        _UsersWithNfts[token].price = price;
        _UsersWithNfts[token].state = nNftState.Active;
        _UsersWithNfts[token].exist = true;

        _counter++;

        emit Minted(to, token);

    }

    function getStateName(nNftState state)
        internal
        pure
        returns (string memory)
    {
        if (state == nNftState.Active) return "Active";
        if (state == nNftState.Inactive) return "Inactive";
        return "";
    }

    // returns (nNft memory)
    function getContentByToken(string memory token)
        public
        view
        checkIfPaused
        returns (
              //string memory uriFile,
            string memory hashFile,
              //string memory uriMetaFile,
              //string memory hashMetaFile,
              //string memory uriLicense,
              //string memory copyright,
            string memory uri,
            uint256 price,
            string memory state
        )
    {
        require(bytes(token).length != 0, "token is mandatory");
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        nNft storage nft = _UsersWithNfts[token];
        string memory sts = getStateName(_UsersWithNfts[token].state);
        return (
            //nft.uriFile,
            nft.hashFile,
            //nft.uriMetaFile,
            //nft.hashMetaFile,
            //nft.uriLicense,
            //nft.copyright,
            concatenate(urlBase, token),
            nft.price,
            sts
        );
    }

    struct Ownership {
        address owner;
        uint256 percentatge;
    }

    function getOnwersByToken(string memory token)
        public
        view
        checkIfPaused
        returns (Ownership[] memory)
    {
        require(bytes(token).length != 0, "token is mandatory");
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        uint256 total = _UsersWithNfts[token]._owners.length;
        uint256 subtotal = 0;
        for (uint256 i = 0; i < total; i++) {
            address aux = _UsersWithNfts[token]._owners[i];
            if (_UsersWithNfts[token].owners[aux].percentatge > 0) {
                subtotal++;
            }
        }
        Ownership[] memory ownees = new Ownership[](subtotal);
        uint256 j = 0;
        for (uint256 i = 0; i < total; i++) {
            address aux = _UsersWithNfts[token]._owners[i];
            nNftOwner storage ow = _UsersWithNfts[token].owners[aux];
            if (ow.percentatge > 0) {
                ownees[j].owner = aux;
                ownees[j].percentatge = ow.percentatge;
                j++;
            }
        }
        return (ownees);
    }

    struct Withdrawship {
        address owner;
        uint256 amount;
        uint256 percentatge;
    }

    function getPendingWithdrawsByToken(string memory token)
        public
        view
        isOwner
        returns (Withdrawship[] memory)
    {
        require(bytes(token).length != 0, "token is mandatory");
        require(_UsersWithNfts[token].exist, "token doesn't exist");

        uint256 total = _UsersWithNfts[token]._owners.length;
        Withdrawship[] memory ownees = new Withdrawship[](total);

        for (uint256 i = 0; i < total; i++) {
            address aux = _UsersWithNfts[token]._owners[i];
            nNftOwner storage ow = _UsersWithNfts[token].owners[aux];
            ownees[i].owner = aux;
            ownees[i].percentatge = ow.percentatge;
            ownees[i].amount = ow.amountToWithdraw;
        }
        return (ownees);
    }

    function getWithdrawsForMeByToken(string memory token)
        public
        view
        checkIfPaused
        returns (uint256)
    {
        require(bytes(token).length != 0, "token is mandatory");
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        //require(
        //    _UsersWithNfts[token].owners[msg.sender].percentatge != 0,
        //    "address doesn't own the token"
        //);
        return _UsersWithNfts[token].owners[msg.sender].amountToWithdraw;
    }

    function getTotalMinted() public view isOwner returns (uint256) {
        return _counter;
    }

    function transferOwnership(
        address from,
        string memory token,
        address to,
        uint256 percentatge
    ) public isOwner returns (bool) {
        require(to != address(0), "No null to address is allowed");
        require(from != address(0), "No null from address is allowed");
        require(
            from != to,
            "source address must be differnt to destination address"
        );
        require(bytes(token).length != 0, "token is mandatory");
        require(
            percentatge <= 100,
            "percentatge isn't correct, must be between 1 and 100"
        );
        require(
            percentatge > 0,
            "percentatge isn't correct, must be between 1 and 100"
        );
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        require(
            _UsersWithNfts[token].owners[from].percentatge != 0,
            "from address must be among the owners"
        );
        //require(
        //    _UsersWithNfts[token].owners[to].percentatge != 0,
        //    "to address must be among the owners"
        //);
        require(
            _UsersWithNfts[token].owners[from].percentatge >= percentatge,
            //_UsersWithNfts[token].owners[from].value >= percentatge,
            "from address has no enought ownership"
        );
        require(
            _UsersWithNfts[token].owners[to].percentatge + percentatge <= 100,
            //_UsersWithNfts[token].owners[to].value.percentatge + percentatge <= 100,
            "destination address would have more than 100% ownership, impossible!"
        );
        /*require(
            _UsersWithNfts[token].state  == nNftState.Inactive ,
            "token state doesn't allow it"
        );*/

        _UsersWithNfts[token].owners[from].percentatge -= percentatge;
        _UsersWithNfts[token].owners[to].percentatge += percentatge;

        uint256 total = _UsersWithNfts[token]._owners.length;
        bool found = false;
        for (uint256 i = 0; i < total && !found; i++) {
            if (_UsersWithNfts[token]._owners[i] == to) found = true;
        }
        if (!found) _UsersWithNfts[token]._owners.push(to);

        emit Transfered(
            token,
            percentatge,
            from,
            _UsersWithNfts[token].owners[from].percentatge,
            to,
            _UsersWithNfts[token].owners[to].percentatge
        );
        return true;
    }

    function receiveFunds() public payable {
        emit FoundsReceived(msg.sender, msg.value);
    }

    function getBalance() public view isOwner returns (uint256) {
        return address(this).balance;
    }
    function getVersion() public view returns (string memory) {
        return contract_version;
    }

    function buy(
        string memory token,
        //string memory newLicence,
        //string memory newCopyright,
        //address from,
        uint256 newPrice
    ) public payable checkIfPaused {
        require(bytes(token).length != 0, "token is mandatory");
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        //require(bytes(newCopyright).length != 0, "new copyright is mandatory");
        //require(bytes(newLicence).length != 0, "new license is mandatory");
        require(
            _UsersWithNfts[token].state == nNftState.Active,
            "token state doesn't allow it"
        );
        require(
            _UsersWithNfts[token].owners[msg.sender].percentatge == 0,
            "you're already an owner"
        );

        address buyer = msg.sender; // payable(msg.sender);

        //buyer transfer money to contract
        uint256 deposit = msg.value;

        if (deposit < _UsersWithNfts[token].price) {
            revert NotEnoughMoney("buyer has not enought money");
        }

        uint256 totalPayed = 0;

        uint256 total = _UsersWithNfts[token]._owners.length;

        for (uint256 i = 0; i < total; i++) {
            address aux = _UsersWithNfts[token]._owners[i];
            uint256 percent = _UsersWithNfts[token].owners[aux].percentatge;
            if (percent > 0) {
                uint256 amountToWithdraw = (deposit / 100) * percent;
                _UsersWithNfts[token]
                    .owners[aux]
                    .amountToWithdraw = amountToWithdraw;
                _UsersWithNfts[token].owners[aux].percentatge = 0;
                totalPayed += amountToWithdraw;
            }
        }

        // in case % has decimals and remains some budget, then send it to the contract
        uint256 remain = deposit - totalPayed;

        if (remain > 0) {
            (bool sent, ) = payable(_contractOwner).call{value: remain}("");
            if (!sent) {
                emit WithdrawnRemainFail(_contractOwner, remain);
            }
        }

        _UsersWithNfts[token].owners[buyer].percentatge = 100;
        _UsersWithNfts[token].owners[buyer].amountToWithdraw = 0;
        _UsersWithNfts[token]._owners.push(buyer);
        //_UsersWithNfts[token].uriLicense = newLicence;
        //_UsersWithNfts[token].copyright = newCopyright;
        _UsersWithNfts[token].price = newPrice;

        emit Sold(buyer, token, deposit);
    }

    function withdraw(string memory token) public checkIfPaused {
        require(bytes(token).length != 0, "token is mandatory");
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        require(
            _UsersWithNfts[token].state == nNftState.Active,
            "token state doesn't allow it"
        );

        address owner = msg.sender;

        if (_UsersWithNfts[token].owners[owner].amountToWithdraw == 0) {
            revert NoMoneyToWithdraw();
        }
        uint256 amount = _UsersWithNfts[token].owners[owner].amountToWithdraw;
        //to avoid re-entrancy calls, let's setup to zero current amount before trasnferring the ether
        _UsersWithNfts[token].owners[owner].amountToWithdraw = 0;

        //payable(owner).transfer(amount);
        (bool sent, ) = payable(owner).call{value: amount}("");
        if (sent) {
            emit Withdrawn(owner, amount);
        } else {
            //if transaction fails, then restore the original amount
            _UsersWithNfts[token].owners[owner].amountToWithdraw = amount; //maybe isn't need it because revert will cancel any state changes...
            revert WithdrawCancelled(owner, amount);
        }
    }

    function disableByToken(string memory token) public isOwner {
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        _UsersWithNfts[token].state = nNftState.Inactive;
        emit DisabledToken(token);
    }

    function enableByToken(string memory token) public isOwner {
        require(_UsersWithNfts[token].exist, "token doesn't exist");
        _UsersWithNfts[token].state = nNftState.Active;
        emit EnabledToken(token);
    }

    function withdrawOwner(uint256 amount) public isOwner {
        (bool sent, ) = payable(_contractOwner).call{value: amount}("");
        if (sent) {
            emit Withdrawn(_contractOwner, amount);
        } else {
            //if transaction fails, then restore the original amount
            revert WithdrawCancelled(_contractOwner, amount);
        }
    }

    function setUrlBase(string memory uri) public isOwner {
        urlBase = uri;
    }

    function concatenate(string memory a, string memory b)
        private
        pure
        returns (string memory)
    {
        return string(abi.encodePacked(a, "/", b));
    }
}
