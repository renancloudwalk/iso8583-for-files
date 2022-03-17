use std::convert::TryFrom;

/// Receives a payload and returns a cloned payload  without rdw or blocking
//TODO return a result instead of trying to open the vector directly
pub fn deblock_and_remove_rdw_from(payload: Vec<u8>) -> Vec<u8> {
    if has_rdw_or_block(&payload) {
        let mut new_vec: Vec<u8> = vec![];
        let mut position: usize = 0;
        let deblocked_payload = remove_blocking_chunks(payload);

        //since it's possible that the rdw slice ends 4 characters (due to rdw size)
        while let Some(calculated_rdw) = rdw_to_size(&deblocked_payload[position..position + 4]) {
            position = position + 4;
            let new_content = &deblocked_payload[position..(position + calculated_rdw)];

            new_vec.extend_from_slice(new_content);

            position = position + calculated_rdw;
        }
        new_vec
    } else {
        payload.to_vec()
    }
}

fn remove_blocking_chunks<'a>(payload: Vec<u8>) -> Vec<u8> {
    // removing @@ signs (1024 blockings)
    let mut deblocked_payload: Vec<u8> = vec![];
    let mut payload_in_chunks = payload.chunks(2);
    while let Some(two_bytes) = payload_in_chunks.next() {
        if two_bytes != &[b'@', b'@'] {
            deblocked_payload.extend_from_slice(two_bytes);
        }
    }

    deblocked_payload
}

fn has_rdw_or_block(payload: &[u8]) -> bool {
    let payload_vec = payload.to_vec();
    // checks if there is a non ascii character (rdw isn't ascii)
    let rdw_probability = payload_vec.iter().take(4).filter(|c| c.is_ascii()).count();

    // if no last byte exists the file does not exist
    let last_byte = payload_vec.last().unwrap_or(&b'0');

    //when there are non-ascii chars as in rdw and it ends with a block, it is high the probability of having rdw and @@
    rdw_probability >= 3 && last_byte == &b'@'
}

// Each subsequent byte has a potential value of 255 (because it's in ASCII)
// so a RDW of 0u8 0u8 1u8 3u8 actually means that the RDW refers to the next 258 characters
// (0 × 255³) + (0 × 255²) + (1 × 255¹) + (3 × 255⁰) = 258
fn rdw_to_size(rdw_buffer: &[u8]) -> Option<usize> {
    let s: u64 = rdw_buffer
        .iter()
        .enumerate()
        .map(|(index, rdw_number)| -> u64 {
            let index_translation = i8::abs(i8::try_from(index).unwrap() - 3i8);
            let index_power: u64 = 256u64.pow(u32::try_from(index_translation).unwrap());
            u64::try_from(*rdw_number).unwrap() * index_power
        })
        .sum();

    let final_size = usize::try_from(s).unwrap();
    if final_size > 0 {
        Some(final_size)
    } else {
        None
    }
}

#[cfg(test)]
use std::fs::File;
#[cfg(test)]
use std::io::Read;
#[cfg(test)]
fn read_file(file_name: &str) -> Vec<u8> {
    let mut file = File::open(file_name).expect("no file found");
    let metadata = std::fs::metadata(file_name).expect("unable to read metadata");

    let mut payload = vec![0; metadata.len() as usize];

    file.read(&mut payload).expect("buffer overflow");

    payload.to_vec()
}

#[test]
fn test_opening_blocked_file() {
    let file = read_file("tests/T113_sample.ipm");

    deblock_and_remove_rdw_from(file);
}

#[test]
fn test_unblocked_file_must_remain_the_same() {
    let file = read_file("tests/R111_sample.ipm");

    deblock_and_remove_rdw_from(file);
}