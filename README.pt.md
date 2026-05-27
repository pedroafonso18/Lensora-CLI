# Lensora CLI

Lensora é uma CLI em Rust para revisar diffs não commitados do Git com a ajuda de agentes especializados de LLM.

## Visão geral

O fluxo atual da ferramenta é:

1. Ler mudanças não commitadas do repositório Git.
2. Mostrar uma seleção interativa dos arquivos alterados.
3. Permitir escolher um arquivo, vários ou todos.
4. Carregar os prompts dos agentes dentro da pasta `agents/`.
5. Enviar cada diff selecionado para um provider de LLM.
6. Exibir um resumo organizado por agente no terminal.

## Funcionalidades

- Leitura de diff do working tree e do staged area.
- Seleção interativa de arquivos com menu colorido.
- Suporte a múltiplos providers:
	- Anthropic
	- OpenAI
- Configuração via arquivo TOML.
- Suporte a chave direta no TOML com fallback opcional para variável de ambiente.
- Carregamento dos agentes a partir de arquivos `.md` em `agents/`.
- Saída organizada no terminal com espaçamento entre os previews.
- Fluxo apenas de revisão: o Lensora não gera nem aplica patches.

## Estado atual

O projeto já possui:

- CLI base em Rust com `clap`
- Loader de configuração TOML
- Leitura e agrupamento de diffs por arquivo
- Menu interativo para seleção de arquivos
- Orquestração dos agentes de review
- Clientes HTTP para Anthropic e OpenAI
- Formatação colorida no terminal

## Estrutura do projeto

```text
.
├── Cargo.toml
├── README.md
├── README.pt.md
├── lensora.toml
├── lensora.example.toml
├── agents/
└── src/
		└── cli/
```

## Configuração

Crie um arquivo `lensora.toml` a partir do exemplo:

```bash
cp lensora.example.toml lensora.toml
```

Exemplo de configuração:

```toml
[provider]
name = "anthropic"
model = "claude-sonnet-4-6"
api_key = "SUA_CHAVE_AQUI"

[ignore]
paths = ["target/", "Cargo.lock"]

[repo]
preconditions = ["Focus on correctness, regressions, and security."]

[output]
path = "lensora-review.md"
```

### Provider

- `name`: `anthropic` ou `openai`
- `model`: modelo da LLM
- `api_key`: chave direta no TOML

Se `api_key` não estiver definido, o Lensora tenta usar:

- `ANTHROPIC_API_KEY` para Anthropic
- `OPENAI_API_KEY` para OpenAI

### Ignore

Defina caminhos ou padrões que devem ser ignorados na coleta do diff.

### Repo

Use este bloco para registrar precondições, observações ou contexto fixo sobre a codebase que deve entrar na revisão.

### Output

O campo `output.path` está reservado para exportação futura do relatório e já faz parte da configuração.

## Agentes

Cada arquivo `.md` dentro de `agents/` representa um agente especializado de review.

Exemplos atuais:

- `bug.md`
- `consistency.md`
- `functionality.md`
- `security.md`
- `style.md`

Esses agentes são carregados em tempo de execução e recebem o diff selecionado mais o contexto da revisão.

## Como executar

```bash
cargo run
```

Para inspecionar apenas a configuração carregada:

```bash
cargo run -- --print-config
```

## Saída no terminal

O Lensora mostra:

- cabeçalho inicial com o caminho da configuração
- lista de arquivos alterados
- menu de seleção com cor e espaçamento
- preview de resultados por agente, separado por blocos

## Capturas de tela

Requisição:

![Captura da requisição do Lensora](screenshots/request.png)

Resposta:

![Captura da resposta do Lensora](screenshots/response.png)

## Desenvolvimento

Valide a compilação com:

```bash
cargo check
```

Execute a suíte de testes com:

```bash
cargo test
```