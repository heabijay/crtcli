use quick_xml::events::{BytesEnd, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;
use thiserror::Error;

pub mod csproj;

pub mod resource;

#[cfg(test)]
mod tests_data;

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

fn xml_skip_event_blocks(reader: &mut Reader<&[u8]>) -> Result<(), XmlReadEventBlockError> {
    let mut depth = 0;

    loop {
        match reader.read_event()? {
            Event::Start(_) => depth += 1,
            Event::End(_) => {
                depth -= 1;

                if depth < 0 {
                    return Ok(());
                }
            }
            Event::Eof => return Err(XmlReadEventBlockError::UnexpectedEof),
            _ => {}
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

        if i < event_blocks_len - 1
            && let Some(ref separator) = event_blocks.separator
        {
            writer.write_event(Event::Text(separator.clone())).unwrap();
        }
    }

    if let Some(close_sep) = event_blocks.final_separator {
        writer.write_event(Event::Text(close_sep)).unwrap();
    }
}
