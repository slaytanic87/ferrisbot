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

## Define Bot Task

create or adjust the bot role definition as enumerations (natural language) in the prompt template role_definition.md

**Variables placeholder**

Following placeholders must be used in your role definition template

Botname: {name}
No response flag: {NO_ACTION}
