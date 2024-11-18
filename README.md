# BPMN DSL Documentation

This README provides an overview of the Domain-Specific Language (DSL) used for defining BPMN flows. The syntax enables the creation of tasks, events, and branching processes with labeled joins.

## Introduction

This DSL allows users to define business processes using a simplified syntax. The flow structure consists of tasks, start/middle events, end events, and branching/merging points. It supports labeled branches and joins, enabling complex workflows to be modeled concisely.

## DSL Syntax Overview

### Symbols & Meanings

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

- **`X`** : Declares a **(Diverging) Exclusive Gateway**, which is a branching point, with each branch labeled after `->`. You can also add optional text for the edge enclosed in quotes.
  - Example:  
    `X ->Branch2 "Optional text"`

- **`label:`** : Declares a **label**, which is defined by ending with a colon `:`. The label has to include at least one node and a join operator. Labels are used to define a branch.
  - Example:  
  `Branch1:`  
  `- Task1`  
  `- Task2`  
  `J endLabel`

- **`J label`** : Marks a **join operator**, indicating where a branch should merge. It must specify a label. If a join is not wanted, give it a join label that is not used anywhere.
  - Example:  
    `J endLabel`

- **`X <-label`** : Declares a **(Converging) Exclusive Gateway**, which is used to indicate to which label should the last node join to the converging exclusive gateway.
  - Example:  
    `X <-endLabel`

### Flow Example

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
