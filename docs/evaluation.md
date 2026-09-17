# A small evaluation exercise

This is an evaluation protocol, not a claim that Decksmith improves productivity.
No comparative results have been collected.

Use two conditions with the same agent/model version, tool access, time limit and
source brief. Condition A: ask the agent for an eight-slide HTML deck without this
toolkit. Condition B: provide Decksmith and its skill. Supply identical factual
notes, audience (technical product team), purpose (choose a prototype direction),
style constraints and an explicit ban on fabricated statistics. Start fresh tasks.
Alternate condition order across at least three paired runs to reduce practice bias.
Keep installation time separate from authoring time and report both.

Define “usable” before running: complete narrative, supplied facts preserved,
readable at presentation size, no visible clipping/broken assets, working keyboard
navigation, eight-page PDF and portable HTML. Have a reviewer inspect anonymized
exports using the same rubric. Do not use Decksmith's own checker as the only judge.

After the first usable deck, issue the same localized revision: “Shorten the headline
on the solution slide to at most seven words. Preserve its evidence and all other
slides.” Snapshot Git/hashes before and after. Count unwanted changed slides and
check whether the requested edit is accurate. Inspect source and rendered results.

Record one row per run using this unfilled structure:

| Run | Condition | Setup min | Min to usable | Repair prompts | Manual edits | Revision accurate? | Unrelated slides changed | Remaining defects |
|---|---|---|---|---|---|---|---|---|

Correction effort = number of repair prompts plus separately reported manual edits
and minutes; do not collapse these into a made-up weighted score. Revision accuracy
= requested constraints met and unrelated source/visual content preserved. Time to
usable output = start of authoring through reviewer acceptance. Report medians and
ranges, exact sample size, failures and hardware/font differences. Save prompts,
agent transcripts, diffs and artifacts. Do not infer statistical significance from
a small convenience sample.
