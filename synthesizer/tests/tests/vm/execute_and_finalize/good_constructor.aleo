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
    set 1u8 into data[0u8];

function check:
    async check into r0;
    output r0 as good_constructor.aleo/check.future;
finalize check:
    get data[0u8] into r0;
    assert.eq r0 1u8;



