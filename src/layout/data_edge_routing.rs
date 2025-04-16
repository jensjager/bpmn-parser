use crate::common::bpmn_event::{get_node_size, BpmnEvent};
use crate::common::datanode::DataNode;
use crate::common::graph::DataNodeId;
use crate::common::graph::Graph;
use crate::common::graph::NodeId;
use crate::common::node::Node;
use std::collections::HashMap;

const NODE_MARGIN: usize = 5;

pub fn find_data_edges(graph: &mut Graph) {
    // HashMap to store coordinates of obstacles with node id and is_datanode as key
    // HashMap stores tuples of top left and bottom right coordinates of obstacles
    let mut matrix: HashMap<(usize, bool), (usize, usize, usize, usize)> = HashMap::new();

    for node in &graph.nodes {
        if node.event == Some(BpmnEvent::Dummy()) {
            continue;
        }
        if let (Some(x), Some(y), Some(x_offset), Some(y_offset)) =
            (node.x, node.y, node.x_offset, node.y_offset)
        {
            let (width, height) = get_node_size(node.event.as_ref().unwrap());
            add_to_matrix(
                &mut matrix,
                &node.id.0,
                false,
                width,
                height,
                x,
                y,
                x_offset,
                y_offset,
            );
        }
    }

    for data_node in graph.data_nodes.iter() {
        if let (Some(x), Some(y), Some(x_offset), Some(y_offset)) = (
            data_node.x,
            data_node.y,
            data_node.x_offset,
            data_node.y_offset,
        ) {
            let (width, height) = get_node_size(&data_node.datatype.clone().unwrap());
            add_to_matrix(
                &mut matrix,
                &data_node.id.0,
                true,
                width,
                height,
                x,
                y,
                x_offset,
                y_offset,
            );
        }
    }

    data_edge_routing(matrix, graph);
}

fn add_to_matrix(
    matrix: &mut HashMap<(usize, bool), (usize, usize, usize, usize)>,
    node_id: &usize,
    is_datanode: bool,
    width: usize,
    height: usize,
    x: f64,
    y: f64,
    x_offset: f64,
    y_offset: f64,
) {
    let x2 = x as usize + width as usize + x_offset as usize;
    let y2 = y as usize + height as usize + y_offset as usize;

    matrix.insert(
        (node_id.clone(), is_datanode),
        (
            x as usize + x_offset as usize,
            y as usize + y_offset as usize,
            x2,
            y2,
        ),
    );
}

fn is_in_obstacle_ignore_self(
    x: usize,
    y: usize,
    from_id: usize,
    to_id: usize,
    matrix: &HashMap<(usize, bool), (usize, usize, usize, usize)>,
) -> bool {
    for ((id, is_datanode), (x1, y1, x2, y2)) in matrix.iter() {
        if (*id == from_id && *is_datanode) || (*id == to_id && !is_datanode) {
            if x >= *x1 + 1 && x <= *x2 - 1 && y >= *y1 + 1 && y <= *y2 - 1 {
                return true;
            }
        } else {
            if x >= *x1 - NODE_MARGIN
                && x <= *x2 + NODE_MARGIN
                && y >= *y1 - NODE_MARGIN
                && y <= *y2 + NODE_MARGIN
            {
                return true;
            }
        }
    }
    false
}

fn data_edge_routing(
    matrix: HashMap<(usize, bool), (usize, usize, usize, usize)>,
    graph: &mut Graph,
) {
    for data_edge in graph.data_edges.iter_mut() {
        if data_edge.dummy.is_some() {
            continue;
        }
        let (start_x_y, end_x_y) = find_start_and_end_points(
            &graph.data_nodes[data_edge.from.0],
            &graph.nodes[data_edge.to.0],
        );

        let mut bend_points = Vec::new();
        for start_points in start_x_y.iter() {
            for end_points in end_x_y.iter() {
                if possible_direct(
                    &matrix,
                    start_points,
                    end_points,
                    data_edge.from,
                    data_edge.to,
                ) {
                    bend_points.push((start_points.clone(), end_points.clone()));
                }
            }
        }
        if bend_points.len() > 0 {
            let edge = find_shortest_path(&bend_points);
            data_edge.bend_points = Some(vec![edge.0, edge.1]);
            if data_edge.is_reversed {
                data_edge.bend_points.as_mut().unwrap().reverse();
            }
        }
    }
}

fn possible_direct(
    matrix: &HashMap<(usize, bool), (usize, usize, usize, usize)>,
    start_xy: &(f64, f64),
    end_xy: &(f64, f64),
    from_id: DataNodeId,
    to_id: NodeId,
) -> bool {
    let dx = end_xy.0 - start_xy.0;
    let dy = end_xy.1 - start_xy.1;

    let steps = dx.abs().max(dy.abs()) as usize;
    let step_x = dx / steps as f64;
    let step_y = dy / steps as f64;

    let mut x = start_xy.0;
    let mut y = start_xy.1;

    for _ in 0..=steps {
        if is_in_obstacle_ignore_self(
            x.round() as usize,
            y.round() as usize,
            from_id.0,
            to_id.0,
            matrix,
        ) {
            return false;
        }
        x += step_x;
        y += step_y;
    }

    true
}

fn find_start_and_end_points(
    data_node: &DataNode,
    node: &Node,
) -> (Vec<(f64, f64)>, Vec<(f64, f64)>) {
    let (dn_x, dn_y, (dn_width, dn_height)) = (
        data_node.x.unwrap() + data_node.x_offset.unwrap(),
        data_node.y.unwrap() + data_node.y_offset.unwrap(),
        get_node_size(&data_node.datatype.clone().unwrap()),
    );
    let (node_x, node_y, (node_width, node_height)) = (
        node.x.unwrap() + node.x_offset.unwrap(),
        node.y.unwrap() + node.y_offset.unwrap(),
        get_node_size(&node.event.as_ref().unwrap()),
    );
    let start_points = vec![
        // left
        ((dn_x, dn_y + dn_height as f64 / 2.0)),
        // right
        ((dn_x + dn_width as f64, dn_y as f64 + dn_height as f64 / 2.0)),
        // top
        ((dn_x + dn_width as f64 / 2.0, dn_y)),
        // bottom
        ((dn_x + dn_width as f64 / 2.0, dn_y + dn_height as f64)),
    ];

    let end_points = vec![
        // left
        ((node_x, node_y + node_height as f64 / 2.0)),
        // right
        ((
            node_x + node_width as f64,
            node_y as f64 + node_height as f64 / 2.0,
        )),
        // top
        ((node_x + node_width as f64 / 2.0, node_y)),
        // bottom
        ((
            node_x + node_width as f64 / 2.0,
            node_y + node_height as f64,
        )),
    ];

    (start_points, end_points)
}

fn find_shortest_path(bend_points: &Vec<((f64, f64), (f64, f64))>) -> ((f64, f64), (f64, f64)) {
    let mut min_distance = f64::MAX;
    let mut start_xy: (f64, f64) = (0.0, 0.0);
    let mut end_xy: (f64, f64) = (0.0, 0.0);
    for (start, end) in bend_points.iter() {
        let distance = (start.0 - end.0).abs() + (start.1 - end.1).abs();
        if distance < min_distance {
            min_distance = distance;
            start_xy = *start;
            end_xy = *end;
        }
    }

    (start_xy, end_xy)
}
