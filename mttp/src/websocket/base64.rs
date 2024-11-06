const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

// Written by claude opus
pub fn encode(input: &[u8]) -> String {
    let mut encoded = String::new();
    let mut i = 0;

    while i < input.len() {
        let mut chunk = [0u8; 3];
        for j in 0..3 {
            if i + j < input.len() {
                chunk[j] = input[i + j];
            }
        }

        let b0 = chunk[0] >> 2;
        let b1 = ((chunk[0] & 0b00000011) << 4) | (chunk[1] >> 4);
        let b2 = ((chunk[1] & 0b00001111) << 2) | (chunk[2] >> 6);
        let b3 = chunk[2] & 0b00111111;

        encoded.push(BASE64_CHARS[b0 as usize] as char);
        encoded.push(BASE64_CHARS[b1 as usize] as char);

        if i + 1 < input.len() {
            encoded.push(BASE64_CHARS[b2 as usize] as char);
        } else {
            encoded.push('=');
        }

        if i + 2 < input.len() {
            encoded.push(BASE64_CHARS[b3 as usize] as char);
        } else {
            encoded.push('=');
        }

        i += 3;
    }

    encoded
}

#[test]
fn base_64_dec_test1() {
    let s = "amogus amogus amogus the voices the voices the fog is coming the fog is coming";
    let encoded = encode(s.as_bytes());

    let expected = "YW1vZ3VzIGFtb2d1cyBhbW9ndXMgdGhlIHZvaWNlcyB0aGUgdm9pY2VzIHRoZSBmb2cgaXMgY29taW5nIHRoZSBmb2cgaXMgY29taW5n";

    assert_eq!(encoded, expected);
}

#[test]
fn base_64_dec_test2() {
    let s = "Lorem ipsum dolor sit amet consectetur adipisicing elit. Veniam neque in tempore sunt ut aliquid, laudantium, sapiente error soluta animi eveniet facere laboriosam fugit optio autem odio harum, vitae eligendi sint esse quasi. Vero, dolores quidem ad ipsa beatae sequi quibusdam! Facere animi accusantium quae, quo optio nobis quidem in.";
    let encoded = encode(s.as_bytes());

    let expected = "TG9yZW0gaXBzdW0gZG9sb3Igc2l0IGFtZXQgY29uc2VjdGV0dXIgYWRpcGlzaWNpbmcgZWxpdC4gVmVuaWFtIG5lcXVlIGluIHRlbXBvcmUgc3VudCB1dCBhbGlxdWlkLCBsYXVkYW50aXVtLCBzYXBpZW50ZSBlcnJvciBzb2x1dGEgYW5pbWkgZXZlbmlldCBmYWNlcmUgbGFib3Jpb3NhbSBmdWdpdCBvcHRpbyBhdXRlbSBvZGlvIGhhcnVtLCB2aXRhZSBlbGlnZW5kaSBzaW50IGVzc2UgcXVhc2kuIFZlcm8sIGRvbG9yZXMgcXVpZGVtIGFkIGlwc2EgYmVhdGFlIHNlcXVpIHF1aWJ1c2RhbSEgRmFjZXJlIGFuaW1pIGFjY3VzYW50aXVtIHF1YWUsIHF1byBvcHRpbyBub2JpcyBxdWlkZW0gaW4u";

    assert_eq!(encoded, expected);
}
