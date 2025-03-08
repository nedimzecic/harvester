use std::sync::Arc;
use tokio::net::UdpSocket;
use tokio::sync::mpsc;
use tokio_postgres::{Client, NoTls};

const BATCH_SIZE: usize = 100;
const UDP_BUFFER_SIZE: usize = 1316;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let socket = UdpSocket::bind("0.0.0.0:514").await?;
	let (tx, mut rx) = mpsc::channel(10_000);

	let (pg_client, connection) = tokio_postgres::connect("host=timescaledb dbname=nginx user=nginx password=nginx", NoTls).await?;

	tokio::spawn(async move {
		if let Err(e) = connection.await {
			eprintln!("Postgres connection error: {}", e);
		}
	});

	let pg_client = Arc::new(pg_client);
	let db_client = Arc::clone(&pg_client);

	tokio::spawn(async move {
		let mut batch = Vec::with_capacity(BATCH_SIZE);
		while let Some(msg) = rx.recv().await {
			batch.push(msg);
			if batch.len() >= BATCH_SIZE {
				if let Err(e) = insert_batch(&db_client, &batch).await {
					eprintln!("Failed to insert batch: {}", e);
				}
				batch.clear();
			}
		}
	});

	let mut buf = vec![0; UDP_BUFFER_SIZE];
	loop {
		let (len, _addr) = socket.recv_from(&mut buf).await?;
		if let Ok(msg) = std::str::from_utf8(&buf[..len]) {
			let _ = tx.send(msg.to_string()).await;
		}
	}
}

async fn insert_batch(client: &Client, batch: &[String]) -> Result<(), Box<dyn std::error::Error>> {
	let mut query = "INSERT INTO nginx_access (time,hostname,request_method,http_host,uri,status,bytes_sent,request_time,remote_addr) VALUES ".to_string();

	for msg in batch {
		let line: Vec<&str> = msg.split(": ").collect();
		query.push_str(line[1]);
	}

	query.truncate(query.len() - 1);
	client.simple_query(&query).await?;

	Ok(())
}
