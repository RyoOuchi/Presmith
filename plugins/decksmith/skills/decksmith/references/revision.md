# Targeted revisions

1. Identify stable slide and element IDs from deck.json and source. Read the whole
   requested slide, including its notes and scoped CSS/JS.
2. Inspect git status/diff before editing; preserve unrelated user changes. If there
   is no Git history, record hashes of unrelated slide source files.
3. Modify the smallest source scope. Retain IDs, facts, notes and slide order unless
   the request changes them. Prefer [data-slide-id="ID"] selectors for local styles.
4. Inspect the diff and verify unrelated source hashes or Git diffs are unchanged.
5. Check and render --slide ID. Open the image at full size. If shared resources
   changed, check/render the entire deck and inspect the contact sheet and slides.
6. Repair up to three passes, then state any remaining issue precisely. Export when
   requested; retain the generated artifact paths and diagnostic result.

Example request: shorten the headline on solution without changing other slides.
Do not rewrite the narrative, switch themes, rename IDs or reformat all files.
