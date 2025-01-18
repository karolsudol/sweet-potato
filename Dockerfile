FROM clickhouse/clickhouse-server:latest

# Copy the initialization script
COPY init-db.sh /docker-entrypoint-initdb.d/
RUN chmod +x /docker-entrypoint-initdb.d/init-db.sh 