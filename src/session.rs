use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
/// As a part of my refactor, I'm making the server treat sessions with a simple
/// state machine. Im also not integrating this until I can :)
/// This connects already written logic for specific protocols
/// Also, this directs the clients as well so i dont have to jump between 3 different
/// submodules to write a new client server interaction
use tokio::net::TcpStream;
use tokio::time::timeout;

pub enum SessionState {
    Handshake { complete: bool },
    Ready,
    Transfer,
    Naming,
    Dead,
}

#[repr(u32)]
#[derive(Copy, Clone)]
pub enum ClientSignal {
    MoveTransfer = 1,
    MoveNaming = 2,
}

#[repr(u32)]
pub enum ServerResponse {
    Success = 1,
    Failure = 2,
}

pub struct Session {
    stream: TcpStream,
    state: SessionState,
    timeout: Duration,
}

impl Session {
    pub fn new(stream: TcpStream, timeout: Duration) -> Self {
        Self {
            stream,
            state: SessionState::Handshake { complete: false },
            timeout,
        }
    }

    pub async fn send_signal(&mut self, signal: ClientSignal) -> anyhow::Result<()> {
        self.stream.write_u32(signal as u32).await?;

        let response: ServerResponse = timeout(self.timeout, self.stream.read_u32())
            .await
            .map_err(|_| { self.state = SessionState::Dead; anyhow::Error::msg("Server timed out, session moved to dead") })?
            .map_err(|e| anyhow::Error::new(e))
            .map(|v| {
                match v {
                    1 => ServerResponse::Success,
                    2 => ServerResponse::Failure,
                    e => panic!("Client recieved a server response in session.rs, send_signal that is not properly handled: {}", e),
                }
            })?;

        if let ServerResponse::Failure = response {
            return Err(anyhow::Error::msg("Could not transition state"));
        }

        match signal {
            ClientSignal::MoveNaming => self.state = SessionState::Naming,
            ClientSignal::MoveTransfer => self.state = SessionState::Transfer,
        }

        Ok(())
    }

    async fn read_signal(&mut self) -> anyhow::Result<()> {
        let signal: ClientSignal = timeout(self.timeout, self.stream.read_u32())
            .await
            .map_err(|_| {
                self.state = SessionState::Dead;
                anyhow::Error::msg("Client timed out, killing stream and moving to dead")
            })?
            .map_err(|e| anyhow::Error::new(e))
            .map(|v| match v {
                1 => ClientSignal::MoveTransfer,
                2 => ClientSignal::MoveNaming,
                v => panic!(
                    "Client sent a signal which is not properly handled in read_signal: {}",
                    v
                ),
            })?;

        let response: ServerResponse = match (signal, &self.state) {
            (ClientSignal::MoveTransfer, SessionState::Ready) => {
                self.state = SessionState::Transfer;
                ServerResponse::Success
            }
            (ClientSignal::MoveNaming, SessionState::Ready) => {
                self.state = SessionState::Naming;
                ServerResponse::Success
            }
            (_, _) => ServerResponse::Failure,
        };

        self.stream.write_u32(response as u32).await?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::net::TcpListener;
    use tokio::time::sleep;

    #[tokio::test]
    async fn state_changes() -> anyhow::Result<()> {
        let listener = TcpListener::bind("127.0.0.1:0").await?;
        let rel = listener.local_addr()?;

        tokio::task::spawn(async move {
            let (mut stream, _) = listener.accept().await?;

            Ok::<(), anyhow::Error>(())
        });

        Ok(())
    }
}
