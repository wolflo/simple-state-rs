// SPDX-License-Identifier: MIT
pragma solidity 0.8.10;

contract SimpleState {
    uint256 public state;

    function move(uint256 next) external returns (uint256) {
        require(next <= 5, "Illegal move");
        if (state == 0) {
            require(next == 1, "Illegal move from 0");
        }
        if (state == 1) {
            require(next == 2 || next == 3 || next == 4, "Illegal move from 2");
        }
        if(state == 2 || state == 3 || state == 4) {
            require(next == 5 || next == 1, "Illegal move from 2,3,4");
        }
        require(state != 5, "No moves from 5");
        state = next;
    }

}
