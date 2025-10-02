use crate::CommandHandledError;
use crate::app::{CrtClient, CrtClientError};
use crate::cmd::app::AppCommand;
use crate::cmd::cli::{CommandDynError, CommandResult};
use crate::cmd::utils::{humanize_bytes, humanize_duration_time_precise};
use anstyle::{AnsiColor, Color, Style};
use async_trait::async_trait;
use clap::Args;
use clap::builder::{ValueParser, ValueParserFactory};
use crossterm::event::EventStream;
use crossterm::{QueueableCommand, cursor, event, style, terminal};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use std::io::Write;
use std::net::SocketAddr;
use std::process::ExitCode;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;
use thiserror::Error;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::RwLock;
use tokio::task::JoinHandle;
use tokio::time::Instant;
use tokio_tungstenite::WebSocketStream;
use tokio_tungstenite::tungstenite::Message;
use tokio_util::bytes::Bytes;
use tokio_util::sync::CancellationToken;

#[derive(Args, Debug)]
pub struct TunnelCommand {
    /// Print defined connection strings in the Creatio configuration
    #[arg(long)]
    connection_strings: bool,

    /// Local port forwarding rule(s) in the format [bind_address:]port:host:host_port
    #[arg(short = 'L', value_name = "[bind_address:]port:host:host_port")]
    local_forward: Vec<ForwardMappingArg>,
}

#[derive(Error, Debug)]
enum TunnelCommandError {
    #[error("binding tcp listener to {0} failed: {1}")]
    TcpListenerBind(SocketAddr, #[source] std::io::Error),

    #[error("sorry, seems like your terminal is not supported: {0}")]
    TerminalNotSupported(#[source] std::io::Error),

    #[error("sorry, failed to validate your Creatio permissions: {0}")]
    TunnelingPermission(String),
}

#[derive(Debug, Clone)]
struct ForwardMappingArg {
    bind_address: SocketAddr,
    host: String,
    host_port: u16,
}

#[derive(Error, Debug, PartialEq)]
pub enum ForwardMappingArgParsingError {
    #[error("invalid format")]
    Format,

    #[error("invalid IP address: '{0}'")]
    IpAddr(String),

    #[error("invalid port number: '{0}'")]
    Port(String),
}

impl TryFrom<&str> for ForwardMappingArg {
    type Error = ForwardMappingArgParsingError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let parts: Vec<&str> = value.split(':').collect();

        let (bind_address, port_str, host, host_port_str) = match parts.len() {
            3 => ("127.0.0.1", parts[0], parts[1], parts[2]),
            4 => {
                let bind_address = if parts[0].is_empty() {
                    "0.0.0.0"
                } else {
                    parts[0]
                };

                (bind_address, parts[1], parts[2], parts[3])
            }
            _ => return Err(ForwardMappingArgParsingError::Format),
        };

        let port = port_str
            .parse::<u16>()
            .map_err(|_| ForwardMappingArgParsingError::Port(port_str.to_string()))?;

        let host_port = host_port_str
            .parse::<u16>()
            .map_err(|_| ForwardMappingArgParsingError::Port(host_port_str.to_string()))?;

        let bind_address = SocketAddr::new(
            bind_address
                .parse()
                .map_err(|_| ForwardMappingArgParsingError::IpAddr(bind_address.to_string()))?,
            port,
        );

        Ok(ForwardMappingArg {
            bind_address,
            host: host.to_string(),
            host_port,
        })
    }
}

impl ValueParserFactory for ForwardMappingArg {
    type Parser = ValueParser;

    fn value_parser() -> Self::Parser {
        ValueParser::new(|s: &str| ForwardMappingArg::try_from(s).map_err(|e| e.to_string()))
    }
}

#[async_trait]
impl AppCommand for TunnelCommand {
    async fn run(&self, client: Arc<CrtClient>) -> CommandResult {
        ensure_has_creatio_permissions(&client).await?;

        if self.connection_strings {
            return print_connection_strings(&client).await;
        }

        if self.local_forward.is_empty() {
            print_forward_mappings_not_specified();
            return Err(CommandHandledError(ExitCode::FAILURE).into());
        }

        let mut listener_contexts = vec![];
        let mut join_handles = vec![];

        for forward_mapping in &self.local_forward {
            let (listener_context, join_handle) =
                start_tcp_listener_forwarding(client.clone(), forward_mapping).await?;

            listener_contexts.push(listener_context);
            join_handles.push(join_handle);
        }

        let start_time = Instant::now();

        let tunneling_dashboard_result =
            show_tunneling_dashboard(Arc::clone(&client), listener_contexts)
                .await
                .map_err(TunnelCommandError::TerminalNotSupported);

        hide_tunneling_dashboard();

        tunneling_dashboard_result?;

        eprintln!(
            "Tunneling session via {url} stopped — {time}",
            url = client.base_url(),
            time = humanize_duration_time_precise(start_time.elapsed()),
        );

        return Ok(());

        async fn ensure_has_creatio_permissions(client: &CrtClient) -> CommandResult {
            let response = client.crtcli_tunneling_service().get_status().await?;

            if response.allowed {
                Ok(())
            } else {
                Err(
                    TunnelCommandError::TunnelingPermission(
                        response
                            .error
                            .expect("CheckPermissionsResponse.Error is null, but CheckPermissionsResponse.Result is true")
                            .message
                    )
                    .into()
                )
            }
        }

        async fn print_connection_strings(client: &CrtClient) -> CommandResult {
            let connection_strings = client
                .crtcli_tunneling_service()
                .get_connection_strings()
                .await?;

            eprintln!(
                "{bold}Listing connection strings at {base_url}:{bold:#}",
                base_url = client.base_url(),
                bold = Style::new().bold()
            );

            for (i, (key, value)) in connection_strings.iter().enumerate() {
                println!(
                    "{underline}{key}{underline:#}",
                    underline = Style::new().underline()
                );
                println!("{italic}{value}{italic:#}", italic = Style::new().italic());

                if i < connection_strings.len() - 1 {
                    println!();
                }
            }

            Ok(())
        }

        fn print_forward_mappings_not_specified() {
            let bold = Style::new().bold();
            let bold_underline = Style::new().bold().underline();

            eprintln!(
                "{red_bold}error:{red_bold:#} the following required arguments were not provided:",
                red_bold = Style::new()
                    .fg_color(Some(Color::Ansi(AnsiColor::Red)))
                    .bold(),
            );

            eprintln!(
                "  {green}-L <[bind_address:]port:host:host_port>{green:#}",
                green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
            );

            eprintln!();
            eprintln!(
                "{bold_underline}Usage:{bold_underline:#} {bold}crtcli app <URL/APP> tunnel -L{bold:#} <[bind_address:]port:host:host_port>"
            );
            eprintln!();
            eprintln!("For more information, try '{bold}--help{bold:#}'.");
        }

        async fn show_tunneling_dashboard(
            client: Arc<CrtClient>,
            listener_contexts: Vec<Arc<ListenerContext>>,
        ) -> Result<(), std::io::Error> {
            terminal::enable_raw_mode()?;
            anstream::stderr().queue(terminal::EnterAlternateScreen)?;
            anstream::stderr().queue(cursor::Hide)?;

            let start_time = Instant::now();
            let refresh_interval = Duration::from_millis(500);

            let cancellation_token = start_polling_cancellation_token();
            let mut stderr = anstream::stderr();

            loop {
                let elapsed_str = humanize_duration_time_precise(start_time.elapsed());

                stderr
                    .queue(cursor::MoveTo(0, 0))?
                    .queue(style::Print(format!(
                        "{bold_underline}crtcli tunneling{bold_underline:#}{underline} via {url} — {time} (Ctrl+C or q to quit){underline:#}",
                        bold_underline = Style::new().bold().underline(),
                        underline = Style::new().underline(),
                        url = client.base_url(),
                        time = elapsed_str,
                    )))?
                    .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?
                    .queue(cursor::MoveToNextLine(1))?;

                for listener_context in &listener_contexts {
                    render_listener_context(listener_context).await?;
                }

                stderr
                    .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?
                    .flush()?;

                tokio::select! {
                    biased;
                    _ = cancellation_token.cancelled() => { break; },
                    _ = tokio::time::sleep(refresh_interval) => {}
                }
            }

            return Ok(());

            async fn render_listener_context(
                listener_context: &ListenerContext,
            ) -> Result<(), std::io::Error> {
                let mut stderr = anstream::stderr();

                let active = listener_context.active_connections.load(Ordering::Relaxed);
                let total = listener_context.total_connections.load(Ordering::Relaxed);
                let rx = humanize_bytes(listener_context.total_rx_bytes.load(Ordering::Relaxed));
                let tx = humanize_bytes(listener_context.total_tx_bytes.load(Ordering::Relaxed));
                let bind_addr = listener_context.forward_mapping.bind_address;
                let host_addr = format!(
                    "{}:{}",
                    listener_context.forward_mapping.host,
                    listener_context.forward_mapping.host_port
                );

                let connections_padding = {
                    const MAX_TOTAL_CONNECTIONS_WIDTH: usize = 4;
                    const MAX_ACTIVE_CONNECTIONS_WIDTH: usize = 2;

                    let active_connections_length = match active {
                        _ if active >= 1000 => 4,
                        _ if active >= 100 => 3,
                        _ if active >= 10 => 2,
                        _ => 1,
                    };

                    let total_connections_length = match total {
                        _ if total >= 1000 => 4,
                        _ if total >= 100 => 3,
                        _ if total >= 10 => 2,
                        _ => 1,
                    };

                    MAX_TOTAL_CONNECTIONS_WIDTH + MAX_ACTIVE_CONNECTIONS_WIDTH
                        - active_connections_length
                        - total_connections_length
                };

                stderr
                    .queue(style::Print(format!(
                        "  {cyan}{bind_addr}{cyan:#} -> {magenta}{host_addr}{magenta:#}",
                        cyan = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Cyan))),
                        magenta = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Magenta))),
                    )))?
                    .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?
                    .queue(cursor::MoveToNextLine(1))?
                    .queue(style::Print(format!(
                        "    {empty:>connections_padding$}{yellow}{active}{yellow:#}/{dim}{total}{dim:#} connections,  TX: {green}{tx:>10}{green:#}, RX: {green}{rx:>10}{green:#}",
                        empty = "",
                        yellow = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Yellow))),
                        dim = Style::new().dimmed(),
                        green = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Green))),
                    )))?;

                if let Some(last_connection_error_str) = listener_context
                    .last_connection_error_str
                    .read()
                    .await
                    .clone()
                {
                    stderr
                        .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?
                        .queue(cursor::MoveToNextLine(1))?
                        .queue(style::Print(format!(
                            "    {red}--> Error: {last_connection_error_str} <--{red:#}",
                            red = Style::new().fg_color(Some(Color::Ansi(AnsiColor::Red))),
                        )))?;
                }

                stderr
                    .queue(terminal::Clear(terminal::ClearType::UntilNewLine))?
                    .queue(cursor::MoveToNextLine(1))?;

                Ok(())
            }
        }

        fn hide_tunneling_dashboard() {
            let _ = anstream::stderr().queue(cursor::Show);
            let _ = anstream::stderr().queue(terminal::LeaveAlternateScreen);
            let _ = terminal::disable_raw_mode();
        }

        fn start_polling_cancellation_token() -> CancellationToken {
            let mut event_stream = EventStream::new();
            let cancellation_token = CancellationToken::new();

            let is_close_event = |key_event: event::KeyEvent| {
                (key_event.code == event::KeyCode::Char('c')
                    && key_event.modifiers.contains(event::KeyModifiers::CONTROL))
                    || key_event.code == event::KeyCode::Char('q')
            };

            {
                let cancellation_token = cancellation_token.clone();

                tokio::spawn(async move {
                    loop {
                        if let Some(Ok(event::Event::Key(key_event))) = event_stream.next().await
                            && is_close_event(key_event)
                        {
                            let _ = cancellation_token.cancel();
                            break;
                        }

                        if cancellation_token.is_cancelled() {
                            break;
                        }
                    }
                });
            }

            cancellation_token
        }

        async fn start_tcp_listener_forwarding(
            client: Arc<CrtClient>,
            forward_mapping: &ForwardMappingArg,
        ) -> Result<(Arc<ListenerContext>, JoinHandle<()>), CommandDynError> {
            let listener = TcpListener::bind(forward_mapping.bind_address)
                .await
                .map_err(|err| {
                    TunnelCommandError::TcpListenerBind(forward_mapping.bind_address, err)
                })?;

            let listener_context = Arc::new(ListenerContext::new(
                client.clone(),
                forward_mapping.clone(),
            ));

            let handle = {
                let context = Arc::clone(&listener_context);

                tokio::spawn(async move {
                    loop {
                        if let Ok((tcp_stream, client_addr)) = listener.accept().await {
                            let context =
                                ListenerConnectionContext::new(Arc::clone(&context), client_addr);

                            tokio::spawn(async move {
                                let _ = process_tcp_listener_connection(tcp_stream, context).await;
                            });
                        }
                    }
                })
            };

            Ok((listener_context, handle))
        }

        async fn process_tcp_listener_connection(
            tcp_stream: TcpStream,
            connection_context: ListenerConnectionContext,
        ) -> CommandResult {
            let result = connection_context
                .listener
                .client
                .crtcli_tunneling_service()
                .connect(
                    &connection_context.listener.forward_mapping.host,
                    connection_context.listener.forward_mapping.host_port,
                )
                .await;

            connection_context
                .listener
                .apply_last_connection_error_from_result(&result)
                .await;

            let websocket = result?;

            let (mut tcp_read, mut tcp_write) = tokio::io::split(tcp_stream);
            let (mut ws_write, mut ws_read) = websocket.split();

            let _ = tokio::select! {
                _ = forward_tcp_to_ws(&mut tcp_read, &mut ws_write, &connection_context) => {},
                _ = forward_ws_to_tcp(&mut ws_read, &mut tcp_write, &connection_context) => {},
            };

            Ok(())
        }
    }
}

struct ListenerContext {
    client: Arc<CrtClient>,
    forward_mapping: ForwardMappingArg,
    active_connections: AtomicUsize,
    total_connections: AtomicUsize,
    total_tx_bytes: AtomicU64,
    total_rx_bytes: AtomicU64,
    last_connection_error_str: RwLock<Option<String>>,
}

impl ListenerContext {
    pub fn new(client: Arc<CrtClient>, forward_mapping: ForwardMappingArg) -> Self {
        Self {
            client,
            forward_mapping,
            active_connections: AtomicUsize::new(0),
            total_connections: AtomicUsize::new(0),
            total_tx_bytes: AtomicU64::new(0),
            total_rx_bytes: AtomicU64::new(0),
            last_connection_error_str: RwLock::new(None),
        }
    }

    pub async fn apply_last_connection_error_from_result<V>(
        &self,
        result: &Result<V, CrtClientError>,
    ) {
        match result {
            Ok(_) => {
                if self.last_connection_error_str.read().await.is_some() {
                    self.last_connection_error_str.write().await.take();
                }
            }
            Err(err) => {
                self.last_connection_error_str
                    .write()
                    .await
                    .replace(err.to_string());
            }
        }
    }
}

struct ListenerConnectionContext {
    listener: Arc<ListenerContext>,
    _client_addr: SocketAddr,
}

impl ListenerConnectionContext {
    pub fn new(listener: Arc<ListenerContext>, client_addr: SocketAddr) -> Self {
        listener.active_connections.fetch_add(1, Ordering::Relaxed);
        listener.total_connections.fetch_add(1, Ordering::Relaxed);

        Self {
            listener,
            _client_addr: client_addr,
        }
    }
}

impl Drop for ListenerConnectionContext {
    fn drop(&mut self) {
        self.listener
            .active_connections
            .fetch_sub(1, Ordering::Relaxed);
    }
}

async fn forward_ws_to_tcp<W>(
    ws_read: &mut SplitStream<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>>,
    tcp_write: &mut W,
    listener_context: &ListenerConnectionContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    W: AsyncWrite + Unpin,
{
    while let Some(message_result) = ws_read.next().await {
        match message_result {
            Ok(message) => match message {
                Message::Binary(data) => {
                    tcp_write.write_all(&data).await?;
                    tcp_write.flush().await?;

                    listener_context
                        .listener
                        .total_rx_bytes
                        .fetch_add(data.len() as u64, Ordering::Relaxed);
                }
                Message::Text(_) => {
                    return Err("websocket received text message, which is not supported".into());
                }
                Message::Close(_) => {
                    return Ok(());
                }
                _ => {}
            },
            Err(e) => {
                return Err(Box::new(e));
            }
        }
    }

    Ok(())
}

async fn forward_tcp_to_ws<R>(
    tcp_read: &mut R,
    ws_write: &mut SplitSink<WebSocketStream<impl AsyncRead + AsyncWrite + Unpin>, Message>,
    connection_context: &ListenerConnectionContext,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>>
where
    R: AsyncRead + Unpin,
{
    static WS_PING_INTERVAL: Duration = Duration::from_secs(30);

    let mut buffer = vec![0u8; 81920];

    loop {
        tokio::select! {
            message = tcp_read.read(&mut buffer) => {
                match message {
                    Ok(0) => {
                        let _ = ws_write.send(Message::Close(None)).await;

                        return Ok(());
                    }
                    Ok(n) => {
                        ws_write
                            .send(Message::Binary(buffer[..n].to_vec().into()))
                            .await?;
                        ws_write.flush().await?;

                        connection_context
                            .listener
                            .total_tx_bytes
                            .fetch_add(n as u64, Ordering::Relaxed);
                    }
                    Err(e) => {
                        if e.kind() == tokio::io::ErrorKind::WouldBlock {
                            tokio::task::yield_now().await;

                            continue;
                        }

                        return Err(Box::new(e));
                    }
                }
            },
            _ = tokio::time::sleep(WS_PING_INTERVAL) => {
                ws_write.send(Message::Ping(Bytes::new())).await?;
                ws_write.flush().await?;
            },
        }
    }
}
