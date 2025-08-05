# aiced-cli

AI-powered code analysis tool with interactive diff viewer and automated change application.

## Features

- **Multi-repository analysis**: Analyze multiple Git repositories from a single configuration
- **AI-powered insights**: Uses Anthropic's Claude API for intelligent code analysis
- **Interactive diff viewer**: Web-based interface to review and selectively apply changes
- **Flexible configuration**: Support for custom analysis profiles and repository-specific settings
- **Automated workflows**: Optional PR creation and notification systems
- **Comprehensive validation**: Built-in configuration and repository validation

## Installation

```bash
# Clone the repository
git clone https://github.com/sheikh1309/ailyzer-cli
cd aiced-cli

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

## Prerequisites

- Rust 1.70+
- Anthropic API key (set as `ANTHROPIC_API_KEY` environment variable)
- Git repositories to analyze

## Quick Start

1. **Initialize configuration:**
   ```bash
   aiced init
   ```

2. **Configure your repositories** by editing the generated config file

3. **Analyze repositories:**
   ```bash
   # Analyze all configured repositories
   aiced analyze

   # Analyze specific repository
   aiced analyze --repo my-project

   # Analyze with custom profile
   aiced analyze --profile security-focused
   ```

4. **Review changes** in the interactive web interface that opens automatically

5. **Validate configuration:**
   ```bash
   aiced validate
   ```

## Commands

### `aiced init`
Creates sample configuration files with multi-repository setup.

### `aiced analyze [OPTIONS]`
Analyzes configured repositories and presents changes via interactive diff viewer.

**Options:**
- `--repo <NAME>`: Analyze specific repository
- `--tags <TAGS>`: Filter analysis by tags
- `--profile <PROFILE>`: Use specific analysis profile

### `aiced list`
Lists all configured repositories with their settings.

### `aiced validate`
Validates configuration files and repository paths.

### `aiced dashboard --port <PORT>` 
Starts web dashboard (planned feature).

### `aiced history [OPTIONS]`
Shows analysis history (planned feature).

**Options:**
- `--days <DAYS>`: Number of days to show (default: 7)

## Configuration

The tool uses TOML configuration files for repository and analysis settings. Key configuration areas:

- **Repository settings**: Path, auto-pull, PR creation preferences
- **AI provider settings**: API keys, model preferences
- **Analysis profiles**: Custom prompts and filtering rules
- **Notification settings**: Slack, email, webhooks
- **Output preferences**: Formatting and verbosity levels

## Interactive Diff Viewer

The web-based diff viewer provides:
- Side-by-side code comparison
- Selective change application
- Change categorization and filtering
- Real-time validation feedback
- Session management with timeout handling

## Architecture

- **Command Runner**: Central orchestrator for all CLI commands
- **Repository Manager**: Handles multi-repository operations
- **Code Analyzer**: AI-powered analysis engine
- **Diff Server**: Web interface for change review
- **File Modifier**: Safe change application with validation
- **Configuration System**: Flexible TOML-based configuration

## Development Status

**Implemented:**
- ✅ Core CLI interface and commands
- ✅ Multi-repository analysis
- ✅ Interactive diff viewer
- ✅ Configuration management
- ✅ AI integration (Anthropic Claude)
- ✅ Error handling and validation

**Planned:**
- 🚧 Web dashboard with historical data
- 🚧 PR creation automation
- 🚧 Notification systems (Slack, email)
- 🚧 Plugin system
- 🚧 Analysis result caching
- 🚧 Performance metrics and monitoring

## Environment Variables

```bash
export ANTHROPIC_API_KEY="your-api-key-here"
export RUST_LOG="info"  # Optional: for detailed logging
```

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## License

MIT License - see LICENSE file for details.

## Support

For issues and questions:
- Check the configuration with `aiced validate`
- Enable debug logging with `RUST_LOG=debug`
- Review error messages for suggestions
- Open issues on the GitHub repository

