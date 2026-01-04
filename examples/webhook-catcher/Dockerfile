FROM python:3.12-alpine

WORKDIR /app

# Create non-root user
RUN adduser -D -u 1000 appuser

# Copy and set ownership
COPY --chown=appuser:appuser main.py .

USER appuser

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD wget -q --spider http://127.0.0.1:8080/healthz || exit 1

ENTRYPOINT ["python", "main.py"]
