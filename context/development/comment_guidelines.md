# Comment Guidelines

**A comment says what the code does, in the present tense, and only when the code does not say it itself.**

The code explains itself. A comment covers the part it cannot: the surprise, the invariant, the unit.

## Do not write

- **The history.** No "previously", "used to", "now", "no longer". Git has the history.
- **The situation.** No "this is needed because we ran into". If the code only makes sense with the story attached, the code is wrong.
- **The sermon.** No "without this, X would happen". Say what the line does, and give the consequence one short clause if it's the part that isn't obvious.
- **A restatement.** A comment that repeats the function name is noise.

## Do write

- What a reader can rely on: the order of `KEYS`, which style element a theme change touches.
- The surprise: a capture-phase listener, a swap that reads `current` instead of its argument.
- Which zwipe rule a style copies, when the name doesn't make it obvious.

## Form

- One line when it can be.
- Section banners are one line: `// == the picker ==`.
- No em dashes, in code or in prose. No spaced hyphen joining two fragments either.
- American spelling.

## Checking

```bash
grep -nE 'previously|used to|no longer|without this|in order to|—' *.user.js scripts/*.mjs
```
