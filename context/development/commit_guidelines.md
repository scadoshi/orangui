# Commit Guidelines

- Concise, one-line messages (multi-line only when many changes)
- Group related files logically
- No emojis
- Use `git diff` to understand changes before committing
- **Never** include AI-agent signatures in commits
    - No "Co-Authored-By: Claude..."
    - No "Generated with [Claude Code]..."
    - No "Written with the help of ..."
- Never push without being asked

## Before you push

```bash
node scripts/sync-themes.mjs          # if zwipe's palettes changed
node --check orangui-fluent.user.js
```

Bump `@version` in the userscript header when the change should reach installed copies. Tampermonkey compares versions and ignores a push that doesn't raise it.
