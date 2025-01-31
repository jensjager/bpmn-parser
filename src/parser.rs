use crate::common::bpmn_event::BpmnEvent;
use crate::common::graph::DataNodeId;
use crate::common::graph::Graph;
use crate::common::graph::NodeId;
use crate::lexer;
use crate::lexer::lex;
use crate::lexer::DataKind;
use crate::lexer::DataMeta;
use crate::lexer::EdgeMeta;
use crate::lexer::EventMeta;
use crate::lexer::GatewayInnerMeta;
use crate::lexer::GatewayNodeMeta;
use crate::lexer::NodeMeta;
use crate::lexer::SequenceFlowMeta;
use crate::lexer::{Statement, StatementStream, TokenCoordinate};
use std::collections::HashMap;
use std::dbg;

use annotate_snippets::renderer::Renderer;
use annotate_snippets::Level;
use annotate_snippets::Snippet;

struct ParseContext {
    last_node_id: Option<usize>,
    current_pool: Option<String>,
    current_lane: Option<String>,
    lifeline_state: LifelineState,
    current_token_coordinate: TokenCoordinate,
    branching: ParseBranching,
}

// X ->l1 ; l1: - task; J end; X<-end
#[derive(Debug)]
enum LifelineState {
    /// At the beginning of the text file, a pool or lane, and after a terminal
    /// node.
    NoLifelineActive {
        // None if it is the beginning of the text file.
        previous_lifeline_termination_statement: Option<TokenCoordinate>,
    },
    /// When a branch target was encountered.
    StartedFromBranch {
        edge_type: EdgeType,
        name: String,
        token_coordinate: TokenCoordinate,
    },
    /// During a chain of tasks/events.
    ActiveLifeline {
        last_node_id: NodeId,
        token_coordinate: TokenCoordinate,
    },
}

#[derive(Debug)]
struct DanglingEdgeInfo {
    known_node_id: NodeId,
    edge_type: EdgeType,
    // Might be empty
    edge_text: String,
    tc: TokenCoordinate,
}

#[derive(Default)]
struct ParseBranching {
    // TODO maybe different maps are better. Let's see.
    /// An edge whose start is known but end is unknown.
    /// With gateways, the merge node has one label and is targeted by many incoming lifelines.
    /// Hence a Vec. I.e. there can be many `G ->label` nodes for one `X <-label` node.
    dangling_end_map: HashMap<String, Vec<DanglingEdgeInfo>>,
    /// An edge whose end is known but start is unknown. There is always a 1:1 relation here, so no
    /// Vec. We require that `G <-label` uses a unique label, so when looking at the `X ->label`
    /// node we know exactly how many there are.
    dangling_start_map: HashMap<String, DanglingEdgeInfo>,
}

/// During postprocessing, when stitching dangling edges together, the edge type is required to
/// ensure that a Go does not jump to a Gateway join, and vice versa. It's more of a syntax
/// correctness check.
#[derive(PartialEq, Debug)]
enum EdgeType {
    Go,
    GatewayBranch,
    GatewayJoin,
    DataAssociation,
}

type ParseError = Vec<(String, TokenCoordinate, Level)>;

struct Parser {
    graph: Graph,
    context: ParseContext,
}

impl Parser {
    /// Create a new parser from a lexer
    fn new() -> Self {
        Parser {
            graph: Graph::default(),
            context: ParseContext {
                last_node_id: None,
                current_pool: None,
                current_lane: None,
                lifeline_state: LifelineState::NoLifelineActive {
                    previous_lifeline_termination_statement: None,
                },
                current_token_coordinate: TokenCoordinate::default(),
                branching: ParseBranching::default(),
            },
        }
    }

    /// Parses the input and returns a graph
    fn parse(
        self,
        input: String,
        tokens: StatementStream,
    ) -> Result<Graph, Box<dyn std::error::Error>> {
        self.parse_inner(tokens).map_err(|e| {
            let result = Renderer::styled()
                .render(
                    Level::Error.title("Parser error").snippet(
                        Snippet::source(&input)
                            .line_start(1)
                            .fold(true)
                            .annotations(e.iter().map(
                                |(label, TokenCoordinate { start, end }, annotation_type)| {
                                    annotation_type.span(*start..*end).label(label)
                                },
                            )),
                    ),
                )
                .to_string();

            Box::new(lexer::MyParseError(result)).into()
        })
    }

    /// Exists purely for easier mapping of ParseError to Box<dyn...>
    fn parse_inner(mut self, tokens: StatementStream) -> Result<Graph, ParseError> {
        // Parse the input
        for (coordinate, token) in tokens {
            self.context.current_token_coordinate = coordinate;
            match token {
                Statement::Pool(label) => self.parse_pool(&label)?,
                Statement::Lane(label) => self.parse_lane(&label)?,
                Statement::Event(meta) => self.parse_event(meta, false)?,
                Statement::EventEnd(meta) => self.parse_event(meta, true)?,
                Statement::ActivityTask(meta) => self.parse_task(meta)?,
                Statement::GatewayBranchStart(meta) => self.parse_gateway_branch_start(meta)?,
                Statement::GatewayBranchEnd(meta) => self.parse_gateway_branch_end(meta)?,
                Statement::GatewayJoinStart(meta) => self.parse_gateway_join_start(meta)?,
                Statement::GatewayJoinEnd(meta) => self.parse_gateway_join_end(meta)?,
                Statement::SequenceFlowStart(meta) => self.parse_sequence_flow_start(meta)?,
                Statement::SequenceFlowEnd(meta) => self.parse_sequence_flow_end(meta)?,
                Statement::Data(meta) => self.parse_data(meta)?,
                Statement::Layout(_) => {
                    eprintln!("Warning: layout instructions are currently ignored.")
                } //Token::Go => {
                  //    self.parse_go()?;
                  //    continue;
                  //}
                  //Token::Label(label) => self.parse_label(&label)?,
                  //Token::Join(exit_label, text) => self.parse_join(exit_label, text)?,
            }
        }

        let dangling_end_map = self.context.branching.dangling_end_map;
        let dangling_start_map = self.context.branching.dangling_start_map;
        let all_keys: std::collections::HashSet<_> = dangling_end_map
            .keys()
            .chain(dangling_start_map.keys())
            .collect();

        for key in &all_keys {
            match (dangling_end_map.get(*key), dangling_start_map.get(*key)) {
                (Some(starts), Some(end)) => {
                    for start in starts {
                        if start.edge_type != end.edge_type {
                            return Err(vec![(format!("The label {key} cannot go from this {{}} type ..."), start.tc, Level::Error), ("... to this {} type. Can only combine `X ->` with `G <-`, `G->` with `X <-` and `F ->` with `F <-`".to_string(), end.tc,Level::Error)]);
                        }
                        if self.graph.nodes[start.known_node_id.0].pool
                            != self.graph.nodes[end.known_node_id.0].pool
                        {
                            return Err(vec![(format!("The sequence flow connection `{key}` is not allowed to cross pools, but this node ..."), start.tc, Level::Error ), ("... and this node live in different pools.".to_string(), end.tc, Level::Error)]);
                        }
                        self.graph.add_edge(
                            start.known_node_id,
                            end.known_node_id,
                            Some(start.edge_text.clone()),
                        );
                    }
                }
                (Some(starts), None) => {
                    return Err(starts
                        .iter()
                        .map(|start| {
                            (
                                format!("Label `{key}` is not defined elsewhere in the document."),
                                start.tc,
                                Level::Error,
                            )
                        })
                        .collect());
                }
                (None, Some(end)) => {
                    return Err(vec![(
                        format!("Label `{key}` is not defined elsewhere in the document."),
                        end.tc,
                        Level::Error,
                    )]);
                }
                (None, None) => unreachable!(), // Keys come from at least one map
            }
        }

        Ok(self.graph)
    }

    /// Set the current pool
    fn parse_pool(&mut self, label: &str) -> Result<(), ParseError> {
        let old_state = std::mem::replace(
            &mut self.context.lifeline_state,
            LifelineState::NoLifelineActive {
                previous_lifeline_termination_statement: None,
            },
        );
        err_from_unfinished_lifeline(old_state, self.context.current_token_coordinate)?;
        self.context.current_pool = Some(label.to_string());
        self.context.current_lane = None;
        self.context.last_node_id = None;
        // For better error reporting, reset this to "fresh".
        Ok(())
    }

    /// Set the current lane
    fn parse_lane(&mut self, label: &str) -> Result<(), ParseError> {
        let old_state = std::mem::replace(
            &mut self.context.lifeline_state,
            LifelineState::NoLifelineActive {
                previous_lifeline_termination_statement: None,
            },
        );
        err_from_unfinished_lifeline(old_state, self.context.current_token_coordinate)?;
        self.context.current_lane = Some(label.to_string());
        self.context.last_node_id = None;
        Ok(())
    }

    /// Parse a gateway
    fn parse_gateway_branch_start(&mut self, meta: GatewayNodeMeta) -> Result<(), ParseError> {
        // Assign a unique node ID to this gateway
        let node_id = self.graph.add_node(
            BpmnEvent::Gateway(meta.gateway_type),
            self.context.current_pool.clone(),
            self.context.current_lane.clone(),
        );

        self.connect_nodes(
            node_id,
            LifelineState::NoLifelineActive {
                previous_lifeline_termination_statement: Some(
                    self.context.current_token_coordinate,
                ),
            },
        )?;

        for EdgeMeta { target, text_label } in meta.sequence_flow_jump_metas.into_iter() {
            self.context.branching.dangling_end_map.try_insert(
                target,
                vec![DanglingEdgeInfo {
                    known_node_id: node_id,
                    edge_type: EdgeType::GatewayBranch,
                    edge_text: text_label,
                    tc: self.context.current_token_coordinate,
                }],
            ).map_err(|e| vec![(
                    format!("A label used in a gateway branch node `->{}` must be unique. It has been used here ....",
                    e.entry.key()), e.entry.get().first().unwrap().tc, Level::Error
                ),
                ("... and here".to_string(), self.context.current_token_coordinate, Level::Error),
            ])?;
        }
        Ok(())
    }

    // Helper to handle joins. Note, this is actually a real BPMN node, other than the Go target
    // and the intermediate labels.
    fn parse_gateway_join_end(&mut self, meta: GatewayNodeMeta) -> Result<(), ParseError> {
        // Assign a unique node ID to this gateway
        let node_id = self.graph.add_node(
            BpmnEvent::Gateway(meta.gateway_type),
            self.context.current_pool.clone(),
            self.context.current_lane.clone(),
        );
        let old_state = std::mem::replace(
            &mut self.context.lifeline_state,
            LifelineState::ActiveLifeline {
                last_node_id: node_id,
                token_coordinate: self.context.current_token_coordinate,
            },
        );
        err_from_unfinished_lifeline(old_state, self.context.current_token_coordinate)?;
        for EdgeMeta { target, text_label } in meta.sequence_flow_jump_metas.into_iter() {
            self.context.branching.dangling_start_map.try_insert(
                target.clone(),
                DanglingEdgeInfo {
                    known_node_id: node_id,
                    edge_type: EdgeType::GatewayJoin,
                    edge_text: text_label,
                    tc: self.context.current_token_coordinate,
                },
            ).map_err(|e|
                vec![
                (format!("A label used in a sequence flow `<-{}` must be unique. It has been used here ....", e.entry.key()), e.entry.get().tc,Level::Error),
                ("... and here".to_string(), self.context.current_token_coordinate, Level::Error),
                ]
            )?;
        }
        Ok(())
    }

    fn parse_gateway_branch_end(&mut self, meta: GatewayInnerMeta) -> Result<(), ParseError> {
        match self.context.lifeline_state {
LifelineState::ActiveLifeline { token_coordinate, .. } | LifelineState::StartedFromBranch { token_coordinate, .. } =>
            return Err(vec![
(
                "Any sequence flow above this statement should be finished, but this is not the case.".to_string(),
                self.context.current_token_coordinate,Level::Error
            ),
            (
                "This statement does not finish the sequence flow. Try using `. End Event`, `F ->somewhere` or `X ->lbl1 ->lbl2` to finish the sequence flow.".to_string(),
                    token_coordinate, Level::Note
            )
            ]),
_ => (),
        }
        if !meta.sequence_flow_jump_meta.text_label.is_empty() {
            dbg!("The label text should be stored within the edge or so, and warn if the other end already provided a text for this edge.");
        }
        self.context.lifeline_state = LifelineState::StartedFromBranch {
            edge_type: EdgeType::GatewayBranch,
            name: meta.sequence_flow_jump_meta.target,
            token_coordinate: self.context.current_token_coordinate,
        };
        Ok(())
    }

    fn parse_gateway_join_start(&mut self, meta: GatewayInnerMeta) -> Result<(), ParseError> {
        let old_state = std::mem::replace(
            &mut self.context.lifeline_state,
            LifelineState::NoLifelineActive {
                previous_lifeline_termination_statement: Some(
                    self.context.current_token_coordinate,
                ),
            },
        );
        let known_node_id = match old_state {
            LifelineState::ActiveLifeline { last_node_id, .. } => last_node_id,
            a => {
                return Err(err_from_no_lifeline_active(
                    a,
                    self.context.current_token_coordinate,
                ))
            }
        };
        // The only place where there are multiple uses of the same label allowed.
        self.context
            .branching
            .dangling_end_map
            .entry(meta.sequence_flow_jump_meta.target)
            .or_default()
            .push(DanglingEdgeInfo {
                known_node_id,
                edge_type: EdgeType::GatewayJoin,
                edge_text: meta.sequence_flow_jump_meta.text_label,
                tc: self.context.current_token_coordinate,
            });
        Ok(())
    }

    /// Connects the current node with the previous node, handling incoming joining/jumping
    /// lifelines as well.
    fn connect_nodes(
        &mut self,
        current_node_id: NodeId,
        new_lifeline_state: LifelineState,
    ) -> Result<(), ParseError> {
        // Use `replace` here to avoid lifetime errors.
        match std::mem::replace(&mut self.context.lifeline_state, new_lifeline_state) {
            LifelineState::StartedFromBranch {
                edge_type,
                name,
                token_coordinate,
            } => {
                self.context.branching.dangling_start_map.try_insert(
                    name,
                    DanglingEdgeInfo {
                        known_node_id: current_node_id,
                        edge_type,
                        edge_text: "".to_string(),
                        tc: token_coordinate,
                    },
                ).map_err(|e| {
                vec![
                (format!("A label used in a sequence flow `<-{}` must be unique. It has been used here ....", e.entry.key()), e.entry.get().tc,Level::Error),
                ("... and here".to_string(), self.context.current_token_coordinate, Level::Error),
                ]
            })?;
            }
            LifelineState::ActiveLifeline { last_node_id, .. } => {
                // No text for regular lifeline edges.
                self.graph.add_edge(last_node_id, current_node_id, None);
            }
            a => {
                return Err(err_from_no_lifeline_active(
                    a,
                    self.context.current_token_coordinate,
                ))
            }
        }
        Ok(())
    }

    fn connect_data_node(
        &mut self,
        data_node_id: DataNodeId,
        new_lifeline_state: LifelineState,
    ) -> Result<(), ParseError> {
        match std::mem::replace(&mut self.context.lifeline_state, new_lifeline_state) {
            LifelineState::StartedFromBranch {
                edge_type,
                name,
                token_coordinate,
            } => {
                return Err(vec![(
                    "This statement requires an active lifeline.".to_string(),
                    token_coordinate,
                    Level::Error,
                )])
            }

            LifelineState::ActiveLifeline { last_node_id, .. } => {
                self.graph
                    .add_data_edge_reversed(DataNodeId(data_node_id.0), last_node_id, None);
            }
            a => {
                return Err(err_from_no_lifeline_active(
                    a,
                    self.context.current_token_coordinate,
                ))
            }
        }
        Ok(())
    }

    /// Common function to parse an event or task
    fn parse_event(&mut self, meta: EventMeta, is_end: bool) -> Result<(), ParseError> {
        match self.context.lifeline_state {
            LifelineState::NoLifelineActive { .. } if !is_end => {
                let last_node_id = self.graph.add_node(
                    BpmnEvent::Start(meta),
                    self.context.current_pool.clone(),
                    self.context.current_lane.clone(),
                );
                self.context.lifeline_state = LifelineState::ActiveLifeline {
                    last_node_id,
                    token_coordinate: self.context.current_token_coordinate,
                };
            }
            _ if is_end => {
                let node_id = self.graph.add_node(
                    BpmnEvent::End(meta),
                    self.context.current_pool.clone(),
                    self.context.current_lane.clone(),
                );
                self.connect_nodes(
                    node_id,
                    LifelineState::NoLifelineActive {
                        previous_lifeline_termination_statement: Some(
                            self.context.current_token_coordinate,
                        ),
                    },
                )?;
            }
            _ => {
                let node_id = self.graph.add_node(
                    BpmnEvent::Middle(meta),
                    self.context.current_pool.clone(),
                    self.context.current_lane.clone(),
                );
                self.connect_nodes(
                    node_id,
                    LifelineState::ActiveLifeline {
                        last_node_id: node_id,
                        token_coordinate: self.context.current_token_coordinate,
                    },
                )?;
            }
        };
        Ok(())
    }

    fn parse_task(&mut self, meta: NodeMeta) -> Result<(), ParseError> {
        let node_id = self.graph.add_node(
            BpmnEvent::ActivityTask(meta),
            self.context.current_pool.clone(),
            self.context.current_lane.clone(),
        );
        self.connect_nodes(
            node_id,
            LifelineState::ActiveLifeline {
                last_node_id: node_id,
                token_coordinate: self.context.current_token_coordinate,
            },
        )
    }

    fn parse_sequence_flow_start(&mut self, meta: SequenceFlowMeta) -> Result<(), ParseError> {
        let old_state = std::mem::replace(
            &mut self.context.lifeline_state,
            LifelineState::NoLifelineActive {
                previous_lifeline_termination_statement: Some(
                    self.context.current_token_coordinate,
                ),
            },
        );
        let known_node_id = match old_state {
            LifelineState::ActiveLifeline { last_node_id, .. } => last_node_id,
            a => {
                return Err(err_from_no_lifeline_active(
                    a,
                    self.context.current_token_coordinate,
                ))
            }
        };

        self.context
            .branching
            .dangling_end_map
            .try_insert(meta.sequence_flow_jump_meta.target,
            vec![DanglingEdgeInfo {
                known_node_id,
                edge_type: EdgeType::Go,
                edge_text: meta.sequence_flow_jump_meta.text_label,
                tc: self.context.current_token_coordinate,
            }]).map_err(|e| vec![(
                    format!("A label used in a sequence flow `->{}` must be unique. It has been used here ....",
                    e.entry.key()), e.entry.get().first().unwrap().tc, Level::Error
                ),
                ("... and here".to_string(), self.context.current_token_coordinate, Level::Error),
            ])?;

        Ok(())
    }

    /// Handle parsing for a join 'Go' token
    fn parse_sequence_flow_end(&mut self, meta: SequenceFlowMeta) -> Result<(), ParseError> {
        // Check that a valid node type follows
        self.context.lifeline_state = LifelineState::StartedFromBranch {
            edge_type: EdgeType::Go,
            name: meta.sequence_flow_jump_meta.target,
            token_coordinate: self.context.current_token_coordinate,
        };

        Ok(())
    }

    fn parse_data(&mut self, meta: DataMeta) -> Result<(), ParseError> {
        let datakind = match meta.data_kind {
            DataKind::DataObject => BpmnEvent::DataObjectReference(meta.clone()),
            DataKind::DataStore => BpmnEvent::DataStoreReference(meta.clone()),
        };

        let data_node_id = self.graph.add_data_node(
            Some(datakind),
            self.context.current_pool.clone(),
            self.context.current_lane.clone(),
        );

        self.connect_data_node(
            data_node_id,
            LifelineState::ActiveLifeline {
                last_node_id: self.graph.nodes.last().unwrap().id,
                token_coordinate: self.context.current_token_coordinate,
            },
        )?;

        for EdgeMeta { target, text_label } in meta.data_association_jump_metas.into_iter() {
            let edge_type = EdgeType::DataAssociation;
            self.context
                .branching
                .dangling_end_map
                .try_insert(
                    target,
                    vec![DanglingEdgeInfo {
                        known_node_id: NodeId(data_node_id.0),
                        edge_type,
                        edge_text: text_label,
                        tc: self.context.current_token_coordinate,
                    }],
                ).map_err(|e| vec![(
                        format!("A label used in a sequence flow `->{}` must be unique. It has been used here ....",
                        e.entry.key()), e.entry.get().first().unwrap().tc, Level::Error
                    ),
                    ("... and here".to_string(), self.context.current_token_coordinate, Level::Error),
                ])?;
        }

        Ok(())
    }
}

#[track_caller]
fn err_from_no_lifeline_active(
    l: LifelineState,
    current_token_coordinate: TokenCoordinate,
) -> ParseError {
    match l {
            LifelineState::NoLifelineActive { previous_lifeline_termination_statement: None } => vec![
(
                "This statement requires an active sequence flow. Try to add a start event `#` before this line.".to_string(),
                current_token_coordinate, Level::Error
)]
            ,
            LifelineState::NoLifelineActive { previous_lifeline_termination_statement: Some(tc) } => vec![
(
                "This statement requires an active sequence flow.".to_string(),
                current_token_coordinate, Level::Error
),(
                "The sequence flow was previously terminated here.".to_string(),
                tc, Level::Note
)]
            ,
            LifelineState::StartedFromBranch { token_coordinate, .. } => vec![
(
                "This statement requires an active sequence flow.".to_string(),
                current_token_coordinate, Level::Error
),(
                "This statement itself is just a meta instruction and does not add a node. Try adding a `- Some Activity` after this statement.".to_string(),
                token_coordinate, Level::Note
)]
            ,
        l => panic!("{l:?}, {current_token_coordinate:?}"),
    }
}

fn err_from_unfinished_lifeline(
    l: LifelineState,
    current_token_coordinate: TokenCoordinate,
) -> Result<(), ParseError> {
    match l {
        LifelineState::StartedFromBranch {
            token_coordinate, ..
        } => Err(vec![
            (
                "This statement cannot appear within an active sequence flow.".to_string(),
                current_token_coordinate,
                Level::Error,
            ),
            (
                // Have a slightly clearer error message here.
                "This statement introduced a sequence flow.".to_string(),
                token_coordinate,
                Level::Note,
            ),
        ]),
        LifelineState::ActiveLifeline {
            token_coordinate, ..
        } => Err(vec![
            (
                "This statement cannot appear within an active sequence flow.".to_string(),
                current_token_coordinate,
                Level::Error,
            ),
            (
                "This statement is part of the currently active sequence flow.".to_string(),
                token_coordinate,
                Level::Note,
            ),
        ]),
        LifelineState::NoLifelineActive { .. } => Ok(()),
    }
}

pub fn parse(input: String) -> Result<Graph, Box<dyn std::error::Error>> {
    Parser::new().parse(input.clone(), lex(input)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn basic() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
# Start Event
- Middle Event
. End Event
"#;

        let graph = parse(input.to_string())?;
        assert!(
            graph.edges.len() == 2,
            "Edge count is wrong, should be: {}",
            graph.edges.len()
        );
        Ok(())
    }

    #[test]
    fn gateway() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
# Start Event
X ->Branch1 "Condition 1" ->Branch2 "Cond 2"

G <- Branch1
- Task B1
G ->JoinPoint "Joining Branches"

G <- Branch2
- Task B1
G ->JoinPoint "Joining Branches"

X<-JoinPoint
. End Event
"#;

        let graph = parse(input.to_string())?;
        assert!(
            graph.edges.len() == 6,
            "Edge count is wrong, should be: {}",
            graph.edges.len()
        );
        Ok(())
    }

    #[test]
    fn pool() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
= Pool
== Lane1
# Start Event
- Task1
. End Event
== Lane2
# Start Event2
- Task2
. End Event 2
"#;

        let graph = parse(input.to_string())?;
        assert!(
            graph.edges.len() == 4,
            "Edge count is wrong, should be: {}",
            graph.edges.len()
        );
        Ok(())
    }

    #[test]
    fn jump() -> Result<(), Box<dyn std::error::Error>> {
        let input = r#"
= Pool
== Lane1
# Start Event
- Task
F ->jump
== Lane2
F <-jump
- Task
. End Event
"#;

        let graph = parse(input.to_string())?;
        assert!(
            graph.edges.len() == 3,
            "Edge count is wrong, should be: {}",
            graph.edges.len()
        );
        Ok(())
    }
}
