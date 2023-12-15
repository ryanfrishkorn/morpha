# Morpha

A personalized, configurable assistant that archives all conversations using [SQLite](https://sqlite.org/index.html).

## Overview

Previously, I created a simplistic OpenAI CLI for making one-off queries. It
became a very handy tool when I wanted to navigate a topic I was unfamiliar with.
Saving the information to a searchable database also proved useful, as I could
query information that I vaguely remembered weeks later, for a quick refresh.
There was a flashcard-like sort of learning reinforcement that I discovered.

This is a conversational CLI that archives all conversations using SQLite.
Full text search will be implemented using the [FTS5](https://sqlite.org/fts5.html) extension.

Future version will implement `/commands` that can be used in both interactive 
and non-interactive modes.

Ideas include:

```
(NOT YET IMPLEMENTED)

/search <term>
    search conversations for terms or topics and display or
    export structured data

/list <conversation|message>
    list all conversations or messages with relevant metadata

/cite <message> (include for context in current conversation)
    give the assistant context either in conversational text
    or adding the data to a payload

/explain <conversation|message>
    explain the topic of conversation in more detail or with
    respect to a specific point or idea

```

## Install

(rustup compatible method)

```shell
cargo install --path .
```

## Manual Install

builds binary to `target/release/`
```shell
cargo build --profile release
```

## Configuration

Ensure your shell is exporting a valid OpenAI API key.
```shell
export OPENAI_API_KEY="your-alphanumeric-key-here"
```

Place the `design/personality.md` in your home directory as `.morpha_profile` and
edit the personality description to your requirements. This will serve as the 
`instruction` for the OpenAI client.

```shell
cp data/personality.md ~/.morpha_profile
```

### Use

For help and options:
`morpha --help`

For an interactive conversation, run with your preferred options.

```shell
morpha
```

For a single prompt and response (non-interactive), pipe your query via standard
input. This reads all lines of input, and will exit after the first response.

```shell
bash-5.2$ echo "How does atmospheric pressure affect the boiling point of water?" | morpha
```

The response is printed to standard output in plain text easily piped somewhere
useful.
```
The atmospheric pressure indeed influences the boiling point of water. As the
elevation increases, atmospheric pressure decreases, leading to a lower boiling
point for water. Conversely, at lower elevations, where atmospheric pressure is
higher, the boiling point of water also increases. This relationship follows
the principle that higher pressure raises the boiling point, while lower
pressure reduces it. Such understanding is pivotal in various scientific and
practical applications, including the development of cooking techniques and the
operation of steam-based machinery.
```

## Archiving
All conversations are archived in `${HOME}/.morpha.sqlite3`
