// SPDX-License-Identifier: MIT
pragma solidity 0.8.10;

import "ds-test/test.sol";
import "../SimpleState.sol";

contract SimpleStateTest is DSTest {
    SimpleState internal state;

    function setUp() public {
        state = new SimpleState();
    }

    function testSetup() public {
        assertEq(state.state(), 0);
    }

    function testMove() public {
        state.move(1);
        assertEq(state.state(), 1);
        state.move(3);
        assertEq(state.state(), 3);
        state.move(5);
        assertEq(state.state(), 5);
    }

    function testFailMoveFrom0() public {
        state.move(6);
    }
    function testFailMoveFrom1() public {
        state.move(1);
        state.move(0);
    }
    function testFailMoveFrom3() public {
        state.move(1);
        state.move(3);
        state.move(4);
    }
    function testFailMoveFrom5() public {
        state.move(1);
        state.move(3);
        state.move(5);
        state.move(0); // 5 should be terminal state
    }
}
