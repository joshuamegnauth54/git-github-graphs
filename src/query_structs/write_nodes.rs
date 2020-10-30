use crate::error::Result;
use serde::Serialize;
use serde_json::to_writer_pretty;
use std::io::Write;

// Okay...so there's no point for this function anymore since Serde is amazing and deserializes the
// Vector properly, I think.
pub fn write_nodes<I, W>(writer: W, nodes: &[I]) -> Result<()>
where
    I: Serialize,
    W: Write,
{
    Ok(to_writer_pretty(writer, nodes)?)
}
