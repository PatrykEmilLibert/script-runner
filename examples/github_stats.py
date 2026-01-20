#!/usr/bin/env python3
"""
Example Python script for ScriptRunner
This demonstrates automatic dependency detection and installation
"""

import requests
import json
from datetime import datetime
from pathlib import Path

def fetch_github_stats(username: str) -> dict:
    """Fetch GitHub user statistics"""
    url = f"https://api.github.com/users/{username}"
    response = requests.get(url)
    data = response.json()
    
    return {
        "username": data.get("login"),
        "name": data.get("name"),
        "public_repos": data.get("public_repos"),
        "followers": data.get("followers"),
        "following": data.get("following"),
        "created_at": data.get("created_at"),
    }

def main():
    print("=" * 50)
    print("ScriptRunner - Example Script")
    print("=" * 50)
    print()
    
    username = "torvalds"  # Linus Torvalds
    
    print(f"Fetching GitHub stats for @{username}...")
    print()
    
    stats = fetch_github_stats(username)
    
    print(f"Name: {stats['name']}")
    print(f"Public Repos: {stats['public_repos']}")
    print(f"Followers: {stats['followers']}")
    print(f"Following: {stats['following']}")
    print(f"GitHub Member Since: {stats['created_at']}")
    print()
    print("Execution completed successfully!")
    print(f"Timestamp: {datetime.now().isoformat()}")

if __name__ == "__main__":
    main()
