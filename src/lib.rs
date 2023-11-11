use std::io::prelude::*;
use std::io::BufReader;
use byteorder::{LittleEndian, BigEndian, ReadBytesExt};

type Byte = u8;
type Word = u16;
type LongWord = u32;

#[derive(Debug, Copy, Clone)]
pub enum Type {
    RSID,
    PSID,
    Unknown,
}

#[derive(Debug, Copy, Clone)]
pub enum Version {
    V1,
    V2,
    V3,
    V4,
}

#[derive(Debug, Copy, Clone)]
pub enum Format {
    InternalPlayer,
    MusData,
}

#[derive(Debug, Copy, Clone)]
pub enum PlaySID {
    C64,
    PlaySID,
}

#[derive(Debug, Copy, Clone)]
pub enum Clock {
    Unknown,
    PAL,
    NTSC,
    Both,
}

#[derive(Debug, Copy, Clone)]
pub enum ChipModel {
    Unknown,
    MOS6581,
    MOS8580,
    Both,
}

#[derive(Debug, Copy, Clone)]
pub struct Flags {
    pub format: Format,
    pub play_sid: PlaySID,
    pub clock: Clock,
    pub sid_model: ChipModel,
    pub second_sid_model: ChipModel,
    pub third_sid_model: ChipModel,
}

#[derive(Debug, Clone)]
pub struct SidFile {
    pub file_type: Type,
    pub version: Version,
    pub data_offset: Word,  // +06    Word dataOffset
    pub load_address: Word, // +08    Word loadAddress
    pub init_address: Word, // +0A    Word initAddress
    pub play_address: Word, // +0C    Word playAddress
    pub songs: Word,        // +0E    Word songs
    pub start_song: Word,   // +10    Word startSong
    pub speed: LongWord,    // +12    LongWord speed
    pub name: String,       // +16    String name
    pub author: String,     // +36    String author
    pub released: String,   // +56    String released
    pub flags: Option<Flags>,      // +76    Word flags
    pub start_page: Option<Byte>,         // +78    Byte start_page
    pub page_length: Option<Byte>,        // +79    Byte page_length
    pub second_sid_address: Option<Byte>, // +7A    Byte second_SID_address
    pub third_sid_address: Option<Byte>,  // +7C    Byte third_SID_address
    pub real_load_address: Word,
    pub data: Vec<Byte>,
}

impl SidFile {
    pub fn parse(data: &[u8]) -> Result<SidFile, std::io::Error> {
        let mut reader = BufReader::new(data);

        let file_type = Self::get_file_type(&mut reader)?;
        let version = Self::get_version(&mut reader, &file_type)?;
        let data_offset = Self::get_data_offset(&mut reader, &version)?;
        let load_address = Self::get_load_address(&mut reader, &file_type)?;
        let init_address = Self::get_init_address(&mut reader, &file_type)?;
        let play_address = Self::get_play_address(&mut reader, &file_type)?;
        let songs = Self::get_songs(&mut reader)?;
        let start_song = Self::get_start_song(&mut reader, songs)?;
        let speed = Self::get_speed(&mut reader, &file_type)?;
        let name = Self::get_name(&mut reader)?;
        let author = Self::get_author(&mut reader)?;
        let released = Self::get_released(&mut reader)?;

        match version {
            Version::V1 => {
                let real_load_address = Self::get_real_load_address(&mut reader)?;
                let data = Self::get_data(&mut reader)?;

                Ok(SidFile {
                    file_type,
                    version,
                    data_offset,
                    load_address,
                    init_address,
                    play_address,
                    songs,
                    start_song,
                    speed,
                    name,
                    author,
                    released,
                    real_load_address,
                    data,
                    flags: None,
                    start_page: None,
                    page_length: None,
                    second_sid_address: None,
                    third_sid_address: None,
                })
            }
            _ => {
                // fill additional header
                let flags = Self::get_flags(&mut reader)?;
                let start_page = Self::get_start_page(&mut reader)?;
                let page_length = Self::get_page_length(&mut reader)?;
                let second_sid_address = Self::get_sid_address(&mut reader)?;
                let third_sid_address = Self::get_sid_address(&mut reader)?;
                
                let real_load_address = Self::get_real_load_address(&mut reader)?;
                let data = Self::get_data(&mut reader)?;
                
                Ok(Self {
                    file_type,
                    version,
                    data_offset,
                    load_address,
                    init_address,
                    play_address,
                    songs,
                    start_song,
                    speed,
                    name,
                    author,
                    released,
                    real_load_address,
                    flags: Some(flags),
                    start_page: Some(start_page),
                    page_length: Some(page_length),
                    second_sid_address: Some(second_sid_address),
                    third_sid_address: Some(third_sid_address),
                    data,
                })
            }
        }
    }

    fn get_file_type(reader: &mut BufReader<&[u8]>) -> Result<Type, std::io::Error> {
        let file_type = reader.read_u32::<BigEndian>()?;
        match file_type {
            0x52534944 => Ok(Type::RSID),
            0x50534944 => Ok(Type::PSID),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid file_type",
            )),
        }
    }

    fn get_version(
        reader: &mut BufReader<&[u8]>,
        header: &Type,
    ) -> Result<Version, std::io::Error> {
        let version = reader.read_u16::<BigEndian>()?;
        match (header, version) {
            (_, 0x01) => Ok(Version::V1),
            (Type::PSID | Type::RSID, 0x02) => Ok(Version::V2),
            (Type::PSID | Type::RSID, 0x03) => Ok(Version::V3),
            (Type::PSID | Type::RSID, 0x04) => Ok(Version::V4),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid version",
            )),
        }
    }

    fn get_data_offset(
        reader: &mut BufReader<&[u8]>,
        version: &Version,
    ) -> Result<u16, std::io::Error> {
        let data_offset = reader.read_u16::<BigEndian>()?;

        match (version, data_offset) {
            (Version::V1, 0x76) => Ok(0x76),
            (Version::V2, 0x7C) => Ok(0x7C),
            (Version::V3, 0x7C) => Ok(0x7C),
            (Version::V4, 0x7C) => Ok(0x7C),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid data offset",
            )),
        }
    }

    fn get_load_address(
        reader: &mut BufReader<&[u8]>,
        header: &Type,
    ) -> Result<u16, std::io::Error> {
        let load_address = reader.read_u16::<BigEndian>()?;

        match (header, load_address) {
            (Type::PSID, _) => Ok(load_address),
            (Type::RSID, 0xE807..=0xFFFF) => Ok(load_address),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid load address",
            )),
        }
    }

    // but why?
    fn get_real_load_address(
        reader: &mut BufReader<&[u8]>
    ) -> Result<u16, std::io::Error> {
        let load_address: u16 = reader.read_u16::<LittleEndian>()?;
        Ok(load_address)
    }


    fn get_play_address(
        reader: &mut BufReader<&[u8]>,
        header: &Type,
    ) -> Result<u16, std::io::Error> {
        let play_address = reader.read_u16::<BigEndian>()?;

        match (header, play_address) {
            (Type::RSID, 0x0000) => Ok(play_address),
            (Type::PSID, 0x0000..=0xFFFF) => Ok(play_address),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid play address",
            )),
        }
    }

    fn get_init_address(
        reader: &mut BufReader<&[u8]>,
        header: &Type,
    ) -> Result<u16, std::io::Error> {
        let init_address = reader.read_u16::<BigEndian>()?;

        match (header, init_address) {
            (Type::RSID, 0x07E8..=0x9FFF) => Ok(init_address),
            (Type::RSID, 0xC000..=0xCFFF) => Ok(init_address),
            (Type::PSID, 0x0000..=0xFFFF) => Ok(init_address),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid init address",
            )),
        }
    }

    fn get_songs(reader: &mut BufReader<&[u8]>) -> Result<u16, std::io::Error> {
        let songs = reader.read_u16::<BigEndian>()?;
        match songs {
            0x0001..=0x0100 => Ok(songs),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid number of songs",
            )),
        }
    }

    fn get_start_song(reader: &mut BufReader<&[u8]>, songs: u16) -> Result<u16, std::io::Error> {
        let start_song = reader.read_u16::<BigEndian>()?;
        if start_song <= songs {
            Ok(start_song)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid start song",
            ))
        }
    }

    fn get_speed(
        reader: &mut BufReader<&[u8]>,
        header: &Type,
    ) -> Result<u32, std::io::Error> {
        let speed = reader.read_u32::<BigEndian>()?;
        match (header, speed) {
            (Type::RSID, 0x00000000) => Ok(speed),
            (Type::PSID, _) => Ok(speed),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid speed",
            )),
        }
    }

    fn get_name(reader: &mut BufReader<&[u8]>) -> Result<String, std::io::Error> {
        let mut name = [0u8; 32];
        reader.read_exact(&mut name)?;
        let str = String::from_utf8_lossy(&name)
            .to_string()
            .trim_matches(char::from(0))
            .to_string();

        Ok(str)
    }

    fn get_author(reader: &mut BufReader<&[u8]>) -> Result<String, std::io::Error> {
        let mut author = [0u8; 32];
        reader.read_exact(&mut author)?;
        let str = String::from_utf8_lossy(&author)
            .to_string()
            .trim_matches(char::from(0))
            .to_string();

        Ok(str)
    }

    fn get_released(reader: &mut BufReader<&[u8]>) -> Result<String, std::io::Error> {
        let mut released = [0u8; 32];
        reader.read_exact(&mut released)?;
        let str = String::from_utf8_lossy(&released)
            .to_string()
            .trim_matches(char::from(0))
            .to_string();

        Ok(str)
    }

    fn get_flags(reader: &mut BufReader<&[u8]>) -> Result<Flags, std::io::Error> {
        let flags = reader.read_u16::<BigEndian>()?;
        let mut bits: Vec<bool> = vec![false];
        for n in 0..16 {
            bits.push(((flags >> n) & 1) == 1);
        }
        let format = match bits[0] {
            false => Format::InternalPlayer,
            true => Format::MusData,
        };

        let play_sid = match bits[1] {
            false => PlaySID::C64,
            true => PlaySID::PlaySID,
        };

        let clock = match (bits[2], bits[3]) {
            (false, false) => Clock::Unknown,
            (false, true) => Clock::PAL,
            (true, false) => Clock::NTSC,
            (true, true) => Clock::Both,
        };

        let sid_model = Self::get_sid_model(bits[4], bits[5]);
        let second_sid_model = Self::get_sid_model(bits[6], bits[7]);
        let third_sid_model = Self::get_sid_model(bits[8], bits[9]);

        if bits[10..16].iter().any(|&x| x) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "Invalid flags",
            ));
        };

        Ok(Flags {
            format,
            play_sid,
            clock,
            sid_model,
            second_sid_model,
            third_sid_model,
        })
    }

    fn get_sid_model(bit0: bool, bit1: bool) -> ChipModel {
        match (bit0, bit1) {
            (false, false) => ChipModel::Unknown,
            (false, true) => ChipModel::MOS6581,
            (true, false) => ChipModel::MOS8580,
            (true, true) => ChipModel::Both,
        }
    }

    fn get_start_page(reader: &mut BufReader<&[u8]>) -> Result<u8, std::io::Error> {
        let start_page = reader.read_u8()?;
        Ok(start_page)
    }

    fn get_page_length(reader: &mut BufReader<&[u8]>) -> Result<u8, std::io::Error> {
        let page_length = reader.read_u8()?;
        Ok(page_length)
    }

    fn get_sid_address(reader: &mut BufReader<&[u8]>) -> Result<u8, std::io::Error> {
        let sid_address = reader.read_u8()?;
        Ok(sid_address)
    }

    fn get_data(reader: &mut BufReader<&[u8]>) -> Result<Vec<u8>, std::io::Error> {
        let mut data = vec![];
        _ = reader.read_to_end(&mut data);
        Ok(data)
    }
}
