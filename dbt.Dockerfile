FROM python:3.9-slim

WORKDIR /dbt

# Install dbt-clickhouse
RUN pip install dbt-clickhouse

# Copy dbt project files
COPY ./dbt /dbt/

# Default command (can be overridden in docker-compose)
CMD ["tail", "-f", "/dev/null"] 