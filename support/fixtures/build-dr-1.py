#!/usr/bin/env python3
"""Build the dr-1 acceptance fixture.

A dual workspace made with `vc-x1 init` and run through four cycles by a
scripted agent, its session files hand-written in the real transcript
shape, one transcript write per work line, and every relationship the
lookup command must resolve listed in the fixture's README.md and in
its relationships.json. The cycles cover the cases the lookup's probes
found: a single-step cycle, a multi-step cycle landed as a trapezoid, a
line written early and set aside so its write is in an earlier window
than its commit, a cycle-record line that moves within its file, a rung
amended after its push so a rewritten partner and a predecessor exist,
a restart so one window spans two session files, and a commit pair
made by hand with no trailer for the candidates case.

Usage: build-dr-1.py <fixtures-dir> [--local | --publish]

- <fixtures-dir>/dr-1 must not exist. The fixture is built there, and
  its two repos are pushed to the GitHub remotes the design names, or
  with --local to bare repos under <fixtures-dir>/.remotes.
- --publish takes a fixture built with --local, points both repos'
  origin at the GitHub remotes, pushes main, and removes the local
  bares, so a build and its publication can be two steps.
- VC_X1 names the vc-x1 binary to drive, default `vc-x1`.
- Commit times are pinned through JJ_CONFIG so the fixture is
  reproducible and the candidates case has a known spacing.
"""

import datetime as dt
import json
import os
import pathlib
import subprocess
import sys
import uuid

WORK_URL = "https://github.com/winksaville/vc-x1-fixtures-dr-1-work.git"
AGENT_URL = "https://github.com/winksaville/vc-x1-fixtures-dr-1-agent.git"
S1 = "1d000000-0000-4000-8000-000000000001"
S2 = "1d000000-0000-4000-8000-000000000002"
VC = os.environ.get("VC_X1", "vc-x1")
VERSION = "2.1.269"


class Clock:
    """One clock for transcript timestamps and commit times."""

    def __init__(self):
        self.t = dt.datetime(2026, 9, 12, 20, 0, 0, tzinfo=dt.timezone.utc)

    def tick(self, secs=1):
        self.t += dt.timedelta(seconds=secs)

    def iso(self):
        return self.t.strftime("%Y-%m-%dT%H:%M:%S.000Z")

    def jj(self):
        return self.t.strftime("%Y-%m-%dT%H:%M:%S+00:00")


class Jj:
    """jj and vc-x1 invocations under the fixture's own identity and
    the clock's time."""

    def __init__(self, clock, cfg_path):
        self.clock = clock
        self.cfg_path = cfg_path

    def env(self):
        self.cfg_path.write_text(
            "[user]\nname = \"dr-1 builder\"\nemail = \"dr-1@example.com\"\n"
            f"[debug]\ncommit-timestamp = \"{self.clock.jj()}\"\n"
        )
        e = dict(os.environ)
        e["JJ_CONFIG"] = str(self.cfg_path)
        e["JJ_TIMESTAMP"] = self.clock.jj()
        return e

    def run(self, args, cwd, check=True):
        p = subprocess.run(args, cwd=cwd, env=self.env(), text=True, capture_output=True)
        if check and p.returncode != 0:
            sys.exit(f"failed ({p.returncode}): {' '.join(args)}\n{p.stdout}\n{p.stderr}")
        return (p.stdout + p.stderr).strip()

    def jj(self, cwd, *args):
        return self.run(["jj", "--no-pager", *args], cwd)

    def chid(self, cwd, rev):
        return self.jj(cwd, "log", "-r", rev, "--no-graph", "-T", "change_id")

    def cid(self, cwd, rev):
        return self.jj(cwd, "log", "-r", rev, "--no-graph", "-T", "commit_id")


class Session:
    """One session file, appended in the real transcript shape."""

    def __init__(self, sid, agent_dir, cwd, clock):
        self.sid = sid
        self.path = agent_dir / f"{sid}.jsonl"
        self.cwd = str(cwd)
        self.clock = clock
        self.n = 0
        self.last = None
        self.counter = 0

    def _id(self, kind):
        self.counter += 1
        u = uuid.uuid5(uuid.NAMESPACE_URL, f"{self.sid}/{kind}/{self.counter}")
        if kind == "uuid":
            return str(u)
        return f"{kind}_{u.hex[:22]}"

    def _emit(self, d):
        self.clock.tick(1)
        d.setdefault("timestamp", self.clock.iso())
        d.update(
            sessionId=self.sid,
            cwd=self.cwd,
            version=VERSION,
            gitBranch="HEAD",
            userType="external",
            entrypoint="cli",
        )
        with open(self.path, "a") as f:
            f.write(json.dumps(d) + "\n")
        self.n += 1
        return self.n

    def other(self, typ, **kw):
        return self._emit({"type": typ, **kw})

    def user(self, text):
        u = self._id("uuid")
        n = self._emit(
            {
                "parentUuid": self.last,
                "isSidechain": False,
                "promptId": self._id("uuid"),
                "type": "user",
                "message": {"role": "user", "content": text},
                "uuid": u,
                "permissionMode": "auto",
                "origin": {"kind": "human"},
                "promptSource": "typed",
            }
        )
        self.last = u
        return n

    def _assistant(self, block, stop):
        u = self._id("uuid")
        n = self._emit(
            {
                "parentUuid": self.last,
                "isSidechain": False,
                "message": {
                    "model": "claude-fable-5-1",
                    "id": self._id("msg"),
                    "type": "message",
                    "role": "assistant",
                    "content": [block],
                    "stop_reason": stop,
                    "stop_sequence": None,
                    "usage": {"input_tokens": 1200, "output_tokens": 80},
                },
                "apiBlockIndex": 0,
                "requestId": self._id("req"),
                "type": "assistant",
                "uuid": u,
                "effort": "high",
            }
        )
        self.last = u
        return n, u

    def text(self, t):
        n, _ = self._assistant({"type": "text", "text": t}, "end_turn")
        return n

    def tool(self, name, inp):
        tid = self._id("toolu")
        n, u = self._assistant(
            {"type": "tool_use", "id": tid, "name": name, "input": inp, "caller": {"type": "direct"}},
            "tool_use",
        )
        return n, tid, u

    def result(self, tid, parent, content, tool_use_result, is_error=False):
        u = self._id("uuid")
        n = self._emit(
            {
                "parentUuid": parent,
                "isSidechain": False,
                "promptId": self._id("uuid"),
                "type": "user",
                "message": {
                    "role": "user",
                    "content": [
                        {"tool_use_id": tid, "type": "tool_result", "content": content, "is_error": is_error}
                    ],
                },
                "uuid": u,
                "toolUseResult": tool_use_result,
                "sourceToolAssistantUUID": parent,
            }
        )
        self.last = u
        return n


class Agent:
    """The scripted agent: file edits recorded as transcript writes,
    commands recorded as Bash calls, pushes recorded with their
    windows."""

    def __init__(self, work, agent, clock, jj):
        self.work = work
        self.agent = agent
        self.clock = clock
        self.j = jj
        self.session = None
        self.sessions = []
        self.marks = {}
        self.pushes = []
        self.writes = []
        self.discussion = []
        self.moved = []
        self.snapshots = []

    def start_session(self, sid):
        self.session = Session(sid, self.agent, self.work, self.clock)
        self.sessions.append(self.session)
        s = self.session
        s.other("mode", mode="auto")
        s.other(
            "attachment",
            attachment={"type": "instructions", "files": [{"path": str(self.work / "README.md"), "type": "Project"}]},
        )

    def write(self, rel, content, note):
        path = self.work / rel
        path.write_text(content)
        s = self.session
        n, tid, u = s.tool("Write", {"file_path": str(path), "content": content})
        s.result(
            tid, u, f"File created successfully at: {path}",
            {"type": "create", "filePath": str(path), "content": content, "structuredPatch": [], "originalFile": None, "userModified": False},
        )
        self.writes.append({"session": s.sid, "line": n, "tool": "Write", "file": rel, "text": note, "push": None})
        return n

    def edit(self, rel, old, new, note, moves=()):
        path = self.work / rel
        for text in moves:
            self.moved.append({"file": rel, "text": text, "push": len(self.pushes)})
        before = path.read_text()
        assert before.count(old) == 1, f"{rel}: old string not unique: {old!r}"
        path.write_text(before.replace(old, new, 1))
        s = self.session
        n, tid, u = s.tool("Edit", {"file_path": str(path), "old_string": old, "new_string": new, "replace_all": False})
        s.result(
            tid, u, f"The file {path} has been updated successfully.",
            {"filePath": str(path), "oldString": old, "newString": new, "originalFile": before, "structuredPatch": [], "userModified": False, "replaceAll": False},
        )
        self.writes.append({"session": s.sid, "line": n, "tool": "Edit", "file": rel, "text": note, "push": None})
        return n

    def bash(self, cmd, desc, out, write=None):
        s = self.session
        n, tid, u = s.tool("Bash", {"command": cmd, "description": desc})
        s.result(tid, u, out, {"stdout": out, "stderr": "", "interrupted": False, "isImage": False})
        if write:
            rel, note = write
            self.writes.append({"session": s.sid, "line": n, "tool": "Bash", "file": rel, "text": note, "push": None})
        return n

    def user(self, text):
        return self.session.user(text)

    def say(self, t, discussion=False):
        n = self.session.text(t)
        if discussion:
            self.discussion.append({"session": self.session.sid, "line": n, "push": None})
        return n

    def _snapshot(self):
        return {s.sid: s.n for s in self.sessions}

    def _files(self):
        """The work files' lines as they are at a push."""
        out = {}
        for rel in ("notes.md", "design.md", "TODO.md"):
            path = self.work / rel
            out[rel] = path.read_text().splitlines() if path.exists() else []
        self.snapshots.append(out)

    def push(self, bookmark, title, body, case):
        self.clock.tick(30)
        before = self._snapshot()
        self._files()
        self.j.run([VC, "push", bookmark, "--yes", "--title", title, "--body", body], self.work)
        work_chid = self.j.chid(self.work, bookmark)
        agent_chid = self.j.chid(self.agent, "main")
        window = []
        for sid, end in before.items():
            start = self.marks.get(sid, 0) + 1
            if end >= start:
                window.append({"session": sid, "start": start, "end": end})
            self.marks[sid] = end
        idx = len(self.pushes)
        self.pushes.append({
            "title": title, "bookmark": bookmark, "work": work_chid, "agent": agent_chid,
            "window": window, "case": case, "rewritten": False, "predecessor": False,
        })
        for w in self.writes + self.discussion:
            if w["push"] is None:
                w["push"] = idx
        cmd = f'vc-x1 push {bookmark} --title "{title}" --body "{body}"'
        self.bash(cmd, "Commit both repos and push the rung", "push: completed all stages (verified)")
        return idx

    def hand_commit(self, title, body, case):
        """A commit pair made by hand, no trailers: work first, the
        agent 20 seconds later."""
        self.clock.tick(30)
        before = self._snapshot()
        self._files()
        self.j.jj(self.work, "commit", "-m", f"{title}\n\n{body}")
        self.j.jj(self.work, "bookmark", "set", "main", "-r", "@-")
        self.j.jj(self.work, "git", "push", "--bookmark", "main")
        work_chid = self.j.chid(self.work, "main")
        self.bash(
            f'jj commit -m "{title}" && jj bookmark set main -r @- && jj git push --bookmark main',
            "Commit the work repo by hand and push main", "Changes to push to origin: bookmark: main",
        )
        self.clock.tick(20)
        self.j.jj(self.agent, "commit", "-m", f"{title}\n\n{body}")
        self.j.jj(self.agent, "bookmark", "set", "main", "-r", "@-")
        self.j.jj(self.agent, "git", "push", "--bookmark", "main")
        agent_chid = self.j.chid(self.agent, "main")
        window = []
        for sid, end in before.items():
            start = self.marks.get(sid, 0) + 1
            if end >= start:
                window.append({"session": sid, "start": start, "end": end})
            self.marks[sid] = end
        idx = len(self.pushes)
        self.pushes.append({
            "title": title, "bookmark": None, "work": work_chid, "agent": agent_chid,
            "window": window, "case": case, "rewritten": False, "predecessor": False,
        })
        for w in self.writes + self.discussion:
            if w["push"] is None:
                w["push"] = idx
        self.bash(
            f'jj commit -R .claude -m "{title}" && jj bookmark set main -r @- -R .claude && jj git push --bookmark main -R .claude',
            "Commit the agent repo by hand and push main", "Changes to push to origin: bookmark: main",
        )
        return idx

    def create_bookmark(self, name):
        self.j.jj(self.work, "git", "push", "--named", f"{name}=@-")
        self.bash(f"jj git push --named {name}=@-", "Create and publish the cycle's bookmark",
                  f"Changes to push to origin:\n  bookmark: {name} [add]")

    def land_linear(self, name):
        self.clock.tick(10)
        self.j.jj(self.work, "bookmark", "set", "main", "-r", name)
        self.j.jj(self.work, "git", "push", "--bookmark", "main")
        self.bash(f"jj bookmark set main -r {name} && jj git push --bookmark main",
                  "Fast-forward main to the cycle and publish it", "Changes to push to origin:\n  bookmark: main [move forward]")
        self._delete_bookmark(name)

    def land_trapezoid(self, name):
        self.clock.tick(10)
        self.j.jj(self.work, "rebase", "-r", name, "--onto", "main", "--onto", f"{name}-")
        self.j.jj(self.work, "new", name)
        parents = self.j.jj(self.work, "log", "-r", name, "--no-graph", "-T", "parents.map(|p| p.change_id().short(8))")
        self.bash(f"jj rebase -r {name} --onto main --onto {name}- && jj new {name}",
                  "Reshape the closing into the trapezoid merge", f"Rebased 1 commits onto destination\nParents: {parents}")
        self.j.jj(self.work, "bookmark", "set", "main", "-r", name)
        self.j.jj(self.work, "git", "push", "--bookmark", "main")
        self.bash(f"jj bookmark set main -r {name} && jj git push --bookmark main",
                  "Fast-forward main to the merge and publish it", "Changes to push to origin:\n  bookmark: main [move forward]")
        self._delete_bookmark(name)

    def _delete_bookmark(self, name):
        self.j.jj(self.work, "bookmark", "delete", name)
        self.j.jj(self.work, "git", "push", "--bookmark", name)
        self.bash(f"jj bookmark delete {name} && jj git push --bookmark {name}",
                  "Delete the landed bookmark locally and remotely", f"Changes to push to origin:\n  bookmark: {name} [delete]")


def publish(fixtures):
    """Point a locally built dr-1 at GitHub and push both mains."""
    fix = fixtures / "dr-1"
    work, agent = fix, fix / ".claude"
    for repo, url in ((work, WORK_URL), (agent, AGENT_URL)):
        subprocess.run(["git", "remote", "set-url", "origin", url], cwd=repo, check=True)
        subprocess.run(["git", "push", "-q", "origin", "main"], cwd=repo, check=True)
        subprocess.run(["jj", "--no-pager", "git", "fetch"], cwd=repo, check=True)
    for stray in (fixtures / ".jjconfig-dr-1.toml",):
        if stray.exists():
            stray.unlink()
    if (fixtures / ".remotes").is_dir():
        shutil.rmtree(fixtures / ".remotes")
    print(f"published {fix} to {WORK_URL} and {AGENT_URL}")


def main():
    args = [a for a in sys.argv[1:] if not a.startswith("--")]
    local = "--local" in sys.argv
    if len(args) != 1:
        sys.exit(__doc__)
    fixtures = pathlib.Path(args[0]).resolve()
    if "--publish" in sys.argv:
        publish(fixtures)
        return
    fix = fixtures / "dr-1"
    if fix.exists():
        sys.exit(f"{fix} exists: remove it first")
    fixtures.mkdir(parents=True, exist_ok=True)
    remotes = fixtures / ".remotes"
    remotes.mkdir(exist_ok=True)
    clock = Clock()
    jj = Jj(clock, fixtures / ".jjconfig-dr-1.toml")

    # The workspace, made as vc-x1 init makes one, with local bare
    # remotes that are re-pointed at GitHub unless --local.
    jj.run([VC, "init", str(fix), "--repo", f"local={remotes}"], fixtures, check=False)
    work, agent = fix, fix / ".claude"
    if not (agent / ".jj").is_dir():
        sys.exit("init did not produce the dual layout")
    if not local:
        subprocess.run(["git", "remote", "set-url", "origin", WORK_URL], cwd=work, check=True)
        subprocess.run(["git", "remote", "set-url", "origin", AGENT_URL], cwd=agent, check=True)
        subprocess.run(["git", "push", "-q", "origin", "main"], cwd=work, check=True)
        subprocess.run(["git", "push", "-q", "origin", "main"], cwd=agent, check=True)
    init_work = jj.chid(work, "main")
    init_agent = jj.chid(agent, "main")

    a = Agent(work, agent, clock, jj)

    # Cycle A: single-step.
    a.start_session(S1)
    a.user("add a notes file, as a single-step cycle")
    a.say("Writing notes.md and pushing it as the cycle's one commit.", discussion=True)
    a.create_bookmark("add-the-notes-file")
    a.write("notes.md", "# Notes\n\nA first note.\n", "A first note.")
    pA = a.push("add-the-notes-file", "docs: add the notes file",
                "The workspace has no notes file, so there is nowhere to put a\nnote.\n\n* A note needs a file.\n  - notes.md holds the first one.",
                "single-step")
    a.land_linear("add-the-notes-file")

    # Cycle B: multi-step, a line set aside at the opening, landed as a
    # trapezoid.
    clock.tick(600)
    a.user("open the design note cycle, multi-step")
    a.say("Opening: the In Progress block, and an early line for notes.md that I will set aside.", discussion=True)
    a.create_bookmark("the-design-note")
    a.write("TODO.md",
            "# Todo\n\n## In Progress\n\n### feat: the design note\n\nA design note, written over two rungs.\n\n## Todo\n\n### docs: a glossary\n\nThe terms the notes use.\n\n### test: a second fixture\n\nOne work repo with two agent repos.\n\n## Closed\n\n_None._\n",
            "A design note, written over two rungs.")
    a.edit("notes.md", "A first note.\n", "A first note.\nA note written early and set aside.\n",
           "A note written early and set aside.")
    a.say("Setting the early line aside so the opening carries only its own change.")
    patch = jj.jj(work, "diff", "--git", "notes.md")
    (fixtures / ".early.patch").write_text(patch + "\n")
    jj.jj(work, "restore", "notes.md")
    a.bash("jj diff --git notes.md > /tmp/dr-1-early.patch && jj restore notes.md",
           "Set the early notes.md line aside as a patch", "")
    pB0 = a.push("the-design-note", "feat: the design note opening",
                 "Opens the cycle \"feat: the design note\". Its record is the In\nProgress block in TODO.md.",
                 "opening")
    a.say("Writing the design note.")
    a.write("design.md", "# Design\n\nThe first decision.\n", "The first decision.")
    pB1 = a.push("the-design-note", "feat: write the design note",
                 "The design has no note, so the first decision is unwritten.\n\n* A decision needs a note.\n  - design.md holds the first decision.",
                 "rung")
    a.say("Restoring the early line, then the second decision.")
    subprocess.run(["git", "apply", str(fixtures / ".early.patch")], cwd=work, check=True)
    a.bash("git apply /tmp/dr-1-early.patch", "Restore the early notes.md line from its patch", "")
    a.edit("design.md", "The first decision.\n", "The first decision.\nThe second decision.\n", "The second decision.")
    pB2 = a.push("the-design-note", "feat: the second decision",
                 "The note holds one decision and the early note is still set\naside.\n\n* The second decision is unwritten.\n  - design.md gains it.\n* The early line is in a patch, not the file.\n  - notes.md takes it back.",
                 "rung")
    a.say("Closing: the record moves to Closed.")
    a.edit("TODO.md",
           "## In Progress\n\n### feat: the design note\n\nA design note, written over two rungs.\n\n## Todo\n\n### docs: a glossary\n\nThe terms the notes use.\n\n### test: a second fixture\n\nOne work repo with two agent repos.\n\n## Closed\n\n_None._\n",
           "## In Progress\n\n_No cycle currently in progress._\n\n## Todo\n\n### docs: a glossary\n\nThe terms the notes use.\n\n### test: a second fixture\n\nOne work repo with two agent repos.\n\n## Closed\n\n### feat: the design note\n\nA design note, written over two rungs.\n\n- Result: pass, both decisions are in design.md.\n",
           "- Result: pass, both decisions are in design.md.",
           moves=["A design note, written over two rungs."])
    pB3 = a.push("the-design-note", "feat: the design note closing",
                 "Closes the cycle \"feat: the design note\". Its record is the In\nProgress block in TODO.md.",
                 "closing")
    a.land_trapezoid("the-design-note")
    a.pushes[pB3]["rewritten"] = True
    a.say("Landed as a trapezoid. Restarting for the tool update.")

    # Cycle C: a restart, then a rung amended after its push, landed
    # keep separate.
    clock.tick(600)
    a.start_session(S2)
    a.user("acquaint")
    a.say("Continuation notes read. Opening fix: the design note.", discussion=True)
    a.create_bookmark("fix-the-design-note")
    a.edit("TODO.md",
           "## In Progress\n\n_No cycle currently in progress._\n\n## Todo\n\n### docs: a glossary\n\nThe terms the notes use.\n\n### test: a second fixture\n\nOne work repo with two agent repos.\n\n## Closed\n\n### feat: the design note\n\nA design note, written over two rungs.\n\n- Result: pass, both decisions are in design.md.\n",
           "## In Progress\n\n### fix: the design note\n\nThe first decision needs revising.\n\n## Todo\n\n### docs: a glossary\n\nThe terms the notes use.\n\n### test: a second fixture\n\nOne work repo with two agent repos.\n\n## Closed\n\n_None._\n",
           "The first decision needs revising.")
    pC0 = a.push("fix-the-design-note", "fix: the design note opening",
                 "Opens the cycle \"fix: the design note\". Its record is the In\nProgress block in TODO.md.",
                 "opening")
    a.say("Revising the first decision.")
    a.edit("design.md", "The first decision.\n", "The first decision, revised.\n", "The first decision, revised.")
    pC1 = a.push("fix-the-design-note", "fix: revise the first decision",
                 "The first decision was made before the second and no longer\nreads with it.\n\n* The first decision is stale.\n  - design.md revises it.",
                 "rung")
    a.say("The revision misses the check it was made for. Amending the rung.")
    a.edit("design.md", "The first decision, revised.\n", "The first decision, revised and checked.\n",
           "The first decision, revised and checked.")
    clock.tick(30)
    jj.jj(work, "squash")
    jj.jj(work, "git", "push", "--bookmark", "fix-the-design-note")
    a.snapshots.pop()
    a._files()

    a.bash("jj squash && jj git push --bookmark fix-the-design-note",
           "Fold the fix into the rung and re-push the bookmark", "Changes to push to origin:\n  bookmark: fix-the-design-note [move sideways]")
    a.pushes[pC1]["rewritten"] = True
    clock.tick(10)
    jj.run([VC, "squash-push", "-R", str(agent)], work)
    a.pushes[pC1]["predecessor"] = True
    # The amend's lines are now in the partner's amended diff, so the
    # window's end moves to the squash-push.
    for w in a.pushes[pC1]["window"]:
        if w["session"] == S2:
            w["end"] = a.session.n
    a.marks[S2] = a.session.n
    for w in a.writes + a.discussion:
        if w["push"] is None:
            w["push"] = pC1
    a.bash("vc-x1 squash-push -R .claude", "Fold the session tail into the rung's partner",
           "squash-push: done")
    a.say("Closing.")
    a.edit("TODO.md",
           "## In Progress\n\n### fix: the design note\n\nThe first decision needs revising.\n\n## Todo\n\n### docs: a glossary\n\nThe terms the notes use.\n\n### test: a second fixture\n\nOne work repo with two agent repos.\n\n## Closed\n\n_None._\n",
           "## In Progress\n\n_No cycle currently in progress._\n\n## Todo\n\n### docs: a glossary\n\nThe terms the notes use.\n\n### test: a second fixture\n\nOne work repo with two agent repos.\n\n## Closed\n\n### fix: the design note\n\nThe first decision needs revising.\n\n- Result: pass, the first decision is revised and checked.\n",
           "- Result: pass, the first decision is revised and checked.",
           moves=["The first decision needs revising."])
    pC2 = a.push("fix-the-design-note", "fix: the design note closing",
                 "Closes the cycle \"fix: the design note\". Its record is the In\nProgress block in TODO.md.",
                 "closing")
    a.land_linear("fix-the-design-note")

    # D: a commit pair made by hand, no trailers.
    clock.tick(600)
    a.user("append a line to notes.md and commit it by hand, no push command")
    a.say("Appending the line with a heredoc and committing both repos by hand.", discussion=True)
    with open(work / "notes.md", "a") as f:
        f.write("A line committed by hand.\n")
    a.bash("cat >> notes.md <<'EOF'\nA line committed by hand.\nEOF", "Append a line to notes.md", "",
           write=("notes.md", "A line committed by hand."))
    pD = a.hand_commit("chore: a line committed by hand", "Made by hand, so no ochid trailer on either side.", "no-trailer")

    # The record: README.md and relationships.json, pushed as a
    # single-step cycle after everything they describe.
    clock.tick(600)
    rel = describe(a, work, init_work, init_agent)
    (work / "relationships.json").write_text(json.dumps(rel, indent=2) + "\n")
    (work / "README.md").write_text(readme(rel))
    a.user("describe the fixture")
    a.say("Writing the README and relationships.json from the build's record.")
    a.create_bookmark("describe-the-fixture")
    a.bash("python3 support/fixtures/build-dr-1.py ../vc-x1-fixtures", "Write README.md and relationships.json", "")
    a.push("describe-the-fixture", "docs: describe the fixture",
           "The fixture's relationships live only in the build script's\nmemory.\n\n* A reader and a test need them in the tree.\n  - README.md lists them and relationships.json carries them.",
           "single-step")
    a.land_linear("describe-the-fixture")
    jj.run([VC, "squash-push", "-R", str(agent)], work)
    (fixtures / ".early.patch").unlink()
    if not local:
        jj.cfg_path.unlink()
        shutil.rmtree(remotes)
    print(f"built {fix}")
    print(json.dumps(rel["pushes"], indent=1))


def jj_windows(a):
    """Each push's window as jj records it: for every session file the
    agent commit changed, the lines past its parent's count. The build's
    own counters miss what a later squash-push folds into a partner, so
    the committed trees are the record."""
    agent = a.agent
    for p in a.pushes:
        window = []
        for sess in a.sessions:
            name = f"{sess.sid}.jsonl"
            def count(rev):
                out = subprocess.run(
                    ["jj", "--no-pager", "file", "show", "-r", rev, name],
                    cwd=agent, env=a.j.env(), text=True, capture_output=True)
                return len(out.stdout.splitlines()) if out.returncode == 0 else 0
            before, after = count(p["agent"] + "-"), count(p["agent"])
            if after > before:
                window.append({"session": sess.sid, "start": before + 1, "end": after})
        p["window"] = window
    def owner(sid, line):
        for i, p in enumerate(a.pushes):
            for w in p["window"]:
                if w["session"] == sid and w["start"] <= line <= w["end"]:
                    return i
        return None
    for w in a.writes + a.discussion:
        w["push"] = owner(w["session"], w["line"])


def describe(a, work, init_work, init_agent):
    """The relationships as JSON: the pushes with their windows, the
    writes with the work line each produced, and the discussion lines."""
    jj_windows(a)
    final = {}
    for rel in ("notes.md", "design.md", "TODO.md"):
        final[rel] = (work / rel).read_text().splitlines()
    writes = []
    for w in a.writes:
        p = a.pushes[w["push"]]
        lines = final[w["file"]]
        line = lines.index(w["text"]) + 1 if w["text"] in lines else None
        arrives = None
        for i, snap in enumerate(a.snapshots):
            present = w["text"] in snap.get(w["file"], [])
            before = i > 0 and w["text"] in a.snapshots[i - 1].get(w["file"], [])
            if present and not before:
                arrives = i
        moved_at = next((m["push"] for m in a.moved if m["file"] == w["file"] and m["text"] == w["text"]), None)
        writes.append({
            "file": w["file"], "text": w["text"], "line_on_main": line,
            "work": p["work"], "agent": p["agent"], "push": w["push"],
            "arrives": arrives, "moved_at": moved_at,
            "write": {"session": w["session"], "line": w["line"], "tool": w["tool"]},
        })
    blamed = {}
    for rel in ("notes.md", "design.md", "TODO.md"):
        out = a.j.jj(work, "file", "annotate", "-r", "main", "-T",
                     'commit.change_id() ++ "\\n"', rel)
        blamed[rel] = out.splitlines()
    for w in writes:
        if w["line_on_main"]:
            w["blamed_on_main"] = blamed[w["file"]][w["line_on_main"] - 1]
    return {
        "fixture": "dr-1",
        "sessions": {"s1": S1, "s2": S2},
        "init": {"work": init_work, "agent": init_agent},
        "pushes": a.pushes,
        "writes": writes,
        "discussion": [{"session": d["session"], "line": d["line"], "push": d["push"]} for d in a.discussion],
    }


def readme(rel):
    """The fixture's README: what it is, the cycles, and every
    relationship as a table."""
    out = ["# dr-1: a dual workspace with known partners\n"]
    out.append(
        "The first vc-x1 acceptance fixture, `dr` for dual repo: this work repo and its agent repo at\n"
        "`.claude`, laid out as `vc-x1 clone` lays a dual workspace out, run through four cycles by a\n"
        "scripted agent so every partner relationship is known. `dr-1` is the simple one-to-one shape,\n"
        "one work repo to one agent repo. A one-work-to-many-agents fixture is a numbered sibling.\n"
        "Built by `support/fixtures/build-dr-1.py` in the vc-x1 repo, and `relationships.json` beside\n"
        "this file carries the same facts for the tests.\n"
    )
    out.append("Change ids are jj's. A partner is the commit the `ochid:` trailer names on the other side.\n")
    out.append("## Pushes\n")
    out.append("| push | title | case | work change id | agent change id | window | rewritten | predecessor |")
    out.append("|---:|:---|:---|:---|:---|:---|:---:|:---:|")
    for i, p in enumerate(rel["pushes"]):
        win = ", ".join(f"{w['session'][-4:]}:{w['start']}-{w['end']}" for w in p["window"])
        out.append(f"| {i} | {p['title']} | {p['case']} | `{p['work'][:12]}` | `{p['agent'][:12]}` | {win} | {'yes' if p['rewritten'] else ''} | {'yes' if p['predecessor'] else ''} |")
    out.append("\nThe window is the session file, by its last four characters, and the lines the agent commit added,\n"
               "read from the agent commits with jj. This file is written before its own push, so that push is\n"
               "not in the table, and `relationships.json` is the same record.\n")
    out.append("## Work lines and their transcript writes\n")
    out.append("| file | line on main | text | write | in push | arrives | moved at |")
    out.append("|:---|---:|:---|:---|---:|---:|---:|")
    for w in rel["writes"]:
        line = w["line_on_main"] if w["line_on_main"] else "gone"
        moved = w["moved_at"] if w["moved_at"] is not None else ""
        out.append(f"| {w['file']} | {line} | {w['text']} | {w['write']['session'][-4:]}:{w['write']['line']} {w['write']['tool']} | {w['push']} | {w['arrives']} | {moved} |")
    out.append(
        "\n`gone` marks a line no longer on main: a cycle-record line the next opening deleted, so it is\n"
        "in the landmark's tree only, or a line a later edit replaced. `in push` is the push whose window\n"
        "holds the write, `arrives` the push whose commit first carries the line, and `moved at` the\n"
        "closing whose diff removed it from In Progress and added it to Closed, past the Todo entries.\n"
    )
    out.append("## Discussion lines\n")
    out.append("| session:line | push |")
    out.append("|:---|---:|")
    for d in rel["discussion"]:
        out.append(f"| {d['session'][-4:]}:{d['line']} | {d['push']} |")
    out.append("\n## The cases\n")
    out.append(
        "- Single-step: push 0, one commit with the bare cycle title, landed by a fast-forward.\n"
        "- Trapezoid: pushes 1 to 4, landed as a merge whose first parent is the trunk, so the closing\n"
        "  is rewritten and its change id is what the trailer still finds.\n"
        "- Set aside: the notes.md line written at the opening, restored from a patch at the second\n"
        "  rung, so its transcript write is in push 1's window and its commit is push 3's.\n"
        "- Moved: the cycle-record lines move from In Progress to Closed at a closing, so blame gives\n"
        "  the closing and the write is in the opening's window.\n"
        "- Amended: push 6's rung was squashed into after its push and the partner squash-pushed, so\n"
        "  both sides have a predecessor and the amended line's write is in the amended partner.\n"
        "- Restart: push 5's window spans both session files.\n"
        "- No trailer: push 8 is a pair made by hand, so a lookup names candidates by time.\n"
    )
    return "\n".join(out) + "\n"


if __name__ == "__main__":
    main()
