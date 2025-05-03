# What is this?

Ferrisbot is a Rust based bot designed for Telegram messenger.

## Setup LLM

### Install Ollama

Download and install Ollama

[Download URL](https://ollama.com/download)

### setup LLM modell

Llama3.2

```bash
ollama run llama3.2:latest
```

or

mistral-nemo 12B

```bash
ollama run mistral-nemo:12b
```

## Setup Telegram token & parameters

```bash
export TELEGRAM_TOKEN = <MY_TELEGRAM_BOT_TOKEN>
export OLLAMA_HOST_ADDR = "http:localhost"
export OLLAMA_PORT = 11434
export LLM_MODEL = "llama3.2:latest"
```
