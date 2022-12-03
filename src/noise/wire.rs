//! Implement the noise-frame specification for read
//! and write operations
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp;

use crate::error::PadawanError;

/// Read a noise frame from the remote peer and put the payload
/// into the given `buffer`
pub async fn recv<'a>(
    read: &mut tcp::ReadHalf<'a>,
    buffer: &mut Vec<u8>,
) -> Result<(), PadawanError> {
    let n = read.read_u16().await?;
    buffer.resize(n as usize, 0);
    read.read_exact(buffer).await?;
    Ok(())
}

/// Create a noise frame from the given `payload` and send it to the remote
/// peer
pub async fn send<'a>(
    write: &mut tcp::WriteHalf<'a>,
    payload: &[u8],
) -> Result<usize, PadawanError> {
    write.write_u16(payload.len() as u16).await?;
    Ok(write.write(payload).await?)
}
