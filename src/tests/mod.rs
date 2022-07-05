

#[cfg(test)]
mod tests {
    use bitreader::BitReader;
    use bytes::BytesMut;
    use crate::decoder::{Decoder, FixedHeaderDecoder};
    use crate::encoder::{Encoder, FixedHeaderEncoder};
    use log::{debug, info, error, trace, warn};

    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_variable_byte_integer() {
        let numbers:Vec<u64> = vec![0, 1, 127, 128, 129, 300, 1000];
        for number in numbers{
            info!("Testing with number: {}", number);
            let mut encoded_buffer = BytesMut::new();
            let encoder = FixedHeaderEncoder::new().write_variable_byte_integer(number, &mut encoded_buffer);
            let mut reader = BitReader::new(&encoded_buffer);
            let mut decoder = FixedHeaderDecoder::new();
            let decoded_number = decoder.read_variable_byte_integer(&mut reader).unwrap();
            assert_eq!(decoded_number, number);
        }

    }

}