# vvoss.dev Website Project

## Project Overview
Personal website for vvoss.dev, deployed on FreeBSD server using jail architecture.

## Technical Stack
- **Server**: FreeBSD with Bastille jail management
- **Web Server**: nginx (reverse proxy in proxy jail, web server in vvoss_www jail)
- **Frontend**: Static HTML/CSS/JavaScript
- **Domain**: vvoss.dev

## Server Architecture
- **proxy jail** (10.0.0.1): nginx reverse proxy with internet access
- **vvoss_www jail** (10.0.0.11): Isolated web server (blind jail pattern)
- Following the same pattern as existing byvoss_www jail

## Project Structure
```
vvoss.dev/
├── index.html              # Main landing page
├── pages/                  # Additional pages
│   ├── impressum.html     # Legal/Imprint page
│   └── portfolio.html     # Portfolio page
├── styles/                 # CSS files with responsive design
│   ├── base.css           # Base styles
│   ├── phone.css          # Mobile styles
│   ├── tablet.css         # Tablet styles
│   ├── screen.css         # Desktop styles
│   └── wide.css           # Wide screen styles
├── scripts/                # JavaScript files
│   └── base.js            # Core functionality
├── docs/                   # Documentation
│   └── server-guide.md    # Server setup and deployment guide
└── CLAUDE.md              # This file - project documentation
```

## Development Guidelines

### Language Conventions
- **Communication**: German in chat/discussions
- **Code**: British English for individual names and comments
- **Native language elements**: Keep in their original form

### Code Standards
- **HTML**: Pure HTML5 standard with features up to 2023
- **CSS**: Pure CSS3 standard with features up to 2023
- **JavaScript**: Only when absolutely necessary, no frameworks
- **Browser Support**: Full compatibility across all major browsers
- **No Dependencies**: No external libraries or frameworks

### Code Style
- Semantic HTML5 elements for structure
- Modern CSS3 features (Grid, Flexbox, Custom Properties)
- Responsive design using CSS media queries
- Accessibility-first approach (ARIA labels, semantic markup)
- Performance optimised (minimal HTTP requests, optimised assets)

## Deployment

### Local Development
```bash
# Serve locally for testing
python3 -m http.server 8000
# or
npx http-server
```

### Production Deployment
```bash
# Build and sync to server
rsync -avz --delete ./ freebsd@byvoss.tech:/tmp/vvoss-deploy/

# On server
ssh freebsd@byvoss.tech
sudo cp -r /tmp/vvoss-deploy/* /usr/local/bastille/jails/vvoss_www/root/usr/local/www/vvoss.dev/
sudo bastille cmd vvoss_www chown -R www:www /usr/local/www/vvoss.dev
```

## Security Considerations
- Jail isolation (blind jail pattern - no internet access)
- SSL/TLS via Let's Encrypt
- Security headers configured in nginx
- Regular security updates via FreeBSD pkg

## Monitoring
- nginx access/error logs in vvoss_www jail
- Service status checks via bastille commands
- ZFS snapshots for backup

## Future Considerations
- [ ] Implement build process if site complexity grows
- [ ] Add CI/CD pipeline for automated deployments
- [ ] Consider static site generator if content management needed
- [ ] Implement analytics (privacy-focused solution)

## Related Documentation
- [Server Setup Guide](docs/server-guide.md) - Detailed server configuration
- [FreeBSD Handbook](https://docs.freebsd.org/en/books/handbook/) - OS documentation
- [Bastille Documentation](https://bastillebsd.org/) - Jail management

## Contact
- **Domain**: vvoss.dev
- **Server**: byvoss.tech
- **Admin**: admin@vvoss.dev