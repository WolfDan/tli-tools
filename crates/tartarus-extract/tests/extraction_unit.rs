use tartarus_extract::{
    DEFAULT_AES_KEY, DEFAULT_AES_KEY_HEX, ExtractError, PixelFormat, XDI_LUA_HEADER, crop_rgba8,
    decode_mip_to_rgba8, decode_png_rgba8, decrypt_aes256_ecb_in_place, decrypt_luac_bytes,
    encode_png_rgba8, parse_aes_key_hex, parse_multi_language_bin, strip_rich_text,
};

#[test]
fn aes_key_parsing_and_decryption_work() {
    let parsed = parse_aes_key_hex(DEFAULT_AES_KEY_HEX).expect("valid default hex key");
    assert_eq!(parsed, DEFAULT_AES_KEY);

    let err = parse_aes_key_hex("0x1234").expect_err("short key must fail");
    assert!(matches!(err, ExtractError::InvalidKey(_)));

    let mut block = [0xAAu8; 16];
    let original = block;
    decrypt_aes256_ecb_in_place(&mut block, &DEFAULT_AES_KEY);
    assert_ne!(block, original);
}

#[test]
fn rgba8_decode_crop_and_png_roundtrip() {
    // 2x2 BGRA8 image -> RGBA8
    let bgra = [
        10u8, 20, 30, 255, // (0,0): R=30, G=20, B=10
        40, 50, 60, 255, // (1,0): R=60, G=50, B=40
        70, 80, 90, 255, // (0,1): R=90, G=80, B=70
        100, 110, 120, 255, // (1,1): R=120, G=110, B=100
    ];
    let rgba = decode_mip_to_rgba8(&bgra, 2, 2, PixelFormat::B8G8R8A8).expect("decode BGRA8");
    assert_eq!(&rgba[0..4], &[30, 20, 10, 255]);
    assert_eq!(&rgba[12..16], &[120, 110, 100, 255]);

    let cropped = crop_rgba8(&rgba, 2, 2, 1, 1, 1, 1);
    assert_eq!(cropped, vec![120, 110, 100, 255]);

    let png_bytes = encode_png_rgba8(2, 2, &rgba).expect("encode PNG");
    let (w, h, decoded) = decode_png_rgba8(&png_bytes).expect("decode PNG");
    assert_eq!((w, h), (2, 2));
    assert_eq!(decoded, rgba);
}

#[test]
fn rich_text_stripping_and_lang_bin_parsing() {
    let raw = "<RichText.Yellow>Devour</> <Img id=\"1\"/>\\n+20%   Score";
    assert_eq!(strip_rich_text(raw), "Devour +20% Score");

    // Construct a synthetic multi-language binary record:
    // <u32 klen><key><u32 klen><key><u32 0><u32 vlen><val>
    let key = b"S16_TestKey";
    let val = b"Hello Tartarus";
    let klen = key.len() as u32;
    let vlen = val.len() as u32;
    let mut buf = Vec::new();
    buf.extend_from_slice(&klen.to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&klen.to_le_bytes());
    buf.extend_from_slice(key);
    buf.extend_from_slice(&0u32.to_le_bytes());
    buf.extend_from_slice(&vlen.to_le_bytes());
    buf.extend_from_slice(val);

    let map = parse_multi_language_bin(&buf, Some("S16_"));
    assert_eq!(
        map.get("S16_TestKey").map(String::as_str),
        Some("Hello Tartarus")
    );
}

#[test]
fn luac_decrypt_passthrough_on_decrypted_header() {
    let mut sample = XDI_LUA_HEADER.to_vec();
    sample.extend_from_slice(&[1, 2, 3, 4, 5, 6]);
    let dec = decrypt_luac_bytes(&sample);
    assert_eq!(dec, sample);
    assert_eq!(dec.len(), 16);
    assert!(dec.starts_with(XDI_LUA_HEADER));
}
