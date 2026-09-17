# Findings

Errors: manifest.* (JSON/IDs/paths/files); element.duplicate_id;
asset.request_failed; asset.broken_image; runtime.javascript; runtime.not_ready;
layout.out_of_bounds; text.overflow; text.clipped.
Warning: text.too_small below 18 logical pixels (library folios exempt).

Findings identify slide_id, element_id, source and measurements when available.
For overflow use the logical 1280×720 canvas, with a 2px rounding tolerance. Fix
content/space first. Do not blindly suppress findings. External browser requests
are blocked, so use local assets. Readiness hooks time out after 10 seconds.

Geometry checks do not understand all SVG internals, canvas pixels, masks,
pseudo-elements, complex text flow or visual occlusion. Intentional overlap is
valid and is not reported. The checker is not a narrative, contrast or fact checker.
Use image inspection to resolve ambiguity; explain legitimate remaining findings.
