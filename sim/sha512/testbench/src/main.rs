#![no_std]
#![no_main]

use sim_bios::sim_test;
extern crate volatile;
use volatile::Volatile;

use hal_sha512::hal_sha512::*;

const kData: &[u8; 142] = b"Every one suspects himself of at least one of the cardinal virtues, and this is mine: I am one of the few honest people that I have ever known";
const kExpectedDigest: [u64; 8] = [0x02fc78c0d16b727a, 0x18570a3279e6c97b, 0x113b8871b2e92051, 0x4c0947b20169fedf, 0x1a67094ad04ad031, 0xab5f8cc340125001, 0xffbd7d7af36d3a3a, 0xf7e8465d73bbd86d];
// 7a726bd1c078fc02 7bc9e679320a5718 5120e9b271883b11 dffe6901b247094c 31d04ad04a09671a 01501240c38c5fab 3a3a6df37a7dbdff 6dd8bb735d46e8f7

// allocate a global, unsafe static string. You can use this to force writes to RAM.
#[used] // This is necessary to keep DBGSTR from being optimized out
static mut DBGSTR: [u32; 8] = [0, 0, 0, 0, 0, 0, 0, 0];

fn report(p: &pac::Peripherals, data: u32) {
    unsafe{ p.SIMSTATUS.report.write(|w| w.bits(data)); }
}

#[sim_test]
fn run(p: &pac::Peripherals) {
    let ram_ptr: *mut u32 = 0x0100_0000 as *mut u32;
    let ram = ram_ptr as *mut Volatile<u32>;

    // example of using the DBGSTR to stash a variable from a raw pointer
    unsafe {
        DBGSTR[0] = (*(ram.add(4))).read();
    };

    report(p, 0x1000_0000);
    let mut sha512: BtSha512 = BtSha512::new();
    report(p, 0x1000_0001);
    sha512.config = Sha512Config::ENDIAN_SWAP | Sha512Config::DIGEST_SWAP | Sha512Config::SHA512_EN; // Sha2Config::HMAC_EN; // Sha2Config::SHA256_EN;

    report(p, 0x1000_0002);
    sha512.init();
    report(p, 0x1000_0003);
    sha512.update(kData);
    report(p, 0x1000_0004);
    let mut digest: [u64; 8] = [0; 8];
    sha512.digest(&mut digest);
    report(p, 0x1000_0005);

    let mut pass: bool = true;
    for i in 0..8 {
        report(p, digest[i] as u32);
        report(p, kExpectedDigest[i] as u32);
        report(p, (digest[i] >> 32) as u32);
        report(p, (kExpectedDigest[i] >> 32) as u32);
        if digest[i] != kExpectedDigest[i] {
            pass = false;
        }
    }
    report(p, 0x1000_0006);

    // set success to indicate to the CI framework that the test has passed
    p.SIMSTATUS.simstatus.modify(|_r, w| w.success().bit(pass));
}