use pin_project::pin_project;
use tokio::io::AsyncWrite;
use tokio::process::ChildStdin;

#[pin_project(project = TxProj)]
pub enum TxWrapper {
    Child(#[pin] ChildStdin),
}

impl futures::io::AsyncWrite for TxWrapper {
    fn poll_write(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        match self.project() {
            TxProj::Child(inner) => inner.poll_write(cx, buf),
        }
    }

    fn poll_flush(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.project() {
            TxProj::Child(inner) => inner.poll_flush(cx),
        }
    }

    fn poll_close(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        match self.project() {
            TxProj::Child(inner) => inner.poll_shutdown(cx),
        }
    }
}

pub trait WrapTx {
    fn wrap_tx(self) -> TxWrapper;
}

impl WrapTx for ChildStdin {
    fn wrap_tx(self) -> TxWrapper {
        TxWrapper::Child(self)
    }
}
