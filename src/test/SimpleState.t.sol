// SPDX-License-Identifier: MIT
pragma solidity 0.8.10;

import "ds-test/test.sol";
import "../SimpleState.sol";
import "./NullContract.sol";

contract SimpleStateTest is DSTest {
    SimpleState internal state;

    function setUp() public {
        state = new SimpleState();
    }

    function testSetup() public {
        assertEq(state.state(), 0);
    }

    function testStep() public {
        state.step(1);
        assertEq(state.state(), 1);
        state.step(3);
        assertEq(state.state(), 3);
        state.step(5);
        assertEq(state.state(), 5);
    }

    function testFailStepFrom0() public {
        state.step(6);
    }
    function testFailStepFrom1() public {
        state.step(1);
        state.step(0);
    }
    function testFailStepFrom3() public {
        state.step(1);
        state.step(3);
        state.step(4);
    }
    function testFailStepFrom5() public {
        state.step(1);
        state.step(3);
        state.step(5);
        state.step(0); // 5 should be terminal state
    }
    function testWannacry() public {
        address trigger = address(new NullContract());
        state.wannacry(trigger);
        assertEq(state.trigger(), trigger);
        state.step(1);
        state.step(3);
        state.step(5);
        state.step(6);
        assertEq(state.state(), 6);
    }
}
