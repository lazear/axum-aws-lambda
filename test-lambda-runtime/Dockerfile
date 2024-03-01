FROM python:3.9-slim as builder

WORKDIR /app

RUN apt-get update && \
    apt-get install -y --no-install-recommends gcc

COPY requirements.txt .
RUN python -m venv /opt/venv
ENV PATH="/opt/venv/bin:$PATH"
RUN pip install --upgrade pip && \
    pip install -r requirements.txt

COPY . .

FROM python:3.9-slim

WORKDIR /app

COPY --from=builder /opt/venv /opt/venv
COPY --from=builder /app/test_lambda_runtime.py .

ENV PATH="/opt/venv/bin:$PATH"

EXPOSE 5000

CMD ["python", "test_lambda_runtime.py"]
