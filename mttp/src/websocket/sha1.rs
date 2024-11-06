const BLOCK_SIZE: usize = 64;
const DIGEST_SIZE: usize = 20;

// claude opus made this, i didn't wanna bother with my own sha1 impl for now
#[allow(clippy::needless_range_loop)]
pub fn sha1(data: &[u8]) -> [u8; DIGEST_SIZE] {
    let mut h0: u32 = 0x67452301;
    let mut h1: u32 = 0xEFCDAB89;
    let mut h2: u32 = 0x98BADCFE;
    let mut h3: u32 = 0x10325476;
    let mut h4: u32 = 0xC3D2E1F0;

    let mut message = data.to_vec();
    let original_len = message.len();
    message.push(0x80);

    while (message.len() % BLOCK_SIZE) != (BLOCK_SIZE - 8) {
        message.push(0x00);
    }

    let bits = original_len * 8;
    message.extend(&bits.to_be_bytes());

    for chunk in message.chunks(BLOCK_SIZE) {
        let mut w = [0u32; 80];
        for (i, &value) in chunk.iter().enumerate() {
            w[i >> 2] |= (value as u32) << ((3 - (i & 3)) << 3);
        }
        for i in 16..80 {
            let mut tmp = w[i - 3] ^ w[i - 8] ^ w[i - 14] ^ w[i - 16];
            tmp = tmp.rotate_left(1);
            w[i] = tmp;
        }

        let mut a = h0;
        let mut b = h1;
        let mut c = h2;
        let mut d = h3;
        let mut e = h4;

        for i in 0..80 {
            let (f, k) = match i {
                0..=19 => ((b & c) | (!b & d), 0x5A827999),
                20..=39 => (b ^ c ^ d, 0x6ED9EBA1),
                40..=59 => ((b & c) | (b & d) | (c & d), 0x8F1BBCDC),
                _ => (b ^ c ^ d, 0xCA62C1D6),
            };

            let temp = a
                .rotate_left(5)
                .wrapping_add(f)
                .wrapping_add(e)
                .wrapping_add(k)
                .wrapping_add(w[i]);
            e = d;
            d = c;
            c = b.rotate_left(30);
            b = a;
            a = temp;
        }

        h0 = h0.wrapping_add(a);
        h1 = h1.wrapping_add(b);
        h2 = h2.wrapping_add(c);
        h3 = h3.wrapping_add(d);
        h4 = h4.wrapping_add(e);
    }

    let mut digest = [0u8; DIGEST_SIZE];
    digest[0..4].copy_from_slice(&h0.to_be_bytes());
    digest[4..8].copy_from_slice(&h1.to_be_bytes());
    digest[8..12].copy_from_slice(&h2.to_be_bytes());
    digest[12..16].copy_from_slice(&h3.to_be_bytes());
    digest[16..20].copy_from_slice(&h4.to_be_bytes());

    digest
}

#[test]
fn sha1_test1() {
    let s = "amogus amogus amogus the voices the voices the fog is coming the fog is coming";
    let hash = sha1(s.as_bytes());

    let result = [
        0x71, 0x39, 0xad, 0xa5, 0x9a, 0x60, 0x6c, 0xce, 0xbd, 0x15, 0x56, 0x44, 0x38, 0x6b, 0xd1,
        0x13, 0xa4, 0xf0, 0x8a, 0xe5,
    ];
    assert_eq!(result, hash);
}
