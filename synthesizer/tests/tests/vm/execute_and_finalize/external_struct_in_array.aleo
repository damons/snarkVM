/*
start_height: 19 
randomness: 73488219
cases:
  - program: committee_array_test.aleo
    function: main
    inputs: [true, 5u8]
*/

import credits.aleo;

program committee_array_test.aleo;

// Check two arrays of committee_state for element-wise equality
closure check_array:
    input r0 as [credits.aleo/committee_state; 2u32];
    input r1 as [credits.aleo/committee_state; 2u32];

    // Inline field comparisons for element 0
    assert.eq r0[0u32].is_open     r1[0u32].is_open;
    assert.eq r0[0u32].commission  r1[0u32].commission;

    // Inline field comparisons for element 1
    assert.eq r0[1u32].is_open     r1[1u32].is_open;
    assert.eq r0[1u32].commission  r1[1u32].commission;

// Main function: constructs two identical arrays and checks them
function main:
    input r0 as boolean.private;
    input r1 as u8.private;

    // Build two identical committee_state structs
    cast r0 r1 into r2 as credits.aleo/committee_state;
    cast r0 r1 into r3 as credits.aleo/committee_state;

    // Construct arrays (length uses u32)
    cast r2 r3 into r4 as [credits.aleo/committee_state; 2u32];
    cast r2 r3 into r5 as [credits.aleo/committee_state; 2u32];

    // Compare the array elements explicitly
    call check_array r4 r5;

constructor:
    assert.eq edition 0u16;

