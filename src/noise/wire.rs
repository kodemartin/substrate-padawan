//! Implement the noise-frame specification for read
//! and write operations
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp;

use crate::error::PadawanError;

const MAX_FRAME_SIZE: usize = 65536;
const MAX_PAYLOAD_SIZE: usize = MAX_FRAME_SIZE - super::ENCRYPTION_INFLATION_SIZE;

/// Read a noise frame from the remote peer and put the payload
/// into the given `buffer`
pub async fn recv<'a>(
    read: &mut tcp::ReadHalf<'a>,
    buffer: &mut Vec<u8>,
) -> Result<(), PadawanError> {
    let n = read.read_u16().await?;
    if n as usize > MAX_FRAME_SIZE {
        return Err(PadawanError::NoiseFrameSizeExceeded);
    }
    buffer.resize(n as usize, 0);
    read.read_exact(buffer).await?;
    Ok(())
}

/// Create a noise frame from the given `payload` and send it to the remote peer
pub async fn send<'a>(
    write: &mut tcp::WriteHalf<'a>,
    payload: &[u8],
) -> Result<usize, PadawanError> {
    if payload.len() > MAX_PAYLOAD_SIZE {
        return Err(PadawanError::NoiseFrameSizeExceeded);
    }
    write.write_u16(payload.len() as u16).await?;
    Ok(write.write(payload).await?)
}
