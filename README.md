# BPMN DSL - BPMD

BPMD is a language and its compiler to create `.xml` BPMN diagrams from human-writable text files.
The goal is to that the creator can concentrate mostly on the semantics of the business process, whereas the tool computes at what location each component and end bendpoint should be located.
*This project is in a PoC stage and is missing most features which are required for serious usage.*

```
= This is a Pool
== This is a Lane

# Start Event
X I am an exclusive Gateway ->one-branch"Let's Go here" ->the-other-branch"No, here!"

G <-one-branch
- Wonderful Task
G ->we-are-done

G <-the-other-branch
- Inspiring Task
G ->we-are-done

X <-we-are-done
. Have Some Rest
```

## Why could this be useful?

This tool tries to fall into the same niche as PlantUML and similar tools. The target group is (probably) primarily users which are home in text files. A couple of (subjective) benefits:

* Author can focus on content rather than Layout - an okayish layout is calculated automatically
* Later changes to the process, like adding a task activity somewhere in the middle, don't require a full (manual) rework of the surrounding layout.
* Better integration with version control systems:
  * The syntax aims to split the process description from layout tweaks. Meaning that a diffs are more meaningful (one can rather concentrate on the changes of the process than on layout peculiarities)
* Any advantage which text files bring over binary files (like one can stay within one's beloved text editor / IDE to code, document and create diagrams without mouse interruptions)

## DSL Syntax Overview

The syntax is explained in [doc/doc.html](doc/doc.html).

## Dependencies

* TODO: Write about the cbc solver dependency. Also try to replace this with a pure Rust solver at some point.
* `bpmn-to-image`: The compiler within this repository only creates the `.xml` representation. If you want a `.png` file, you can use `bpmn-to-image` for that. In the `./doc` directory, run `npm install` and look into the `doc/build.sh` script to see how to use it.

## Usage

Use `cargo run --release -- --help` to see how to use the tool.

Basic usage: `cargo run --release -- -i file.bpmd -o file.bpmn`
