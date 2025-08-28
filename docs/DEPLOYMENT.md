# Deployment Guide

## Development Workflow

### Local Development
```bash
# Run locally with mock socket
cargo run

# Build release version
cargo build --release

# Run tests
cargo test
```

### Production Deployment

## Option 1: GitHub-based Deployment (Recommended)

### Initial Repository Setup
```bash
# Clone from GitHub
git clone https://github.com/vvoss-dev/vvoss-web.git
cd vvoss-web

# Or if starting fresh, push to GitHub
git remote add origin https://github.com/vvoss-dev/vvoss-web.git
git push -u origin main
```

### Server Setup for GitHub Deployment

#### Method A: Direct Pull from GitHub
```bash
# On server, create deployment directory
ssh byvoss
sudo mkdir -p /usr/local/deploy/vvoss
cd /usr/local/deploy/vvoss

# Clone repository
git clone https://github.com/vvoss-dev/vvoss-web.git .

# Create update script
cat > update.sh << 'EOF'
#!/bin/sh
git pull origin main
cargo build --release
sudo bastille service vvoss_www vvoss_web stop
sudo cp target/release/vvoss-web /usr/local/bastille/jails/vvoss_www/root/usr/local/bin/
sudo cp -r templates static /usr/local/bastille/jails/vvoss_www/root/usr/local/www/vvoss/
sudo bastille cmd vvoss_www chown -R www:www /usr/local/www/vvoss
sudo bastille service vvoss_www vvoss_web start
EOF
chmod +x update.sh
```

#### Method B: GitHub Webhook + Local Git Mirror
```bash
# Create bare repository as mirror
sudo mkdir -p /usr/local/git/vvoss.git
sudo git init --bare /usr/local/git/vvoss.git

# Add GitHub as remote in bare repo
cd /usr/local/git/vvoss.git
git remote add github https://github.com/vvoss-dev/vvoss-web.git

# Install post-receive hook
# Copy scripts/git-post-receive to /usr/local/git/vvoss.git/hooks/post-receive
sudo chmod +x /usr/local/git/vvoss.git/hooks/post-receive

# Setup webhook receiver (in deploy jail)
# This will pull from GitHub and trigger local deployment
```

### Local Development Workflow
```bash
# Work on feature
git checkout -b feature/my-feature
# ... make changes ...
git add .
git commit -m "Add feature"

# Push to GitHub
git push origin feature/my-feature

# Create Pull Request on GitHub
# After merge to main, deploy manually or via webhook
```

### Manual Deployment from GitHub
```bash
ssh byvoss
cd /usr/local/deploy/vvoss
./update.sh
```

## Option 2: Direct File Transfer

### Build and Transfer
```bash
# Build locally
cargo build --release

# Transfer files
rsync -avz \
    --exclude='.git/' \
    --exclude='target/' \
    --exclude='docs/' \
    ./ byvoss:/tmp/vvoss-deploy/

# Deploy on server
ssh byvoss
sudo cp -r /tmp/vvoss-deploy/* /usr/local/bastille/jails/vvoss_www/root/usr/local/www/vvoss/
sudo bastille cmd vvoss_www chown -R www:www /usr/local/www/vvoss
```

## Build Inside Jail (Alternative)

If Rust is installed in the jail:
```bash
# Transfer source
rsync -avz ./ byvoss:/tmp/vvoss-src/

# Build in jail
ssh byvoss
sudo cp -r /tmp/vvoss-src /usr/local/bastille/jails/vvoss_www/root/tmp/
sudo bastille cmd vvoss_www sh -c "cd /tmp/vvoss-src && cargo build --release"
sudo bastille cmd vvoss_www cp /tmp/vvoss-src/target/release/vvoss-web /usr/local/bin/
```

## Service Management

### Restart Application
```bash
ssh byvoss
sudo bastille service vvoss_www vvoss_web restart
```

### Reload nginx Configuration
```bash
ssh byvoss
sudo bastille service proxy nginx reload
```

## Rollback Procedure

### Using Git
```bash
# Revert to previous commit
git revert HEAD
git push production main

# Or reset to specific commit
git reset --hard <commit-hash>
git push --force production main
```

### Manual Rollback
Keep previous binary version:
```bash
# Before deployment
sudo bastille cmd vvoss_www cp /usr/local/bin/vvoss-web /usr/local/bin/vvoss-web.backup

# Rollback if needed
sudo bastille cmd vvoss_www mv /usr/local/bin/vvoss-web.backup /usr/local/bin/vvoss-web
sudo bastille service vvoss_www vvoss_web restart
```

## Health Checks

### After Deployment
```bash
# Check if socket exists
ssh byvoss "sudo bastille cmd proxy ls -la /var/run/sockets/vvoss_www.sock"

# Test via curl
ssh byvoss "sudo bastille cmd proxy curl --unix-socket /var/run/sockets/vvoss_www.sock http://localhost/"

# Check public access
curl -I https://vvoss.dev
```

## Troubleshooting

### Socket Issues
```bash
# Check socket permissions
sudo bastille cmd vvoss_www ls -la /var/run/sockets/

# Restart service to recreate socket
sudo bastille service vvoss_www vvoss_web restart
```

### Build Issues
```bash
# Check Rust version in jail
sudo bastille cmd vvoss_www rustc --version

# Update Rust if needed
sudo bastille cmd vvoss_www rustup update
```

### Permission Issues
```bash
# Fix ownership
sudo bastille cmd vvoss_www chown -R www:www /usr/local/www/vvoss
sudo bastille cmd vvoss_www chmod 755 /usr/local/bin/vvoss-web
```