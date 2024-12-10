# BPMN DSL Documentation

This README provides an overview of the Domain-Specific Language (DSL) used for defining BPMN flows. The syntax enables the creation of tasks, events, and branching processes with labeled joins.

## Introduction

This DSL allows users to define business processes using a simplified syntax. The flow structure consists of tasks, start/middle events, end events, and branching/merging points. It supports labeled branches and joins, enabling complex workflows to be modeled concisely.

## DSL Syntax Overview

### Symbols & Meanings

- **`=`** : Represents a **pool** in the flow.
  - Example:
    `= Pool` – Defines a pool called `Pool`.

- **`==`** : Represents a **lane** in the flow.
  - Example:
    `== Lane` – Defines a lane called `Lane`.

- **`#`** : Denotes a **start event** or **middle event** in the process.
  - Example:
    `# StartEvent` – Defines the start of the process called `StartEvent`.
    `# MiddleEvent` – Defines a middle event called `MiddleEvent`.

- **`-`** : Represents a **task** in the flow.
  - Example:
    `- TaskName` – Defines a task called `TaskName`.

- **`.`** : Indicates an **end event**, signaling the completion of the process.
  - Example:
    `. EndEvent` – Marks the end of the process.

- **`X ->label`** : Declares a **(Diverging) Exclusive Gateway**, which is a branching point, with each branch labeled after `->`. You can also add optional text for the edge enclosed in quotes.
  - Example:
    `X ->Branch "Optional text"`

- **`O ->label`** : Declares a **(Diverging) Inclusive Gateway**, which is a branching point, with each branch labeled after `->`. You can also add optional text for the edge enclosed in quotes.
  - Example:
    `O ->Branch "Optional text"`

- **`+ ->label`** : Declares a **(Diverging) Parallel Gateway**, which is a branching point, with each branch labeled after `->`. You can also add optional text for the edge enclosed in quotes.
  - Example:
    `+ ->Branch "Optional text"`

- **`* ->label`** : Declares a **(Diverging) Event Gateway**, which is a branching point, with each branch labeled after `->`. You can also add optional text for the edge enclosed in quotes.
  - Example:
    `* ->Branch "Optional text"`

- **`X <-label`** : Declares a **(Converging) Exclusive Gateway**, which is used to indicate to which label should the last node join to the converging exclusive gateway.
  - Example:
    `X <-endLabel`

- **`O <-label`** : Declares a **(Converging) Inclusive Gateway**, which is used to indicate to which label should the last node join to the converging inclusive gateway.
  - Example:
    `O <-endLabel`

- **`+ <-label`** : Declares a **(Converging) Parallel Gateway**, which is used to indicate to which label should the last node join to the converging parallel gateway.
  - Example:
    `+ <-endLabel`

- **`* <-label`** : Declares a **(Converging) Event Gateway**, which is used to indicate to which label should the last node join to the converging event gateway.
  - Example:
    `* <-endLabel`

- **`label:`** : Declares a **label**, which is defined by ending with a colon `:`. The label has to include at least one node and a join operator. Labels are used to define a branch.
  - Example:
  `Branch1:`
  `- Task1`
  `- Task2`
  `J endLabel`

- **`J label`** : Marks a **join operator**, indicating where a branch should merge. You must specify a label. If a join is not wanted, give it a join label that is not used anywhere.
  - Example:
    `J endLabel`

- **`G ->label`** : Declares a **Go (from) operator**, which is used to indicate from which node should a edge start. The edge starts from the previous node so you cannot use it before defining a node beforehand. Also you must define a label for the `G` operator.
  - Example:
    `- Start node`
    `G ->jump`

- **`G <-label`** : Declares a **Go (to) operator**, which is used to indicate to which node should a edge end. The edge ends to the next node you define so you cannot use it for the final line. Also you must define a label for the `G` operator.
  - Example:
    `G <-jump`
    `- End node`

### Branching Example

```plaintext
# Start Event
X ->above"Go Here" ->below"No, here!"

above:
- Above
J endjoin

below:
- And beyond
J endjoin

X <-endjoin
. End Event
```

### Pools & Lanes Example

```plaintext
= Pool
== Lane1
# Start Event
- Task1
. End Event
== Lane2
# Start Event2
- Task2
. End Event 2
```

### Go operator example

```plaintext
= Pool
== Lane1
# Start Event
- Task
G ->jump
== Lane2
G <-jump
- Task
. End Event
```


# Dependencies
To convert BPMN diagrams to images, you need to install the `bpmn-to-image` tool.

### Installation

To install `bpmn-to-image`, you can use npm:

```sh
npm install bpmn-to-image
```

### Usage
To use `bpmn-to-image`, you need to provide the input file and specify the output format (pdf, svg, or png) as the second argument.

Example:
```sh
bpmn-parser input.txt png
```
