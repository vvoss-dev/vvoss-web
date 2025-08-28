#!/bin/sh
# Local setup script for git deployment
# Run this once on your development machine

echo "Setting up git remote for vvoss.dev deployment"

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo "Initializing git repository..."
    git init
fi

# Add production remote if not exists
if ! git remote | grep -q "production"; then
    echo "Adding production remote..."
    git remote add production byvoss:/usr/local/git/vvoss.git
    echo "Production remote added!"
else
    echo "Production remote already exists"
fi

echo ""
echo "Setup complete!"
echo ""
echo "To deploy:"
echo "  git add ."
echo "  git commit -m 'Your commit message'"
echo "  git push production main"
echo ""
echo "First time setup on server:"
echo "  ssh byvoss"
echo "  sudo mkdir -p /usr/local/git/vvoss.git"
echo "  sudo git init --bare /usr/local/git/vvoss.git"
echo "  # Then copy scripts/git-post-receive to /usr/local/git/vvoss.git/hooks/post-receive"