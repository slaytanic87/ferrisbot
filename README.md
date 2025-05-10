# Ferrisbot

<p align="center"><img src="ferrisbot_logo.jpg" alt="ferrisbot" height="300px"></p>

## What is this?

Ferrisbot is a Rust based chat bot, designed to used with Telegram messenger.

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

**Remember: The models you choose must have tool support!**

## Setup Telegram token & parameters

```bash
export TELEGRAM_TOKEN = <MY_TELEGRAM_BOT_TOKEN>
export OLLAMA_HOST_ADDR = "http:localhost"
export OLLAMA_PORT = 11434
export LLM_MODEL = "llama3.2:latest"
export BOT_NAME = "Kate"
```

## Define Bot Task

create or adjust the bot role definition as enumerations (natural language) in the prompt template role_definition.md

### Variables placeholder

Following placeholders must be used in your role definition prompt template

-**Botname**: {name} - replace with the name of your bot which was given

-**No action flag**: {NO_ACTION} - This internal flag is important when you define a situation in the template where the bot should not reponse to a message

#### Notes

- For the Telegram api framework, I'm using my fork version of [mobot](https://github.com/slaytanic87/mobot), where have implemented the missing capabilities: Restriction of chat members, forum message support and get adminitrator member list.
When the features are fully tested and the time allows me, I will create a PR to contribute back if they want.
