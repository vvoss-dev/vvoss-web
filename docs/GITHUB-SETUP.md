# GitHub Setup Guide

## Repository Setup

### 1. Create Repository on GitHub
```bash
# Go to https://github.com/new
# Repository name: vvoss-web
# Description: Personal website - vvoss.dev
# Private repository: Yes (recommended for personal website)
# Initialize: No (we already have local code)
```

### 2. Push Local Code to GitHub
```bash
# Add remote origin
git remote add origin https://github.com/vvoss-dev/vvoss-web.git

# Add all files
git add .
git commit -m "Initial commit: Rust + Tera website"

# Push to GitHub
git push -u origin main
```

## GitHub Secrets Configuration

For automated deployment, configure these secrets in GitHub:
**Settings → Secrets and variables → Actions → New repository secret**

### Required Secrets

#### DEPLOY_HOST
```
byvoss.tech
```

#### DEPLOY_USER
```
freebsd
```

#### DEPLOY_SSH_KEY
Generate a deployment key:
```bash
# On your local machine
ssh-keygen -t ed25519 -f ~/.ssh/vvoss-deploy -C "github-actions@vvoss.dev"

# Copy the private key content
cat ~/.ssh/vvoss-deploy
# Paste this as DEPLOY_SSH_KEY secret

# Add public key to server
ssh-copy-id -i ~/.ssh/vvoss-deploy.pub freebsd@byvoss.tech
```

## Branch Protection Rules

### Configure Main Branch Protection
1. Go to **Settings → Branches**
2. Add rule for `main` branch:
   - ✅ Require pull request before merging
   - ✅ Require status checks to pass before merging
     - Required checks: `test`
   - ✅ Require branches to be up to date
   - ✅ Include administrators (optional)

## GitHub Actions Workflows

### CI Workflow (`.github/workflows/ci.yml`)
- Runs on every push and pull request
- Executes tests, formatting checks, and clippy
- Builds release binary on main branch

### Deploy Workflow (`.github/workflows/deploy.yml`)
- Deploys automatically on push to main
- Can be triggered manually via workflow_dispatch
- Skips deployment for documentation-only changes

## Development Workflow

### Feature Development
```bash
# Create feature branch
git checkout -b feature/my-feature

# Make changes
# ... edit files ...

# Commit changes
git add .
git commit -m "feat: add new feature"

# Push to GitHub
git push origin feature/my-feature

# Create Pull Request on GitHub
# After review and CI passes, merge to main
```

### Hotfix
```bash
# Create from main
git checkout main
git pull origin main
git checkout -b hotfix/critical-fix

# Fix and push
git add .
git commit -m "fix: critical issue"
git push origin hotfix/critical-fix

# Create PR with "hotfix" label
```

## Manual Deployment Trigger

If automatic deployment is disabled or fails:

1. Go to **Actions → Deploy to Production**
2. Click **Run workflow**
3. Select branch: `main`
4. Click **Run workflow**

## Monitoring Deployments

### Check Deployment Status
- **Actions tab**: Shows all workflow runs
- **Deployments**: Under repo insights
- **Commit status**: Green checkmark = deployed

### Rollback Procedure
```bash
# Find last good commit
git log --oneline

# Revert to previous version
git revert HEAD
git push origin main
# This triggers new deployment with reverted code

# Or reset (force push - use carefully)
git reset --hard <commit-hash>
git push --force origin main
```

## SSH Key Rotation

Periodically rotate deployment keys:
```bash
# Generate new key
ssh-keygen -t ed25519 -f ~/.ssh/vvoss-deploy-new -C "github-actions@vvoss.dev"

# Update on server
ssh byvoss
echo "new-public-key-content" >> ~/.ssh/authorized_keys

# Update GitHub secret DEPLOY_SSH_KEY with new private key
# Test deployment
# Remove old key from authorized_keys
```

## Debugging Failed Deployments

### Check GitHub Actions Logs
1. Go to **Actions** tab
2. Click on failed workflow run
3. Expand failed step for details

### Common Issues
- **SSH connection failed**: Check DEPLOY_HOST and DEPLOY_SSH_KEY
- **Permission denied**: Verify DEPLOY_USER has sudo access
- **Build failed**: Check Rust version compatibility
- **Service won't start**: Check logs on server:
  ```bash
  ssh byvoss
  sudo bastille cmd vvoss_www tail -f /var/log/vvoss-web.log
  ```