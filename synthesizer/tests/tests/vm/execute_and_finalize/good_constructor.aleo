/*
randomness: 45791624
cases:
  - program: good_constructor.aleo
    function: check
    inputs: []
*/

import credits.aleo;

program good_constructor.aleo;

mapping data:
    key as u8.public;
    value as u8.public;

constructor:
    assert.eq edition 0u16;
    assert.eq credits.aleo/edition 0u16;
    assert.neq checksum 0field;
    assert.eq credits.aleo/checksum 6192738754253668739186185034243585975029374333074931926190215457304721124008field;
    contains data[0u8] into r0; // Check `contains` without value
    assert.eq r0 false;
    get.or_use data[0u8] 8u8 into r1; // Check `get.or_use` without value
    assert.eq r1 8u8;
    set 1u8 into data[0u8]; // Check `set`
    contains data[0u8] into r2; // Check `contains` with value
    assert.eq r2 true;
    get.or_use data[0u8] 0u8 into r3; // Check `get.or_use` without value
    assert.eq r3 1u8;
    get data[0u8] into r4; // Check `get` with value
    assert.eq r4 1u8;
    remove data[0u8]; // Check `remove`
    contains data[0u8] into r5; // Check `contains` after removal
    assert.eq r5 false;
    set 1u8 into data[0u8]; // Final set


function check:
    async check into r0;
    output r0 as good_constructor.aleo/check.future;
finalize check:
    get data[0u8] into r0;
    assert.eq r0 1u8;



