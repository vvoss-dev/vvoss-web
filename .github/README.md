# GitHub Workflows

## Current Status: DISABLED

We're using server-side Git deployment instead of GitHub Actions.

## How Deployment Works Now

1. **Automatic**: Server checks GitHub every minute via cron
2. **Direct**: Git pull directly on the server
3. **Build**: Cargo builds in the jail
4. **Restart**: Service restarts automatically

## Server-Side Script
- Location: `/usr/local/bin/vvoss-deploy`
- Runs as: root via cron
- Schedule: Every minute (`* * * * *`)

## Documentation
See: `local/docs/server-jail-vvoss-deployment.md`

## Old Workflows
The `.yml.disabled` files are kept for reference but are not active.