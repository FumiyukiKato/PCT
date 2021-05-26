#![allow(dead_code)]
#![allow(non_snake_case)]
#![allow(non_upper_case_globals)]

const L8: u64 = 0x0101010101010101; // Every lowest 8th bit set: 00000001...;
const G2: u64 = 0xAAAAAAAAAAAAAAAA; // Every highest 2nd bit: 101010...;
const G4: u64 = 0x3333333333333333; // 00110011 ... used to group the sum of 4 bits.;
const G8: u64 = 0x0F0F0F0F0F0F0F0F;
const H8: u64 = 0x8080808080808080;
const L9: u64 = 0x0040201008040201;
const H9: u64 = L9 << 8;
const L16: u64 = 0x0001000100010001;
const H16: u64 = 0x8000800080008000;

const ONES_STEP_4: u64 = 0x1111111111111111;
const ONES_STEP_8: u64 = 0x0101010101010101;
const ONES_STEP_9: u64 = 1 << 0 | 1 << 9 | 1 << 18 | 1 << 27 | 1 << 36 | 1 << 45 | 1 << 54;
const ONES_STEP_16: u64 = 1 << 0 | 1 << 16 | 1 << 32 | 1 << 48;
const MSBS_STEP_4: u64 = 0x8 * ONES_STEP_4;
const MSBS_STEP_8: u64 = 0x80 * ONES_STEP_8;
const MSBS_STEP_9: u64 = 0x100 * ONES_STEP_9;
const MSBS_STEP_16: u64 = 0x8000 * ONES_STEP_16;
const INCR_STEP_8: u64 =
    0x80 << 56 | 0x40 << 48 | 0x20 << 40 | 0x10 << 32 | 0x8 << 24 | 0x4 << 16 | 0x2 << 8 | 0x1;

const ONES_STEP_32: u64 = 0x0000000100000001;
const MSBS_STEP_32: u64 = 0x8000000080000000;

#[inline]
fn COMPARE_STEP_8(x: u64, y: u64) -> u64 {
    (((((x) | MSBS_STEP_8) - ((y) & !MSBS_STEP_8)) ^ (x) ^ !(y)) & MSBS_STEP_8) >> 7
}

#[inline]
fn LEQ_STEP_8(x: u64, y: u64) -> u64 {
    (((((y) | MSBS_STEP_8) - ((x) & !MSBS_STEP_8)) ^ (x) ^ (y)) & MSBS_STEP_8) >> 7
}

#[inline]
fn UCOMPARE_STEP_9(x: u64, y: u64) -> u64 {
    ((((((x) | MSBS_STEP_9) - ((y) & !MSBS_STEP_9)) | (x ^ y)) ^ (x | !y)) & MSBS_STEP_9) >> 8
}

#[inline]
fn UCOMPARE_STEP_16(x: u64, y: u64) -> u64 {
    ((((((x) | MSBS_STEP_16) - ((y) & !MSBS_STEP_16)) | (x ^ y)) ^ (x | !y)) & MSBS_STEP_16) >> 15
}

#[inline]
fn ULEQ_STEP_9(x: u64, y: u64) -> u64 {
    ((((((y) | MSBS_STEP_9) - ((x) & !MSBS_STEP_9)) | (x ^ y)) ^ (x & !y)) & MSBS_STEP_9) >> 8
}

#[inline]
fn ULEQ_STEP_16(x: u64, y: u64) -> u64 {
    ((((((y) | MSBS_STEP_16) - ((x) & !MSBS_STEP_16)) | (x ^ y)) ^ (x & !y)) & MSBS_STEP_16) >> 15
}

#[inline]
fn ZCOMPARE_STEP_8(x: u64) -> u64 {
    ((x | ((x | MSBS_STEP_8) - ONES_STEP_8)) & MSBS_STEP_8) >> 7
}

#[inline]
fn suxpopcount(mut x: u64) -> u64 {
    // Step 1:  00 - 00 = 0;  01 - 00 = 01; 10 - 01 = 01; 11 - 01 = 10;
    x = x - ((x & G2) >> 1);
    // step 2:  add 2 groups of 2.
    x = (x & G4) + ((x >> 2) & G4);
    // 2 groups of 4.
    x = (x + (x >> 4)) & G8;
    // Using a multiply to collect the 8 groups of 8 together.
    x = x * L8 >> 56;
    return x;
}

const popcountsize: u64 = 64;
const popcountmask: u64 = popcountsize - 1;

// x86-64 only
#[inline]
pub fn popcount_linear(bits: &[u64], x: u64, nbits: u64) -> i32 {
    if nbits == 0 {
        return 0;
    }
    let lastword: u64 = (nbits - 1) / popcountsize;
    let mut p: i32 = 0;

    unsafe {
        let bits_raw_pointer = bits.as_ptr();
        core::intrinsics::prefetch_read_data(bits_raw_pointer.offset((x + 7) as isize), 0);
        // TODO; SGX
        // https://teaclave.apache.org/api-docs/sgx-sdk/sgx_tstd/intrinsics/fn.prefetch_read_data.html
        for i in 0..lastword {
            // TODO; SGX
            p += core::arch::x86_64::_popcnt64(bits[(x + i) as usize] as i64);
        }
        // 'nbits' may or may not fall on a multiple of 64 boundary,
        // so we may need to zero out the right side of the last word
        // (accomplished by shifting it right, since we're just popcounting)
        let lastshifted: u64 = bits[(x + lastword) as usize] >> (63 - ((nbits - 1) & popcountmask));
        p += core::arch::x86_64::_popcnt64(lastshifted as i64);
    }
    return p;
}


pub fn select64_popcount_search(mut x: u64, mut k: i32) -> i32 {
    let mut loc: i32 = -1;
    // if (k > popcount(x)) { return -1; }
    let mut testbits: i32 = 32;
    unsafe {
        while testbits > 0 {
            let lcount = core::arch::x86_64::_popcnt64((x >> testbits) as i64);
            if k > lcount {
                x &= (1u64 << testbits) - 1;
                loc += testbits;
                k -= lcount;
            } else {
                x >>= testbits;
            }
            testbits >>= 1
        }
    }
    return loc + k
}