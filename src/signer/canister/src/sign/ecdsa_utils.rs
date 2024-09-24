// Converts a SEC1 ECDSA signature to the DER format.
pub fn sec1_to_der(sec1_signature: &[u8]) -> Vec<u8> {
    let r: Vec<u8> = if sec1_signature[0] & 0x80 != 0 {
        // r is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[..32].to_vec());
        tmp
    } else {
        // r is positive.
        sec1_signature[..32].to_vec()
    };

    let s: Vec<u8> = if sec1_signature[32] & 0x80 != 0 {
        // s is negative. Prepend a zero byte.
        let mut tmp = vec![0x00];
        tmp.extend(sec1_signature[32..].to_vec());
        tmp
    } else {
        // s is positive.
        sec1_signature[32..].to_vec()
    };

    let r_len = u8::try_from(r.len()).expect("Failed to convert r length to u8");
    let s_len = u8::try_from(s.len()).expect("Failed to convert s length to u8");

    // Convert signature to DER.
    vec![
        vec![0x30, 4 + r_len + s_len, 0x02, r_len],
        r,
        vec![0x02, s_len],
        s,
    ]
    .into_iter()
    .flatten()
    .collect()
}
