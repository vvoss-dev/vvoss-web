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
└── .github/             # GitHub Actions workflows
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

Automated deployment via GitHub Actions on push to main branch.

## Documentation

Public documentation is in this README. Detailed server and deployment documentation is maintained in a separate private repository for security reasons.

**Note**: The `local/` directory is gitignored and contains a separate private Git repository with server configuration and documentation.

## License
© 2024 Vivian Voss - All rights reserved

This is a personal website project. The source code is proprietary and not licensed for reuse.