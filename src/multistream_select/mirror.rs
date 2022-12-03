//! Handle communications where the listener is expected
//! to the send the same data as the ones received from the
//! dialer.
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};

use super::EncodedProtocol;
use crate::error::PadawanError;
use crate::noise;

/// Read and write concurrently to the stream.
///
/// Useful in cases where the peers may send data immediately after connection.
/// For example, when sending the `/multistream/1.0.0` headers.
///
/// Returns `true` if the protocols match.
pub async fn concurrent<'a>(
    read: &mut ReadHalf<'a>,
    write: &mut WriteHalf<'a>,
    bytes: &[u8],
) -> bool {
    let (mut send, mut recv) = (false, false);
    let mut response = vec![0_u8; bytes.len()];
    loop {
        tokio::select!(
            Ok(n_read) = read.read_exact(&mut response), if !recv  => {
                tracing::trace!("read {:?}", response.as_slice());
                recv = n_read > 0;
            },
            Ok(_) = write.write(bytes), if !send => {
                tracing::trace!("wrote {:?} ", bytes);
                send = true;
            },
            else => break
        );
    }
    response == bytes
}

/// First write to the stream and then get the peer response.
///
/// Returns `true` if the messages match.
pub async fn dial<'a>(
    read: &mut ReadHalf<'a>,
    write: &mut WriteHalf<'a>,
    bytes: &[u8],
) -> bool {
    let mut response = vec![0_u8; bytes.len()];
    if write.write_all(bytes).await.is_err() {
        return false;
    }
    loop {
        if let Ok(n) = read.read_exact(&mut response).await {
            if n > 0 {
                tracing::trace!("read {:?} bytes", n);
                break;
            }
        }
    }
    response == bytes
}

/// Like [`dial`][] but for use during noise transport.
pub async fn dial_noise<'a>(
    read: &mut ReadHalf<'a>,
    write: &mut WriteHalf<'a>,
    mut protocol: EncodedProtocol,
    transport: &mut noise::Transport,
) -> Result<bool, PadawanError> {
    // Send
    let buffer = transport.buffer().write();
    let n = protocol.as_bytes_mut().read(buffer).await?;
    buffer.truncate(n);
    let encrypted = transport.encrypt()?;
    noise::wire::send(write, encrypted).await?;

    // Receive
    let buffer = transport.buffer().encrypted();
    noise::wire::recv(read, buffer).await?;
    let decrypted = transport.decrypt()?;
    Ok(protocol.as_bytes() == decrypted)
}
