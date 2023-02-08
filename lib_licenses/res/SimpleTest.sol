// SPDX-License-Identifier: GPL-3.0
pragma solidity >=0.8.8 <0.9.0;

contract SimpleTest {
    uint256 storedData;

    event WhoCalls(address sender);

    function set(uint256 x) public {
        storedData = x;

        emit WhoCalls(msg.sender);
    }

    function get() public view returns (uint256) {
        return storedData;
    }
}
