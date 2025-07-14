use quick_xml::events::{BytesEnd, BytesStart, BytesText, Event};
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

    #[error("failed to process content of block: {0}")]
    PackageReferenceBlockProcessing(#[from] CsprojBlocksProcessingError),
}

#[derive(Debug, Error)]
pub enum CsprojBlocksProcessingError {
    #[error("error reading csproj xml: {0}")]
    Reader(#[from] quick_xml::Error),

    #[error("failed to read xml block (tag): {0}")]
    ReadingBlock(#[from] XmlReadEventBlockError),
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
    ) -> Result<(), CsprojBlocksProcessingError> {
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

#[derive(Debug)]
struct EventBlocks<'a> {
    separator: Option<BytesText<'a>>,
    final_separator: Option<BytesText<'a>>,
    blocks: Vec<Vec<Event<'a>>>,
}

fn xml_read_event_blocks<'a>(
    reader: &mut Reader<&'a [u8]>,
) -> Result<(EventBlocks<'a>, BytesEnd<'a>), XmlReadEventBlockError> {
    let mut separator: Option<BytesText> = None;
    let mut final_separator: Option<BytesText> = None;
    let mut event_blocks: Vec<Vec<Event>> = vec![];

    loop {
        match reader.read_event()? {
            Event::Text(t) => {
                if separator.is_none() {
                    separator = Some(t)
                } else {
                    final_separator = Some(t)
                }
            }
            Event::Eof => return Err(XmlReadEventBlockError::UnexpectedEof),
            Event::End(e) => {
                let event_blocks = EventBlocks {
                    separator,
                    final_separator,
                    blocks: event_blocks,
                };

                return Ok((event_blocks, e));
            }
            e => event_blocks.push(xml_read_event_block(reader, e)?),
        }
    }
}

#[derive(Debug, Error)]
pub enum XmlReadEventBlockError {
    #[error("unexpected end of file while reading block (not closed tag?)")]
    UnexpectedEof,

    #[error("error while reading block: {0}")]
    Reader(#[from] quick_xml::Error),
}

fn xml_read_event_block<'a>(
    reader: &mut Reader<&'a [u8]>,
    current_event: Event<'a>,
) -> Result<Vec<Event<'a>>, XmlReadEventBlockError> {
    match current_event {
        Event::Start(e) => {
            let mut event_block = vec![Event::Start(e)];
            let mut depth = 0;

            loop {
                match reader.read_event()? {
                    Event::Start(e) => {
                        depth += 1;
                        event_block.push(Event::Start(e));
                    }
                    Event::End(e) => {
                        depth -= 1;

                        event_block.push(Event::End(e));

                        if depth < 0 {
                            return Ok(event_block);
                        }
                    }
                    Event::Eof => return Err(XmlReadEventBlockError::UnexpectedEof),
                    e => event_block.push(e),
                }
            }
        }
        e => Ok(vec![e]),
    }
}

fn xml_write_event_blocks(writer: &mut Writer<Cursor<Vec<u8>>>, event_blocks: EventBlocks) {
    let event_blocks_len = event_blocks.blocks.len();

    for (i, item) in event_blocks.blocks.into_iter().enumerate() {
        for event in item {
            writer.write_event(event).unwrap();
        }

        if i < event_blocks_len - 1 {
            if let Some(ref separator) = event_blocks.separator {
                writer.write_event(Event::Text(separator.clone())).unwrap();
            }
        }
    }

    if let Some(close_sep) = event_blocks.final_separator {
        writer.write_event(Event::Text(close_sep)).unwrap();
    }
}

fn is_item_group_start(e: &BytesStart) -> bool {
    e.name().as_ref() == b"ItemGroup"
}
