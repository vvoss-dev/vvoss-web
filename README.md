# vvoss.dev

Personal website built with Rust and Tera templates, running on FreeBSD.

## Technology Stack
- **Backend**: Rust with Actix-Web
- **Templates**: Tera template engine
- **Styling**: Pure CSS3 (no frameworks)
- **Server**: FreeBSD with Bastille Jails
- **Communication**: Unix Sockets

## Project Structure
```
├── src/                  # Rust application source
├── templates/            # Tera templates
│   ├── base.tera        # Base template
│   └── partials/        # Page templates
├── static/              # Static assets (CSS, JS, images)
├── scripts/             # Deployment scripts
└── docs/                # Documentation
```

## Development

### Prerequisites
- Rust (latest stable)
- Cargo

### Local Development
```bash
# Clone repository
git clone https://github.com/vvoss-dev/vvoss-web.git
cd vvoss-web

# Build and run
cargo run

# Build for production
cargo build --release
```

## Deployment

See [docs/DEPLOYMENT.md](docs/DEPLOYMENT.md) for detailed deployment instructions.

### Quick Deploy (after initial setup)
```bash
git push origin main
ssh byvoss "cd /usr/local/deploy/vvoss && ./update.sh"
```

## Documentation
- [Server Architecture](docs/SERVER.md) - FreeBSD jail setup and configuration
- [Deployment Guide](docs/DEPLOYMENT.md) - Deployment procedures and workflows

## License
© 2024 Vivian Voss - All rights reserved

This is a personal website project. The source code is proprietary and not licensed for reuse.