//! Handle communications where the listener is expected
//! to the send the same data as the ones received from the
//! dialer.
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{ReadHalf, WriteHalf};

use super::Protocol;
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
    protocol: Protocol,
) -> Result<bool, PadawanError> {
    let (mut send, mut recv) = (false, false);
    let encoded = protocol.encode();
    let mut response = vec![0_u8; encoded.len()];
    loop {
        tokio::select!(
            res = read.read_exact(&mut response), if !recv  => {
                let n_read = res?;
                tracing::trace!("multistream read {:?}", response.as_slice());
                recv = n_read > 0;
            },
            res = write.write(&encoded), if !send => {
                res?;
                tracing::trace!("multistream wrote {:?} ", &encoded[..]);
                send = true;
            },
            else => break
        );
    }
    Ok(response == encoded.as_slice())
}

/// First write to the stream and then get the peer response.
///
/// Returns `true` if the messages match.
pub async fn dial<'a>(
    read: &mut ReadHalf<'a>,
    write: &mut WriteHalf<'a>,
    protocol: Protocol,
) -> Result<bool, PadawanError> {
    let encoded = protocol.encode();
    let mut response = vec![0_u8; encoded.len()];
    write.write_all(&encoded).await?;
    loop {
        if read.read_exact(&mut response).await? > 0 {
            break;
        }
    }
    Ok(encoded.as_slice() == response)
}

/// First read from the stream and then send the matching response
/// to the remote peer.
///
/// # Errors
///
/// Fails if the incoming payload has invalid encoding.
pub async fn listen<'a>(
    read: &mut ReadHalf<'a>,
    write: &mut WriteHalf<'a>,
    protocol: Protocol,
) -> Result<(), PadawanError> {
    let encoded = protocol.encode();
    let mut incoming = vec![0_u8; encoded.len()];
    loop {
        if read.read_exact(&mut incoming).await? > 0 {
            break;
        }
    }
    let response = Protocol::decode(&incoming)?.encode();
    Ok(write.write_all(&response).await?)
}

/// Like [`dial`][] but for use during noise transport.
pub async fn dial_noise<'a>(
    read: &mut ReadHalf<'a>,
    write: &mut WriteHalf<'a>,
    protocol: Protocol,
    transport: &mut noise::Transport,
) -> Result<bool, PadawanError> {
    let encoded = protocol.encode();
    // Send
    let buffer = transport.buffer().write();
    let n = encoded.as_slice().read(buffer).await?;
    buffer.truncate(n);
    let encrypted = transport.encrypt()?;
    noise::wire::send(write, encrypted).await?;

    // Receive
    let buffer = transport.buffer().encrypted();
    noise::wire::recv(read, buffer).await?;
    let decrypted = transport.decrypt()?;
    Ok(encoded.as_slice() == decrypted)
}
