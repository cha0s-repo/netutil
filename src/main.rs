use std::str::FromStr;

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

use types::{HttpMethod, Protocol, detect_protocol, parse_addr, parse_headers, ensure_http_url, ensure_ws_url};

mod types;

/// netutil - A network protocol utility tool
#[derive(Parser, Debug)]
#[command(name = "netutil", version, about = "Network protocol utility (HTTP/WS/TCP/UDP/ICMP)")]
struct Cli {
    /// Target URL or address
    target: String,

    /// HTTP method (GET/POST/PUT/DELETE/HEAD/OPTIONS/PATCH)
    #[arg(short, long, default_value = "GET")]
    r#type: String,

    /// Request body / data to send
    #[arg(short, long)]
    data: Option<String>,

    /// Custom header (repeatable)
    #[arg(short = 'H', long)]
    header: Vec<String>,

    /// ICMP ping count
    #[arg(short, long, default_value = "4")]
    count: u32,

    /// WebSocket message
    #[arg(short, long)]
    message: Option<String>,

    /// Force WebSocket
    #[arg(long)]
    ws: bool,

    /// Force ICMP
    #[arg(long)]
    icmp: bool,

    /// Timeout in seconds
    #[arg(long, default_value = "10")]
    timeout: u64,

    /// Show response headers
    #[arg(long)]
    show_headers: bool,

    /// Verbose output
    #[arg(short, long)]
    verbose: bool,
}

fn status_color(status: u16) -> &'static str {
    match status {
        200..=299 => "green",
        300..=399 => "yellow",
        400..=499 => "red",
        500..=599 => "bright red",
        _ => "white",
    }
}

// ─── HTTP ──────────────────────────────────────────────
async fn run_http(cli: &Cli) -> Result<()> {
    let method: HttpMethod = cli.r#type.parse::<HttpMethod>().map_err(|e| anyhow::anyhow!(e))?;
    let url = ensure_http_url(&cli.target);
    let headers = parse_headers(&cli.header);

    println!("{} {}", ">".cyan(), format!("{} {}", method, url).bold());

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(cli.timeout))
        .build()?;

    let mut req = match method {
        HttpMethod::Get => client.get(&url),
        HttpMethod::Post => client.post(&url),
        HttpMethod::Put => client.put(&url),
        HttpMethod::Delete => client.delete(&url),
        HttpMethod::Head => client.head(&url),
        HttpMethod::Options => client.request(reqwest::Method::OPTIONS, &url),
        HttpMethod::Patch => client.patch(&url),
    };

    for (k, v) in &headers {
        req = req.header(k.as_str(), v.as_str());
        if cli.verbose {
            println!("{} {}: {}", ">".cyan(), k.green(), v.dimmed());
        }
    }

    if let Some(body) = &cli.data {
        req = req.body(body.clone());
        println!("{} Content-Length: {}", ">".cyan(), body.len());
        if cli.verbose {
            println!("{} {}", ">".cyan(), body.dimmed());
        }
    }

    let resp = req.send().await?;
    let status = resp.status();

    println!(
        "{} {}",
        "<".cyan(),
        format!("{} {}", status.as_u16(), status.canonical_reason().unwrap_or("Unknown"))
            .color(status_color(status.as_u16())).bold()
    );

    if cli.show_headers {
        for (k, v) in resp.headers() {
            println!("{} {}: {}", "<".cyan(), k.as_str().green(), v.to_str().unwrap_or("?").dimmed());
        }
    }

    let body = resp.text().await?;
    if body.len() > 10000 {
        println!("{} ", "<".cyan());
        println!("{}", &body[..10000]);
        println!("{}", format!("... (truncated, {} bytes total)", body.len()).yellow());
    } else {
        println!("{} ", "<".cyan());
        println!("{}", body);
    }

    Ok(())
}

// ─── WebSocket ─────────────────────────────────────────
async fn run_ws(cli: &Cli) -> Result<()> {
    use futures_util::SinkExt;

    let url = ensure_ws_url(&cli.target);
    let msg = cli.message.as_deref().unwrap_or("hello");

    println!("{} Connecting to {}", ">".cyan(), url.bold());

    let (ws_stream, _) = tokio_tungstenite::connect_async(&url).await
        .map_err(|e| anyhow::anyhow!("WebSocket connect failed: {}", e))?;

    println!("{} Connected", "<".green().bold());
    println!("{} {}", ">".cyan(), msg.bold());

    let (mut write, mut read) = futures_util::StreamExt::split(ws_stream);

    write.send(tokio_tungstenite::tungstenite::Message::Text(msg.into())).await?;

    let timeout = std::time::Duration::from_secs(cli.timeout);
    let result = tokio::time::timeout(timeout, async {
        use futures_util::StreamExt;
        for _ in 0..5 {
            match read.next().await {
                Some(Ok(msg)) => match msg {
                    tokio_tungstenite::tungstenite::Message::Text(t) => {
                        println!("{} {}", "<".cyan(), t);
                    }
                    tokio_tungstenite::tungstenite::Message::Binary(d) => {
                        println!("{} [binary] {} bytes", "<".cyan(), d.len());
                    }
                    tokio_tungstenite::tungstenite::Message::Close(_) => {
                        println!("{} Closed", "<".yellow());
                        return;
                    }
                    _ => {}
                },
                Some(Err(e)) => {
                    eprintln!("{} {}", "!".red(), e);
                    return;
                }
                None => {
                    println!("{} Closed", "<".yellow());
                    return;
                }
            }
        }
    }).await;

    if result.is_err() {
        println!("{} Timeout after {}s", "!".yellow(), cli.timeout);
    }

    Ok(())
}

// ─── TCP ───────────────────────────────────────────────
async fn run_tcp(cli: &Cli) -> Result<()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let (host, port) = parse_addr(&cli.target, 80).map_err(|e| anyhow::anyhow!(e))?;
    let addr = format!("{}:{}", host, port);

    println!("{} Connecting to {}", ">".cyan(), addr.bold());

    let timeout = std::time::Duration::from_secs(cli.timeout);
    let stream = tokio::time::timeout(timeout, tokio::net::TcpStream::connect(&addr)).await??;
    println!("{} Connected", "<".green().bold());

    if let Some(data) = &cli.data {
        println!("{} {}", ">".cyan(), data.dimmed());
        let mut stream = stream;
        stream.write_all(data.as_bytes()).await?;
        stream.flush().await?;

        let mut buf = vec![0u8; 65536];
        match tokio::time::timeout(timeout, stream.read(&mut buf)).await? {
            Ok(n) if n > 0 => {
                let resp = String::from_utf8_lossy(&buf[..n]);
                println!("{} ", "<".cyan());
                println!("{}", resp);
            }
            _ => println!("{} No response (timeout)", "!".yellow()),
        }
    }

    Ok(())
}

// ─── UDP ───────────────────────────────────────────────
async fn run_udp(cli: &Cli) -> Result<()> {
    let (host, port) = parse_addr(&cli.target, 53).map_err(|e| anyhow::anyhow!(e))?;
    let addr = format!("{}:{}", host, port);

    println!("{} UDP target: {}", ">".cyan(), addr.bold());

    let socket = tokio::net::UdpSocket::bind("0.0.0.0:0").await?;

    if let Some(data) = &cli.data {
        let n = socket.send_to(data.as_bytes(), &addr).await?;
        println!("{} Sent {} bytes", ">".cyan(), n);

        let timeout = std::time::Duration::from_secs(cli.timeout);
        let mut buf = vec![0u8; 65536];

        match tokio::time::timeout(timeout, socket.recv_from(&mut buf)).await? {
            Ok((n, src)) => {
                println!("{} Received {} bytes from {}", "<".cyan(), n, src);
                if cli.verbose {
                    let resp = String::from_utf8_lossy(&buf[..n]);
                    println!("{} {}", "<".cyan(), resp);
                }
            }
            Err(_) => println!("{} Timeout after {}s", "!".yellow(), cli.timeout),
        }
    }

    Ok(())
}

// ─── ICMP ──────────────────────────────────────────────
fn run_icmp(cli: &Cli) -> Result<()> {
    let (host, _) = parse_addr(&cli.target, 0).map_err(|e| anyhow::anyhow!(e))?;

    let mut cmd = std::process::Command::new("ping");
    cmd.arg("-c").arg(cli.count.to_string());
    cmd.arg("-W").arg(cli.timeout.to_string());
    if cli.verbose { cmd.arg("-v"); }
    cmd.arg(&host);

    let status = cmd.status()?;
    if !status.success() {
        anyhow::bail!("Ping to {} failed: {:?}", host, status.code());
    }

    Ok(())
}

// ─── Main ──────────────────────────────────────────────
#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let protocol = detect_protocol(&cli.target, cli.ws, cli.icmp);

    match protocol {
        Protocol::Http => run_http(&cli).await,
        Protocol::WebSocket => run_ws(&cli).await,
        Protocol::Tcp => run_tcp(&cli).await,
        Protocol::Udp => run_udp(&cli).await,
        Protocol::Icmp => {
            run_icmp(&cli)?;
            Ok(())
        }
    }
}
