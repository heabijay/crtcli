use crate::pkg::xml_wrappers::*;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use regex::Regex;
use std::cmp::Ordering;
use std::io::Cursor;
use std::sync::LazyLock;
use thiserror::Error;

const TERRASOFT_CONFIGURATION_REFERENCE_BLOCK: &str =
    "Reference Include=\"Terrasoft.Configuration\"";

pub static PKG_CSPROJ_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Files{sep}[^{sep}]+\.csproj$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package csproj file path regex")
});

#[derive(Debug, Error)]
pub enum CsprojProcessingError {
    #[error("error reading csproj xml: {0}")]
    Reader(#[from] quick_xml::Error),

    #[error("failed to process content of package references block: {0}")]
    PackageReferencesBlockProcessing(#[from] XmlReadEventBlockError),
}

pub fn apply_sorting(content: &[u8]) -> Result<Vec<u8>, CsprojProcessingError> {
    let mut reader = Reader::from_reader(content);
    let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

    loop {
        match reader.read_event().map_err(CsprojProcessingError::Reader)? {
            Event::Start(e) if is_package_references_item_group_start(&e) => {
                process_blocks_sorting(&mut reader, &mut writer, e)?;
            }
            Event::Eof => break,
            e => writer.write_event(e).unwrap(),
        }
    }

    return Ok(writer.into_inner().into_inner());

    fn is_package_references_item_group_start(e: &BytesStart) -> bool {
        is_item_group_start(e)
            && e.try_get_attribute("Label")
                .is_ok_and(|x| x.is_some_and(|x| x.value.as_ref() == b"Package References"))
    }

    fn process_blocks_sorting(
        reader: &mut Reader<&[u8]>,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        start_tag: BytesStart,
    ) -> Result<(), XmlReadEventBlockError> {
        writer.write_event(Event::Start(start_tag)).unwrap();

        let (mut event_blocks, end_tag) = xml_read_event_blocks(reader)?;

        event_blocks.blocks.sort_by(|a, b| {
            let a = a.first().expect("event block cannot be empty");
            let b = b.first().expect("event block cannot be empty");

            if a.eq_ignore_ascii_case(TERRASOFT_CONFIGURATION_REFERENCE_BLOCK.as_ref()) {
                Ordering::Less
            } else if b.eq_ignore_ascii_case(TERRASOFT_CONFIGURATION_REFERENCE_BLOCK.as_ref()) {
                Ordering::Greater
            } else {
                a.cmp(b)
            }
        });

        if let Some(ref open_sep) = event_blocks.separator {
            writer.write_event(Event::Text(open_sep.clone())).unwrap();
        }

        xml_write_event_blocks(writer, event_blocks);

        writer.write_event(Event::End(end_tag)).unwrap();

        Ok(())
    }
}

fn is_item_group_start(e: &BytesStart) -> bool {
    e.name().as_ref() == b"ItemGroup"
}
