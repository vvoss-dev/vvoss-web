# Server Architecture - vvoss.dev

## Overview
The website runs on a FreeBSD 14.3-RELEASE server using Bastille jails with Unix socket communication.

## Server Access
```bash
ssh byvoss
# or
ssh freebsd@byvoss.tech
```

## Architecture

### Existing Infrastructure
```
proxy jail (10.0.0.1)
├── nginx reverse proxy
├── SSL termination (Let's Encrypt)
├── Unix sockets in /var/run/sockets/
└── Routes traffic to backend jails

byvoss_www jail (10.0.0.10)
├── byvoss.tech website
└── Communicates via unix:/var/run/sockets/byvoss_www.sock

deploy jail (10.0.0.100)
└── Deployment services

stormtales_master jail (10.0.0.20)
└── Stormtales project
```

### vvoss.dev Setup (To Be Created)
```
vvoss_www jail (10.0.0.11)
├── Rust application with Tera templates
├── Unix socket: /var/run/sockets/vvoss_www.sock
└── Web root: /usr/local/www/vvoss/
```

## Jail Creation Process

### 1. Create the Jail
```bash
sudo bastille create vvoss_www 14.3-RELEASE 10.0.0.11
```

### 2. Install Dependencies
The jail needs temporary internet access for package installation:
```bash
# Enable NAT temporarily
echo 'nat on vtnet0 from 10.0.0.11 to any -> (vtnet0)' | sudo tee -a /etc/pf.conf
sudo pfctl -f /etc/pf.conf

# Install Rust
sudo bastille pkg vvoss_www install -y rust

# Remove NAT (return to blind jail)
sudo sed -i '' '/10.0.0.11/d' /etc/pf.conf
sudo pfctl -f /etc/pf.conf
```

### 3. Mount Socket Directory
```bash
# Create socket directory in proxy jail
sudo bastille cmd proxy mkdir -p /var/run/sockets

# Mount to vvoss_www jail
sudo bastille mount vvoss_www \
    /usr/local/bastille/jails/proxy/root/var/run/sockets \
    /var/run/sockets nullfs rw 0 0
```

## nginx Configuration

### Proxy Jail Configuration
Location: `/usr/local/etc/nginx/sites-available/vvoss.conf`

```nginx
# Backend via Unix Socket
upstream vvoss_backend {
    server unix:/var/run/sockets/vvoss_www.sock;
}

# HTTP to HTTPS redirect
server {
    listen 10.0.0.1:80;
    server_name vvoss.dev www.vvoss.dev;
    
    location /.well-known/acme-challenge/ {
        root /var/www/acme;
    }
    
    location / {
        return 301 https://vvoss.dev$request_uri;
    }
}

# HTTPS Main Site
server {
    listen 10.0.0.1:443 ssl;
    http2 on;
    server_name vvoss.dev;
    
    ssl_certificate /usr/local/etc/letsencrypt/live/vvoss.dev/fullchain.pem;
    ssl_certificate_key /usr/local/etc/letsencrypt/live/vvoss.dev/privkey.pem;
    
    # Security headers
    add_header Strict-Transport-Security "max-age=31536000" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    
    location / {
        proxy_pass http://vvoss_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
    
    # Cache static assets
    location ~* \.(css|js|png|jpg|gif|svg|woff|woff2)$ {
        proxy_pass http://vvoss_backend;
        expires 1y;
        add_header Cache-Control "public, immutable";
    }
}

# WWW redirect
server {
    listen 10.0.0.1:443 ssl;
    http2 on;
    server_name www.vvoss.dev;
    
    ssl_certificate /usr/local/etc/letsencrypt/live/vvoss.dev/fullchain.pem;
    ssl_certificate_key /usr/local/etc/letsencrypt/live/vvoss.dev/privkey.pem;
    
    return 301 https://vvoss.dev$request_uri;
}
```

Enable the configuration:
```bash
sudo bastille cmd proxy ln -sf \
    /usr/local/etc/nginx/sites-available/vvoss.conf \
    /usr/local/etc/nginx/sites-enabled/
```

## SSL Certificate Setup

Using Let's Encrypt in the proxy jail:
```bash
# Install certbot if not present
sudo bastille pkg proxy install -y py39-certbot

# Create certificate
sudo bastille cmd proxy certbot certonly --webroot \
    -w /var/www/acme \
    -d vvoss.dev \
    -d www.vvoss.dev \
    --email admin@vvoss.dev \
    --agree-tos \
    --non-interactive

# Automatic renewal (add to crontab if not exists)
echo '0 3 * * * root /usr/local/bin/certbot renew --quiet --post-hook "service nginx reload"' >> /etc/crontab
```

## Service Management

### Create RC Script
Location: `/usr/local/etc/rc.d/vvoss-web` in the jail

```sh
#!/bin/sh
#
# PROVIDE: vvoss_web
# REQUIRE: NETWORKING
# KEYWORD: shutdown

. /etc/rc.subr

name="vvoss_web"
rcvar="${name}_enable"
command="/usr/local/bin/vvoss-web"
command_interpreter="/usr/local/bin/rust"
pidfile="/var/run/${name}.pid"

# Set working directory
start_precmd="${name}_prestart"
vvoss_web_prestart() {
    cd /usr/local/www/vvoss
}

load_rc_config $name
: ${vvoss_web_enable:="NO"}

run_rc_command "$1"
```

Enable the service:
```bash
sudo bastille cmd vvoss_www sysrc vvoss_web_enable="YES"
```

## Monitoring

### Check Services
```bash
# Jail status
sudo bastille list | grep vvoss

# Socket existence
sudo bastille cmd proxy ls -la /var/run/sockets/ | grep vvoss

# Service status
sudo bastille service vvoss_www vvoss_web status

# nginx status
sudo bastille service proxy nginx status
```

### View Logs
```bash
# Application logs
sudo bastille cmd vvoss_www tail -f /var/log/vvoss-web.log

# nginx access logs
sudo bastille cmd proxy tail -f /var/log/nginx/access.log | grep vvoss

# nginx error logs  
sudo bastille cmd proxy tail -f /var/log/nginx/error.log
```

## Important Paths

### On Host
- Jail root: `/usr/local/bastille/jails/vvoss_www/root/`
- Git repository: `/usr/local/git/vvoss.git` (if using git deployment)

### In vvoss_www Jail
- Application: `/usr/local/bin/vvoss-web`
- Web root: `/usr/local/www/vvoss/`
- Templates: `/usr/local/www/vvoss/templates/`
- Static files: `/usr/local/www/vvoss/static/`
- Socket: `/var/run/sockets/vvoss_www.sock`

### In proxy Jail
- Sites available: `/usr/local/etc/nginx/sites-available/`
- Sites enabled: `/usr/local/etc/nginx/sites-enabled/`
- SSL certificates: `/usr/local/etc/letsencrypt/live/vvoss.dev/`
- Socket directory: `/var/run/sockets/`