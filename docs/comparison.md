# Related Blame Tools

## Perforce Time Lapse View

This GUI is bundled with the P4V client for Perforce SCM

Blame View
- Shows when hunks are last and next modified
- By default stays within the current branch but can follow back to previous branches
- Colors hunk by age
- Visualizer for how long hunks have existed in file

Incremental Diff View
- Shows something like `git diff HEAD~..HEAD` but can slide backwards

Diff View
- Shows diff between two specified commits with slides for each commit

See
- [Documentation](https://www.perforce.com/manuals/p4v/Content/P4V/advanced_files.timelapse.html)
- [Video: Tutorial](https://www.perforce.com/video-tutorials/vcs/using-time-lapse-view)

## `git blame`

A CLI for showing a file with line numbers and annotations for when a change was introduced
- Supports ignoring specific revisions via [blame.ignoreRevsFile](https://git-scm.com/docs/git-config#Documentation/git-config.txt-blameignoreRevsFile)
- To investigate the commit, requires dropping out and doing `git show <sha>`
- If the commit isn't of interest, requires running `git blame <sha>~ <path>` which will fail if the file had moved

## git "pickaxe"

A CLI that shows when a change was introduced or remove so long as your search is correct

See
- [Why Git blame sucks for understanding WTF code (and what to use instead)](https://tekin.co.uk/2020/11/patterns-for-searching-git-revision-histories?utm_source=Reddit)

## DeepGit

A blame GUI that let's you click through to look further back then a commit.

Includes heuristics for detecting moves of code, showing level of confidence.

See [DeepGit](https://www.syntevo.com/deepgit/)

## Git Tower

A git GUI that includes `git blame` support

Let's you click to get info about a commit but doesn't seem to have any other investigation features.

See [Git Tower: Blame Window](https://www.git-tower.com/help/guides/commit-history/blame/windows)

## CLion

An IDE that includes a GUI for `git blame`

- Colors commits differently
- Syntax highlighting
- Click on commit to see `git show` and to jump to the commit
- Can jump to a previous revision
- Can hide revisions
  - A GUI hint that this happens
  - Can be turned on/off

See [CLion: "Locate Code Author"](https://www.jetbrains.com/help/clion/investigate-changes.html#annotate_blame)

## Tig

A git TUI that includes a `git blame` view

No details on website to see how powerful it is

See [Tig](https://jonas.github.io/tig/doc/manual.html)

## git spelunk

A TUI for showing a file with line numbers and annotations for when a change was introduced
- When a hunk is selected, it highlights all related hunks
- `[` and `]` to move through history
- Has a `git show` pane
- `less`-like searching through the file

See [git spelunk](https://github.com/osheroff/git-spelunk)

## git word-blame

A tool to show the commit for each word

See [git word-blame](https://framagit.org/mdamien/git-word-blame/)
