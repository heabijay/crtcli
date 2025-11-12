use crate::pkg::transforms::SortingComparer;
use crate::pkg::xml::*;
use crate::utils::bom;
use quick_xml::events::{BytesStart, Event};
use quick_xml::{Reader, Writer};
use regex::Regex;
use std::cmp::Ordering;
use std::io::Cursor;
use std::sync::LazyLock;
use thiserror::Error;

pub static PKG_RESOURCE_PATH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(&format!(
        r"^Resources{sep}.+?{sep}resource\.(?<culture>.+?)\.xml$",
        sep = regex::escape(std::path::MAIN_SEPARATOR_STR)
    ))
    .expect("failed to compile regex for package resource path regex")
});

#[derive(Debug, Error)]
pub enum ResourceProcessingError {
    #[error("error reading resource xml: {0}")]
    Reader(#[from] quick_xml::Error),

    #[error("failed to process content of items block: {0}")]
    ItemsBlockProcessing(#[from] XmlReadEventBlockError),
}

pub fn apply_sorting(
    content: &[u8],
    comparer: SortingComparer,
) -> Result<Vec<u8>, ResourceProcessingError> {
    let mut writer = Writer::new(Cursor::new(Vec::<u8>::new()));

    if bom::is_bom(content) {
        writer.write_bom().unwrap();
    }

    let mut reader = Reader::from_reader(content);

    loop {
        match reader
            .read_event()
            .map_err(ResourceProcessingError::Reader)?
        {
            Event::Start(e) if is_items_group_start(&e) => {
                process_blocks_sorting(comparer, &mut reader, &mut writer, e)?;
            }
            Event::Eof => break,
            e => writer.write_event(e).unwrap(),
        }
    }

    return Ok(writer.into_inner().into_inner());

    fn process_blocks_sorting(
        comparer: SortingComparer,
        reader: &mut Reader<&[u8]>,
        writer: &mut Writer<Cursor<Vec<u8>>>,
        start_tag: BytesStart,
    ) -> Result<(), ResourceProcessingError> {
        writer.write_event(Event::Start(start_tag)).unwrap();

        let (mut event_blocks, end_tag) = xml_read_event_blocks(reader)?;

        event_blocks.blocks.sort_by(|a, b| {
            let a = a.first().expect("event block cannot be empty");
            let b = b.first().expect("event block cannot be empty");

            match (a, b) {
                (Event::Empty(a), Event::Empty(b)) => {
                    match (a.try_get_attribute("Name"), b.try_get_attribute("Name")) {
                        (Ok(Some(a_name)), Ok(Some(b_name))) => comparer.cmp(
                            &a_name.value.to_ascii_lowercase(),
                            &b_name.value.to_ascii_lowercase(),
                        ),
                        (Ok(Some(_)), _) => Ordering::Greater,
                        (_, Ok(Some(_))) => Ordering::Less,
                        (Ok(_), _) => Ordering::Greater,
                        (_, Ok(_)) => Ordering::Less,
                        _ => Ordering::Equal,
                    }
                }
                (Event::Empty(_), _) => Ordering::Greater,
                (_, Event::Empty(_)) => Ordering::Less,
                _ => Ordering::Equal,
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

fn is_items_group_start(e: &BytesStart) -> bool {
    e.name().as_ref() == b"Items"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sorting() {
        let input = tests_data::resource_tests_data::INPUT.as_bytes();
        let expected_output =
            tests_data::resource_tests_data::RESOURCE_SORTING_EXPECTED_OUTPUT.as_bytes();

        let output = apply_sorting(input, SortingComparer::Alnum).unwrap();

        pretty_assertions::assert_eq!(
            String::from_utf8_lossy(expected_output),
            String::from_utf8_lossy(&output)
        );
    }
}
