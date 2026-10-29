use crate::core::error::{Result, RpgError};
use std::fs;
use std::io::Write;
use std::path::Path;

const fn make_ogg_crc_table() -> [u32; 256] {
    let mut table = [0u32; 256];
    let mut i = 0usize;
    while i < 256 {
        let mut r = (i as u32) << 24;
        let mut j = 0;
        while j < 8 {
            if (r & 0x8000_0000) != 0 {
                r = (r << 1) ^ 0x04C1_1DB7;
            } else {
                r <<= 1;
            }
            j += 1;
        }
        table[i] = r;
        i += 1;
    }
    table
}

static OGG_CRC_TABLE: [u32; 256] = make_ogg_crc_table();

pub fn ogg_crc(data: &[u8]) -> u32 {
    let mut crc = 0u32;
    for &b in data {
        let idx = ((crc >> 24) as u8 ^ b) as usize;
        crc = (crc << 8) ^ OGG_CRC_TABLE[idx];
    }
    crc
}

#[derive(Debug, Clone)]
pub struct OggPage {
    pub header_type: u8,
    pub granule_pos: i64,
    pub serial_num: u32,
    pub seq_num: u32,
    pub segment_table: Vec<u8>,
    pub data: Vec<u8>,
}

impl OggPage {
    pub fn write_page<W: Write>(&self, w: &mut W) -> Result<()> {
        let mut header = Vec::with_capacity(27 + self.segment_table.len());
        header.extend_from_slice(b"OggS");
        header.push(0); // version
        header.push(self.header_type);
        header.extend_from_slice(&self.granule_pos.to_le_bytes());
        header.extend_from_slice(&self.serial_num.to_le_bytes());
        header.extend_from_slice(&self.seq_num.to_le_bytes());
        header.extend_from_slice(&[0, 0, 0, 0]); // temporary crc
        header.push(self.segment_table.len() as u8);
        header.extend_from_slice(&self.segment_table);

        // Compute CRC over header + data
        let mut crc = 0u32;
        for &b in &header {
            let idx = ((crc >> 24) as u8 ^ b) as usize;
            crc = (crc << 8) ^ OGG_CRC_TABLE[idx];
        }
        for &b in &self.data {
            let idx = ((crc >> 24) as u8 ^ b) as usize;
            crc = (crc << 8) ^ OGG_CRC_TABLE[idx];
        }

        // Write CRC at offsets 22..26
        header[22..26].copy_from_slice(&crc.to_le_bytes());

        w.write_all(&header).map_err(RpgError::Io)?;
        w.write_all(&self.data).map_err(RpgError::Io)?;
        Ok(())
    }
}

pub fn parse_ogg_pages(data: &[u8]) -> Result<Vec<OggPage>> {
    let mut cursor = 0;
    let mut pages = Vec::new();

    while cursor + 27 <= data.len() {
        if &data[cursor..cursor + 4] != b"OggS" {
            break;
        }

        let header_type = data[cursor + 5];
        let granule_pos = i64::from_le_bytes(data[cursor + 6..cursor + 14].try_into().unwrap());
        let serial_num = u32::from_le_bytes(data[cursor + 14..cursor + 18].try_into().unwrap());
        let seq_num = u32::from_le_bytes(data[cursor + 18..cursor + 22].try_into().unwrap());
        let seg_count = data[cursor + 26] as usize;

        if cursor + 27 + seg_count > data.len() {
            break;
        }

        let seg_table = data[cursor + 27..cursor + 27 + seg_count].to_vec();
        let payload_len: usize = seg_table.iter().map(|&x| x as usize).sum();

        let data_start = cursor + 27 + seg_count;
        if data_start + payload_len > data.len() {
            break;
        }

        let page_data = data[data_start..data_start + payload_len].to_vec();

        pages.push(OggPage {
            header_type,
            granule_pos,
            serial_num,
            seq_num,
            segment_table: seg_table,
            data: page_data,
        });

        cursor = data_start + payload_len;
    }

    Ok(pages)
}

pub fn contains_vorbis_picture_metadata(data: &[u8]) -> bool {
    let scan_len = data.len().min(16 * 1024 * 1024);
    let slice = &data[..scan_len];
    slice
        .windows(22)
        .any(|w| w.eq_ignore_ascii_case(b"METADATA_BLOCK_PICTURE"))
        || slice
            .windows(8)
            .any(|w| w.eq_ignore_ascii_case(b"COVERART"))
}

pub fn clean_vorbis_comment_packet(packet: &[u8]) -> Option<Vec<u8>> {
    if packet.len() < 7 || &packet[..7] != b"\x03vorbis" {
        return None;
    }

    let mut cursor = 7;
    if cursor + 4 > packet.len() {
        return None;
    }
    let vendor_len = u32::from_le_bytes(packet[cursor..cursor + 4].try_into().unwrap()) as usize;
    cursor += 4;
    if cursor + vendor_len > packet.len() {
        return None;
    }
    let vendor_str = &packet[cursor..cursor + vendor_len];
    cursor += vendor_len;

    if cursor + 4 > packet.len() {
        return None;
    }
    let comment_count = u32::from_le_bytes(packet[cursor..cursor + 4].try_into().unwrap()) as usize;
    cursor += 4;

    let mut preserved_comments: Vec<Vec<u8>> = Vec::new();
    let mut stripped_any = false;

    for _ in 0..comment_count {
        if cursor + 4 > packet.len() {
            return None;
        }
        let comment_len =
            u32::from_le_bytes(packet[cursor..cursor + 4].try_into().unwrap()) as usize;
        cursor += 4;
        if cursor + comment_len > packet.len() {
            return None;
        }

        let comment = &packet[cursor..cursor + comment_len];
        cursor += comment_len;

        let is_picture = (comment.len() >= 23
            && comment[..23].eq_ignore_ascii_case(b"METADATA_BLOCK_PICTURE="))
            || (comment.len() >= 9 && comment[..9].eq_ignore_ascii_case(b"COVERART="))
            || (comment.len() >= 13 && comment[..13].eq_ignore_ascii_case(b"COVERARTMIME="));

        if is_picture {
            stripped_any = true;
        } else {
            preserved_comments.push(comment.to_vec());
        }
    }

    // Check framing bit
    if cursor >= packet.len() || (packet[cursor] & 0x01) == 0 {
        return None;
    }

    if !stripped_any {
        return None; // No bloat found
    }

    let mut new_packet = Vec::with_capacity(packet.len());
    new_packet.extend_from_slice(b"\x03vorbis");
    new_packet.extend_from_slice(&(vendor_len as u32).to_le_bytes());
    new_packet.extend_from_slice(vendor_str);

    new_packet.extend_from_slice(&(preserved_comments.len() as u32).to_le_bytes());
    for comm in &preserved_comments {
        new_packet.extend_from_slice(&(comm.len() as u32).to_le_bytes());
        new_packet.extend_from_slice(comm);
    }
    // Framing bit (RFC 3533 / Vorbis I specification)
    new_packet.push(1);
    // Post-framing-bit zero padding is stripped by not copying any trailing bytes

    Some(new_packet)
}

pub fn optimize_ogg_file(file_path: &Path) -> Result<bool> {
    let data = match fs::read(file_path) {
        Ok(d) => d,
        Err(_) => return Ok(false),
    };

    // Fast-path scan: skip expensive Ogg page parsing if no picture metadata exists
    if !contains_vorbis_picture_metadata(&data) {
        return Ok(false);
    }

    let mut pages = parse_ogg_pages(&data)?;
    if pages.len() < 2 {
        return Ok(false);
    }

    // Page 0 is identification header. Page 1 must start with \x03vorbis (comment header).
    if !pages[1].data.starts_with(b"\x03vorbis") {
        return Ok(false);
    }

    // Multi-page Vorbis comment reassembly:
    // Accumulate packet segments starting from Page 1 until a segment < 255 is encountered.
    let mut packet_data = Vec::new();
    let mut end_page = 1;
    let mut end_seg_idx = 0;
    let mut packet_found = false;

    for (p_idx, page) in pages.iter().enumerate().skip(1) {
        let mut offset = 0;
        for (s_idx, &seg) in page.segment_table.iter().enumerate() {
            let seg_len = seg as usize;
            if offset + seg_len > page.data.len() {
                return Ok(false);
            }
            packet_data.extend_from_slice(&page.data[offset..offset + seg_len]);
            offset += seg_len;
            if seg < 255 {
                end_page = p_idx;
                end_seg_idx = s_idx;
                packet_found = true;
                break;
            }
        }
        if packet_found {
            break;
        }
    }

    if !packet_found {
        return Ok(false);
    }

    let new_comment = match clean_vorbis_comment_packet(&packet_data) {
        Some(c) => c,
        None => return Ok(false),
    };

    // Extract remaining segments and data on end_page that belong to subsequent packet(s)
    let end_p = &pages[end_page];
    let packet_bytes_in_end_page: usize = end_p.segment_table[..=end_seg_idx]
        .iter()
        .map(|&s| s as usize)
        .sum();
    let remaining_segs = end_p.segment_table[end_seg_idx + 1..].to_vec();
    let remaining_data = end_p.data[packet_bytes_in_end_page..].to_vec();

    // Re-segment new_comment (each segment <= 255 bytes)
    let mut all_segs = Vec::new();
    let mut rem = new_comment.len();
    while rem >= 255 {
        all_segs.push(255u8);
        rem -= 255;
    }
    all_segs.push(rem as u8);
    // Append remaining segments from end_page
    all_segs.extend_from_slice(&remaining_segs);

    let mut all_payload = new_comment;
    all_payload.extend_from_slice(&remaining_data);

    // Build replacement Ogg page(s), each with at most 255 segments
    let serial = pages[1].serial_num;
    let granule = pages[end_page].granule_pos;
    let mut new_pages = Vec::new();
    let mut seg_cursor = 0;
    let mut data_cursor = 0;
    let mut first_new_page = true;

    while seg_cursor < all_segs.len() {
        let chunk_len = (all_segs.len() - seg_cursor).min(255);
        let chunk_segs = all_segs[seg_cursor..seg_cursor + chunk_len].to_vec();
        let chunk_data_len: usize = chunk_segs.iter().map(|&s| s as usize).sum();
        let chunk_data = all_payload[data_cursor..data_cursor + chunk_data_len].to_vec();

        let header_type = if first_new_page {
            pages[1].header_type & !0x01 // Not a continuation page
        } else {
            0x01 // Continuation page
        };
        first_new_page = false;

        new_pages.push(OggPage {
            header_type,
            granule_pos: granule,
            serial_num: serial,
            seq_num: 0, // Assigned sequentially below
            segment_table: chunk_segs,
            data: chunk_data,
        });

        seg_cursor += chunk_len;
        data_cursor += chunk_data_len;
    }

    // Replace original comment pages [1..=end_page] with the new page(s)
    pages.splice(1..=end_page, new_pages);

    // Renumber sequence numbers sequentially according to RFC 3533
    for (idx, page) in pages.iter_mut().enumerate() {
        page.seq_num = idx as u32;
    }

    // Write all pages with freshly computed CRC32 checksums
    let mut out_buffer = Vec::with_capacity(data.len());
    for p in &pages {
        p.write_page(&mut out_buffer)?;
    }

    if out_buffer.len() < data.len() {
        fs::write(file_path, out_buffer).map_err(RpgError::Io)?;
        return Ok(true);
    }

    Ok(false)
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ogg_crc_empty() {
        let crc = ogg_crc(&[]);
        assert_eq!(crc, 0);
    }

    #[test]
    fn test_clean_vorbis_comment_strips_picture() {
        let mut packet = Vec::new();
        packet.extend_from_slice(b"\x03vorbis");
        let vendor = b"TestVendor";
        packet.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        packet.extend_from_slice(vendor);

        let comment1: &[u8] = b"LOOPSTART=500";
        let comment2: &[u8] = b"METADATA_BLOCK_PICTURE=fake_base64_blob_here";
        let comment3: &[u8] = b"LOOPLENGTH=1200";

        packet.extend_from_slice(&3u32.to_le_bytes()); // 3 comments
        for c in [comment1, comment2, comment3] {
            packet.extend_from_slice(&(c.len() as u32).to_le_bytes());
            packet.extend_from_slice(c);
        }
        packet.push(1); // framing bit

        let cleaned = clean_vorbis_comment_packet(&packet).expect("Should strip picture bloat");
        assert!(cleaned.len() < packet.len());

        // Verify remaining comments
        let cleaned_str = String::from_utf8_lossy(&cleaned);
        assert!(cleaned_str.contains("LOOPSTART=500"));
        assert!(cleaned_str.contains("LOOPLENGTH=1200"));
        assert!(!cleaned_str.contains("METADATA_BLOCK_PICTURE"));
    }

    #[test]
    fn test_clean_vorbis_comment_no_change_when_no_bloat() {
        let mut packet = Vec::new();
        packet.extend_from_slice(b"\x03vorbis");
        let vendor = b"TestVendor";
        packet.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        packet.extend_from_slice(vendor);

        let comment = b"TITLE=Theme";
        packet.extend_from_slice(&1u32.to_le_bytes());
        packet.extend_from_slice(&(comment.len() as u32).to_le_bytes());
        packet.extend_from_slice(comment);
        packet.push(1);

        assert!(clean_vorbis_comment_packet(&packet).is_none());
    }

    #[test]
    fn test_multipage_ogg_comment_stripping_and_crc() {
        let temp_dir = tempfile::tempdir().unwrap();
        let ogg_path = temp_dir.path().join("test_multipage.ogg");

        // Construct a synthetic multi-page Ogg file:
        // Page 0: Identification header (BOS)
        let page0 = OggPage {
            header_type: 0x02, // BOS
            granule_pos: 0,
            serial_num: 12345,
            seq_num: 0,
            segment_table: vec![30],
            data: vec![0x01; 30],
        };

        // Create a Vorbis comment packet spanning > 65 KB with METADATA_BLOCK_PICTURE
        let mut comment_pkt = Vec::new();
        comment_pkt.extend_from_slice(b"\x03vorbis");
        let vendor = b"TestVendor";
        comment_pkt.extend_from_slice(&(vendor.len() as u32).to_le_bytes());
        comment_pkt.extend_from_slice(vendor);

        let c1 = b"TITLE=My Song";
        let mut big_pic = Vec::with_capacity(70000);
        big_pic.extend_from_slice(b"METADATA_BLOCK_PICTURE=");
        big_pic.resize(70000, b'A');

        comment_pkt.extend_from_slice(&2u32.to_le_bytes()); // 2 comments
        comment_pkt.extend_from_slice(&(c1.len() as u32).to_le_bytes());
        comment_pkt.extend_from_slice(c1);
        comment_pkt.extend_from_slice(&(big_pic.len() as u32).to_le_bytes());
        comment_pkt.extend_from_slice(&big_pic);
        comment_pkt.push(1); // Framing bit

        // Segment the packet across Page 1 and Page 2
        let mut all_segs = Vec::new();
        let mut rem = comment_pkt.len();
        while rem >= 255 {
            all_segs.push(255u8);
            rem -= 255;
        }
        all_segs.push(rem as u8);

        // Page 1 gets first 255 segments (65025 bytes)
        let page1_segs = all_segs[..255].to_vec();
        let page1_data = comment_pkt[..65025].to_vec();

        // Page 2 gets remaining segments + a dummy setup header
        let page2_segs = all_segs[255..].to_vec();
        let page2_data = comment_pkt[65025..].to_vec();

        let page1 = OggPage {
            header_type: 0x00,
            granule_pos: 0,
            serial_num: 12345,
            seq_num: 1,
            segment_table: page1_segs,
            data: page1_data,
        };

        let page2 = OggPage {
            header_type: 0x01, // Continuation
            granule_pos: 0,
            serial_num: 12345,
            seq_num: 2,
            segment_table: page2_segs,
            data: page2_data,
        };

        // Page 3: Audio data packet (EOS)
        let page3 = OggPage {
            header_type: 0x04, // EOS
            granule_pos: 44100,
            serial_num: 12345,
            seq_num: 3,
            segment_table: vec![50],
            data: vec![0x42; 50],
        };

        let mut raw_bytes = Vec::new();
        page0.write_page(&mut raw_bytes).unwrap();
        page1.write_page(&mut raw_bytes).unwrap();
        page2.write_page(&mut raw_bytes).unwrap();
        page3.write_page(&mut raw_bytes).unwrap();
        fs::write(&ogg_path, &raw_bytes).unwrap();

        let orig_len = raw_bytes.len();
        let optimized = optimize_ogg_file(&ogg_path).unwrap();
        assert!(optimized, "Multi-page Vorbis comment should be optimized");

        let new_bytes = fs::read(&ogg_path).unwrap();
        assert!(
            new_bytes.len() < orig_len - 60000,
            "Should have stripped > 60KB"
        );

        // Parse resulting pages and verify sequential sequence numbers and valid CRCs
        let cleaned_pages = parse_ogg_pages(&new_bytes).unwrap();
        for (idx, p) in cleaned_pages.iter().enumerate() {
            assert_eq!(
                p.seq_num, idx as u32,
                "Page sequence numbers must be contiguous"
            );
        }
        // Verify CRC is correct by re-writing and checking byte equality
        let mut re_written = Vec::new();
        for p in &cleaned_pages {
            p.write_page(&mut re_written).unwrap();
        }
        assert_eq!(
            new_bytes, re_written,
            "CRC32 recomputation must match written pages"
        );
    }
}
