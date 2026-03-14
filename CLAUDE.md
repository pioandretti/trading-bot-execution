# CLAUDE.md — trading-bot-execution

Nur der Execution Bot (Rust). Einzige Aufgabe: Orders ausführen.

SIGNALE VON:   Redis Streams auf 10.0.0.1:6379 (Brain Server)
MELDEN AN:     REST API auf 10.0.0.1:8080 (Brain Server)

NIEMALS hier: ML-Logik, API-Abfragen, eigenes Scoring

Bei SYNC_REQUIRED.md in Repo 1:
  → src/models.rs entsprechend synchronisieren
