use std::process;
use std::io::Cursor;
use std::io::prelude::*;
use byteorder::{LittleEndian, ReadBytesExt};

/// https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/Chunk.java
#[allow(non_camel_case_types)]
enum ChunkType {
    NULL = 0x0000,
    STRING_POOL = 0x0001,
    TABLE = 0x0002,
    XML = 0x0003,
    XML_START_NAMESPACE = 0x0100,
    XML_END_NAMESPACE = 0x0101,
    XML_START_ELEMENT = 0x0102,
    XML_END_ELEMENT = 0x0103,
    XML_CDATA = 0x0104,
    XML_RESOURCE_MAP = 0x0180,
    TABLE_PACKAGE = 0x0200,
    TABLE_TYPE = 0x0201,
    TABLE_TYPE_SPEC = 0x0202,
    TABLE_LIBRARY = 0x0203
}

const SORTED_FLAG : i32 = 1 << 8;
const UTF8_FLAG: i32 = 1 << 8;

pub struct Context {
    pub mode: i32,
    pub strings_pool: Vec<String>
}

/// https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/XmlStartElementChunk.java
fn read_xml_start_element_chunk(context: &mut Context, cursor: &mut Cursor<&Vec<u8>>, chunk_offset: u64, header_size: u64, chunk_size: u64) {
    #[cfg(debug_assertions)]
    println!("xml start element chunk, offset: {}", cursor.position());
    let line_number = cursor.read_i32::<LittleEndian>().unwrap();
    let comment = cursor.read_i32::<LittleEndian>().unwrap();
    let namespace = cursor.read_i32::<LittleEndian>().unwrap();
    let name = cursor.read_i32::<LittleEndian>().unwrap();
    let tag_name = context.strings_pool.get(name as usize).unwrap();
    let attribute_start = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF;
    let attribute_size = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF;
    let attribute_count = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF;
    let id_index = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF - 1;
    let class_index = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF - 1;
    let style_index = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF - 1;
    #[cfg(debug_assertions)]
    println!("start element: line_number: {}, comment: {}, namespace: {}, name: {}, attribute_start: {}, attribute_size: {}, attribute_count: {}", 
        line_number, comment, namespace, tag_name, attribute_start, attribute_size, attribute_count);

    let offset = chunk_offset + header_size + attribute_start;
    cursor.set_position(offset);
    for i in 0..attribute_count {
        let namespace = cursor.read_i32::<LittleEndian>().unwrap();
        let name = cursor.read_i32::<LittleEndian>().unwrap();
        let raw_value = cursor.read_i32::<LittleEndian>().unwrap();

        let attr_name = context.strings_pool.get(name as usize).unwrap();

        // TODO: type_value
        // @see https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/XmlAttribute.java
        // @see https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/BinaryResourceValue.java
        let type_value_size = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF;
        let _ = cursor.read_i8().unwrap();
        let type_value_type = cursor.read_i8().unwrap();
        let type_value_data = cursor.read_i32::<LittleEndian>().unwrap();

        if type_value_type == 0x03 { // string
            let value = context.strings_pool.get(raw_value as usize).unwrap();
            #[cfg(debug_assertions)]
            println!("attribute: string {}({})={}", attr_name, name, value); 

            if context.mode == 1 && tag_name == "manifest" && attr_name == "package" {
                println!("{}", value);
                process::exit(-1);  // TODO: just print the package name for now
            }
        } else if type_value_type == 0x10 { // dec
            #[cfg(debug_assertions)]
            println!("attribute: dec {}({})={}", attr_name, name, type_value_data); 
        } else if type_value_type == 0x12 { // INT_BOOLEAN
            #[cfg(debug_assertions)]
            println!("attribute: bool {}({})={}", attr_name, name, type_value_data); 
            if context.mode == 2 && tag_name == "application" && attr_name == "debuggable" {
                println!("{}", if type_value_data != 0 { "true" } else { "false" } ); // 0=false, -1=true
                process::exit(0);  // TODO: just print the debuggable for now
            }
        } else {
            #[cfg(debug_assertions)]
            println!("attribute: {} {}({})={}", type_value_type, attr_name, name, type_value_data); 
        }
        //#[cfg(debug_assertions)]
        //println!("type_value_size: {}, type_value_type: {}, type_value_data: {}, raw_value: {}", type_value_size, type_value_type, type_value_data, raw_value); 
    }

    if context.mode == 2 && tag_name == "application" {
        println!("{}", "false"); // not found debuggable attr in application
        process::exit(-1);  // TODO: just print the debuggable for now
    }

    cursor.set_position(chunk_offset + chunk_size as u64); 
}

/// https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/StringPoolChunk.java
fn read_string_pool_chunk(context: &mut Context, cursor: &mut Cursor<&Vec<u8>>, chunk_offset: u64, header_size: u64, chunk_size: u64) {
    #[cfg(debug_assertions)]
    println!("string pool chunk");
    let string_count = cursor.read_i32::<LittleEndian>().unwrap();
    let style_count = cursor.read_i32::<LittleEndian>().unwrap();
    let flags = cursor.read_i32::<LittleEndian>().unwrap();
    let strings_start = cursor.read_i32::<LittleEndian>().unwrap() as u64;
    let styles_start = cursor.read_i32::<LittleEndian>().unwrap() as u64;

    let is_utf8 = flags & UTF8_FLAG != 0;
    let is_sorted = flags & SORTED_FLAG != 0;
    #[cfg(debug_assertions)]
    println!("string_count: {}, style_count: {}, flags: {}, strings_start: {}, styles_start: {}, is_utf8: {}, is_sorted: {}",
        string_count, style_count, flags, strings_start, styles_start, is_utf8, is_sorted);

    let mut strings_offset_list = Vec::new();
    for i in 0..string_count {
        let string_offset = cursor.read_i32::<LittleEndian>().unwrap() as u64;
        //println!("string index: {}, string offset: {}", i, string_offset);
        strings_offset_list.push(chunk_offset + strings_start + string_offset)
    }

    for string_offset in strings_offset_list {
        cursor.set_position(string_offset);
        //println!("string offset: {}", string_offset);
        /*
        let mut buffer = String::new();
        cursor.read_to_string(&mut buffer).unwrap();
        println!("string: {}", buffer);
        */

        let text;
        if is_utf8 {
            // UTF-8 strings have 2 lengths: the number of characters, and then the encoding length.
            let mut length = cursor.read_i32::<LittleEndian>().unwrap();
            if (length & 0x8000) != 0 {
                let length2 = cursor.read_i32::<LittleEndian>().unwrap() & 0xFFFF;
                length = ((length & 0x7FFF) << 8) | length2;
            }
            //println!("characterCount: {}", length);
            length = cursor.read_i32::<LittleEndian>().unwrap();
            if (length & 0x8000) != 0 {
                let length2 = cursor.read_i32::<LittleEndian>().unwrap() & 0xFFFF;
                length = ((length & 0x7FFF) << 8) | length2;
            }
            //println!("characterCount: {}", length);
            let mut buf: Vec<u8> = Vec::with_capacity(length as usize);
            cursor.read_exact(&mut buf).unwrap();
            text = String::from_utf8(buf).unwrap();
        } else {
            // UTF-16 strings, however, only have 1 length: the number of characters.
            let mut length = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF;
            if (length & 0x8000) != 0 {
                let length2 = cursor.read_i16::<LittleEndian>().unwrap() as u64 & 0xFFFF;
                length = ((length & 0x7FFF) << 16) | length2;
            }
            //println!("characterCount: {}", length);
            let mut buf: Vec<u16> = vec![0; length as usize];
            cursor.read_u16_into::<LittleEndian>(&mut buf).unwrap();
            //println!("strbuf: {}", buf.hex_dump());
            text = String::from_utf16(&buf).unwrap();
        }
        //println!("string: {}", text);
        context.strings_pool.push(text);
    }

    cursor.set_position(chunk_offset + chunk_size as u64); 
}

/// https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/Chunk.java
pub fn read_chunk(context: &mut Context, cursor: &mut Cursor<&Vec<u8>>) {
    let chunk_offset = cursor.position();
    let chunk_type = cursor.read_i16::<LittleEndian>().unwrap();
    let header_size = cursor.read_i16::<LittleEndian>().unwrap();
    let chunk_size = cursor.read_i32::<LittleEndian>().unwrap();
    #[cfg(debug_assertions)]
    println!("chunk_offset: {}, chunk_type: {}, header_size: {}, chunk_size: {}", chunk_offset, chunk_type, header_size, chunk_size);

    if chunk_type == ChunkType::XML as i16 {
        // https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/XmlChunk.java
        // https://cs.android.com/android-studio/platform/tools/base/+/mirror-goog-studio-main:apkparser/binary-resources/src/main/java/com/google/devrel/gmscore/tools/apk/arsc/ChunkWithChunks.java
        #[cfg(debug_assertions)]
        println!("read xml chunk");
        while cursor.position() < chunk_offset + chunk_size as u64 {
            read_chunk(context, cursor);
        }
    } else if chunk_type == ChunkType::STRING_POOL as i16 {
        read_string_pool_chunk(context, cursor, chunk_offset, header_size as u64, chunk_size as u64);
    } else if chunk_type == ChunkType::XML_START_ELEMENT as i16 {
        read_xml_start_element_chunk(context, cursor, chunk_offset, header_size as u64, chunk_size as u64);
    } else {
        cursor.set_position(chunk_offset + chunk_size as u64); // skip unknown chunk type
    }
}
