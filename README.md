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

- **`X`** : Declares a **branching point**, with each branch labeled after `->`. You can also add optional text enclosed in quotes.
  - Example:  
    `X ->Branch1`  
    `X ->Branch2 "Optional text"`

- **`J label`** : Marks a **join point**, indicating where a branch should merge. It must specify a label.
  - Example:  
    `J JoinPoint`

- **`<- label`** : Used to **mark and join nodes**, indicating where a flow should join back to a label.
  - Example:  
    `<- JoinPoint`

### Flow Example

```plaintext
# Start Event
X ->above "Go here!"
->below "No, here!"

above:
- Above
J endjoin

below:
- And beyond
J endjoin

<-endjoin
. Finish
