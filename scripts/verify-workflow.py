#!/usr/bin/env python3
"""Run the documented fresh-project workflow; installs only when explicitly run."""
import hashlib
import json
from pathlib import Path
import re
import signal
import subprocess
import sys
import time
import urllib.request

repo = Path(__file__).resolve().parents[1]
binary = repo / 'target/debug/decksmith'
project = Path(sys.argv[1]).resolve() if len(sys.argv) > 1 else repo / 'verification/workflow talk'
records = []

def run(*args):
    command = [str(binary), *map(str, args)]
    print('+', ' '.join(command), flush=True)
    completed = subprocess.run(command, cwd=project.parent, capture_output=True, text=True, timeout=600)
    if completed.stderr:
        print(completed.stderr, file=sys.stderr, end='')
    record = {'command': command, 'exit_code': completed.returncode}
    if '--json' in command:
        record['result'] = json.loads(completed.stdout)
    records.append(record)
    if completed.returncode:
        raise RuntimeError(f'Failed: {command}\n{completed.stdout}')
    return record

project.parent.mkdir(parents=True, exist_ok=True)
run('init', project)
run('setup', project)
run('doctor', project, '--json')
preview = subprocess.Popen([str(binary), 'dev', str(project), '--port', '0'], stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
try:
    line = preview.stderr.readline()
    match = re.search(r'http://127\.0\.0\.1:\d+/', line)
    if not match:
        raise RuntimeError(f'Preview failed: {line}')
    url = match[0]
    with urllib.request.urlopen(url, timeout=5) as response:
        html = response.read().decode()
        assert 'data-slide-id="intro"' in html
    records.append({'command':['decksmith','dev',str(project),'--port','0'],'url':url,'http_status':200})
finally:
    preview.send_signal(signal.SIGTERM)
    preview.wait(timeout=10)
run('check', project, '--json')
run('render', project, '--json')
run('export', project, '--format', 'html', '--json')
run('export', project, '--format', 'pdf', '--json')

def hashes():
    return {p.name: hashlib.sha256(p.read_bytes()).hexdigest() for p in sorted((project/'slides').glob('*.html'))}

before = hashes()
slide = project/'slides/workflow.html'
slide.write_text(slide.read_text().replace('Three moves. One clear message.', 'One story. Three deliberate moves.'))
after = hashes()
assert before['intro.html'] == after['intro.html']
assert before['next.html'] == after['next.html']
assert before['workflow.html'] != after['workflow.html']
run('check', project, '--slide', 'workflow', '--json')
run('render', project, '--slide', 'workflow', '--out', project/'.decksmith/revision', '--json')
records.append({'targeted_revision': {'slide_id':'workflow','before':before,'after':after,'unrelated_sources_unchanged':True}})
report = repo/'verification/fresh-workflow.json'
report.parent.mkdir(exist_ok=True)
report.write_text(json.dumps({'success':True,'project':str(project),'records':records}, indent=2)+'\n')
print(f'Verified fresh workflow. Evidence: {report}')
