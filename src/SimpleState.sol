// SPDX-License-Identifier: MIT
pragma solidity 0.8.10;

contract SimpleState {
    uint256 public state;
    address public trigger;

    function step(uint256 next) external returns (uint256) {
        require(next <= 6, "Illegal move");
        if (state == 0) {
            require(next == 1, "Illegal move from 0");
        }
        if (state == 1) {
            require(next == 2 || next == 3 || next == 4, "Illegal move from 1");
        }
        if(state == 2 || state == 3 || state == 4) {
            require(next == 5 || next == 1, "Illegal move from 2,3,4");
        }
        if(state == 5) {
            uint256 csize;
            address _trigger = trigger;
            assembly {
                csize := extcodesize(_trigger)
            }
            require(csize > 0, "Trigger wannacry to move");
            require(next == 6, "Illegal move from 5");
        }
        state = next;
    }

    function wannacry(address _trigger) external {
        trigger = _trigger;
    }

}
