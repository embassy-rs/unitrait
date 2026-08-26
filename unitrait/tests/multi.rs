//! Tests multiple opaque associated types, used at arbitrary argument positions,
//! by value, by shared and by mutable reference.

use std::sync::atomic::{AtomicU32, Ordering};

unitrait::unitrait! {
    /// A toy byte-shifting codec.
    pub trait Codec {
        /// Opaque encoder state.
        #[opaque(size = 16, align = 8)]
        #[symbol = "_unitrait_test_codec_encoder_drop"]
        pub type Encoder;

        /// Opaque decoder state.
        #[opaque(size = 16, align = 8)]
        #[symbol = "_unitrait_test_codec_decoder_drop"]
        pub type Decoder;

        /// Returns a fresh encoder adding `shift` to every byte.
        #[symbol = "_unitrait_test_codec_encoder_new"]
        pub fn encoder_new(shift: u8) -> Self::Encoder;

        /// Returns the decoder matching `enc`.
        #[symbol = "_unitrait_test_codec_decoder_for"]
        pub fn decoder_for(enc: &Self::Encoder) -> Self::Decoder;

        /// Encodes `data` in place. The opaque parameter is not in first position.
        #[symbol = "_unitrait_test_codec_encode"]
        pub fn encode(data: &mut [u8], enc: &mut Self::Encoder);

        /// Decodes `data` in place, consuming `dec`. Returns how many bytes this decoder
        /// has decoded in total, including this call.
        #[symbol = "_unitrait_test_codec_decode_last"]
        pub fn decode_last(data: &mut [u8], dec: Self::Decoder) -> u32;

        /// Returns how many bytes `enc` has encoded so far.
        #[symbol = "_unitrait_test_codec_encoded"]
        pub fn encoded(enc: &Self::Encoder) -> u32;
    }

    /// Set the global codec implementation.
    macro codec_impl(path = $crate);
}

static ENCODER_DROPS: AtomicU32 = AtomicU32::new(0);
static DECODER_DROPS: AtomicU32 = AtomicU32::new(0);

struct MyCodec;

struct MyEncoder {
    shift: u8,
    count: u32,
}

struct MyDecoder {
    shift: u8,
    count: u32,
}

impl Drop for MyEncoder {
    fn drop(&mut self) {
        ENCODER_DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl Drop for MyDecoder {
    fn drop(&mut self) {
        DECODER_DROPS.fetch_add(1, Ordering::Relaxed);
    }
}

impl Codec for MyCodec {
    type Encoder = MyEncoder;
    type Decoder = MyDecoder;

    fn encoder_new(shift: u8) -> MyEncoder {
        MyEncoder { shift, count: 0 }
    }

    fn decoder_for(enc: &MyEncoder) -> MyDecoder {
        MyDecoder {
            shift: enc.shift,
            count: 0,
        }
    }

    fn encode(data: &mut [u8], enc: &mut MyEncoder) {
        for b in data.iter_mut() {
            *b = b.wrapping_add(enc.shift);
            enc.count += 1;
        }
    }

    fn decode_last(data: &mut [u8], mut dec: MyDecoder) -> u32 {
        for b in data.iter_mut() {
            *b = b.wrapping_sub(dec.shift);
            dec.count += 1;
        }
        dec.count
    }

    fn encoded(enc: &MyEncoder) -> u32 {
        enc.count
    }
}

codec_impl!(MyCodec);

#[test]
fn test_multiple_opaque_types() {
    let mut enc = encoder_new(3);
    let dec = decoder_for(&enc);

    let mut data = *b"hello";
    encode(&mut data, &mut enc);
    assert_ne!(&data, b"hello");
    assert_eq!(encoded(&enc), 5);

    let before_enc = ENCODER_DROPS.load(Ordering::Relaxed);
    let before_dec = DECODER_DROPS.load(Ordering::Relaxed);

    // `dec` is consumed by value; the implementation drops it exactly once.
    assert_eq!(decode_last(&mut data, dec), 5);
    assert_eq!(&data, b"hello");
    assert_eq!(DECODER_DROPS.load(Ordering::Relaxed), before_dec + 1);

    drop(enc);
    assert_eq!(ENCODER_DROPS.load(Ordering::Relaxed), before_enc + 1);
}
