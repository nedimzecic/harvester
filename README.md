Simple nginx log collector. It takes UDP input, syslog messages from nginx. Output is PostgreSQL.

Table is created this way:
```
CREATE TABLE IF NOT EXISTS nginx_access (
	time TIMESTAMPTZ NOT NULL,
	hostname TEXT NOT NULL,
	request_method TEXT NOT NULL,
	http_host TEXT NOT NULL,
	uri TEXT NOT NULL,
	status SMALLINT NOT NULL,
	bytes_sent INT NOT NULL,
	request_time REAL NOT NULL,
	remote_addr TEXT NOT NULL
);
```

TimescaleDB setup:
```
SELECT create_hypertable('nginx_access', 'time', if_not_exists => TRUE);
SELECT set_chunk_time_interval('nginx_access', INTERVAL '24 hours');
SELECT add_retention_policy('nginx_access', INTERVAL '7 days', if_not_exists => TRUE);
);
```

nginx.conf needs following log format:
```
log_format timescaledb "('$time_iso8601','$hostname','$request_method','$http_host','$uri',$status,$bytes_sent,$request_time,'$remote_addr'),";
```

and vhost access_log is configured this way:
```
access_log syslog:server=localhost:514 timescaledb;
```

Inserts are done with multirow value syntax using batch size.
