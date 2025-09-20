# Rust Rocket + RabbitMQ + Observability (ELK & Prometheus/Grafana)

Production-ready starter:
- **API**: Rust (Rocket) — `GET /coins`, `GET /metrics`
- **Queue**: RabbitMQ (producer/consumer)
- **Logs**: ELK → Logstash (GELF) → Elasticsearch → Kibana
- **Metrics**: Prometheus → Grafana
- **One command** via Docker Compose

---

## Prerequisites
- Docker Desktop (Windows/macOS) or Docker Engine (Linux)
- Internet access (Docker images + Binance API)
- **Windows**: Ensure the project path is allowed in Docker Desktop → *Settings → Resources → File sharing*

---

## Quick Start

```bash
# 1) Clone
git clone https://github.com/yildirim7mustafa/rust-binance-api-with-RabbitMQ-ELK-Prometheus-Grafana.git
cd rust-binance-api-with-RabbitMQ-ELK-Prometheus-Grafana

# 2) Bring up the stack
docker compose up -d --build

# 3) Check containers
docker ps

| Service       | URL                                              | Notes                                 |
| ------------- | ------------------------------------------------ | ------------------------------------- |
| API (Rocket)  | [http://localhost:8000](http://localhost:8000)   | `GET /coins`, `GET /metrics`          |
| RabbitMQ UI   | [http://localhost:15672](http://localhost:15672) | user: `guest`, pass: `guest`          |
| Elasticsearch | [http://localhost:9200](http://localhost:9200)   | use `_cat/indices` to check indices   |
| Kibana        | [http://localhost:5601](http://localhost:5601)   | create Data View: `docker-logs-*`     |
| Prometheus    | [http://localhost:9090](http://localhost:9090)   | Targets page                          |
| Grafana       | [http://localhost:3000](http://localhost:3000)   | `admin/admin` (change on first login) |
