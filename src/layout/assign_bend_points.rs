use crate::common::bpmn_event::get_node_size;
use crate::common::graph::Graph;
use std::collections::BinaryHeap;
use std::collections::HashMap;

const STEP_SIZE: usize = 1;
const NODE_MARGIN: isize = 20;

pub fn assign_bend_points(graph: &mut Graph) {
    let mut matrix_width = 0;
    let mut matrix_height = 0;

    // HashMap to store coordinates of obstacles with node id as key
    // HashMap stores tuples of top left and bottom right coordinates of obstacles
    let mut matrix: HashMap<usize, (usize, usize, usize, usize)> = HashMap::new();

    for pool in graph.pools.iter() {
        for lane in pool.lanes.iter() {
            for node in lane.layers.iter() {
                if let (Some(x), Some(y), Some(x_offset), Some(y_offset)) =
                    (node.x, node.y, node.x_offset, node.y_offset)
                {
                    let (width, height) = get_node_size(node.event.as_ref().unwrap());
                    let x2 = x as usize + width as usize + x_offset as usize;
                    let y2 = y as usize + height as usize + y_offset as usize;
                    if x2 > matrix_width {
                        matrix_width = x2 + 50;
                    }
                    if y2 > matrix_height {
                        matrix_height = y2 + 50;
                    }
                    matrix.insert(
                        node.id.clone(),
                        (
                            x as usize + x_offset as usize - NODE_MARGIN as usize,
                            y as usize + y_offset as usize - NODE_MARGIN as usize,
                            x2 + NODE_MARGIN as usize,
                            y2 + NODE_MARGIN as usize,
                        ),
                    );
                }
            }
        }
    }

    for edge in graph.edges.iter_mut() {
        let (from_x, from_y, from_x2, from_y2) = matrix.get(&edge.from).unwrap();
        let (to_x, to_y, to_x2, to_y2) = matrix.get(&edge.to).unwrap();

        // Vector to store different paths
        let mut choices: HashMap<usize, Vec<(usize, usize)>> = HashMap::new();

        // Add first bend points to choices
        // There must be a margin before any bend points
        // So for the top path, the bend point is at (top_start_x, top_start_y - NODE-MARGIN)
        let start_points_with_margins = vec![
            ((from_x + (from_x2 - from_x) / 2, *from_y), (0, NODE_MARGIN)), // Top exit
            (
                (*from_x2, from_y + (from_y2 - from_y) / 2),
                (-NODE_MARGIN, 0),
            ), // Right exit
            (
                (from_x + (from_x2 - from_x) / 2, *from_y2),
                (0, -NODE_MARGIN),
            ), // Bottom exit
        ];

        let end_points_with_margins = vec![
            ((to_x + (to_x2 - to_x) / 2, *to_y), (0, NODE_MARGIN)), // Top entry
            ((*to_x, to_y + (to_y2 - to_y) / 2), (NODE_MARGIN, 0)), // Left entry
            ((to_x + (to_x2 - to_x) / 2, *to_y2), (0, -NODE_MARGIN)), // Bottom entry
        ];

        for (start_point, margin_start) in start_points_with_margins {
            for (end_point, margin_end) in &end_points_with_margins {
                let path = find_path(
                    start_point.0,
                    start_point.1,
                    end_point.0,
                    end_point.1,
                    matrix_width,
                    matrix_height,
                    &matrix,
                );
                if path.len() > 0 {
                    let length = path.len();
                    let mut bend_points = vec![];
                    for (x, y) in path {
                        bend_points.push((x, y));
                    }

                    // Add the correct start and end coordinates
                    bend_points.insert(
                        0,
                        (
                            (start_point.0 as isize + margin_start.0) as usize,
                            (start_point.1 as isize + margin_start.1) as usize,
                        ),
                    );
                    bend_points.push((
                        (end_point.0 as isize + margin_end.0) as usize,
                        (end_point.1 as isize + margin_end.1) as usize,
                    ));
                    choices.insert(length, bend_points);
                }
            }
        }

        // Add shortest path to edge
        if choices.len() > 0 {
            let mut min = usize::MAX;
            let mut min_path = vec![];
            for (length, path) in choices.iter() {
                if *length < min {
                    min = *length;
                    min_path = path.clone();
                }
            }
            edge.bend_points = Some(
                min_path
                    .iter()
                    .map(|&(x, y)| (x as f64, y as f64))
                    .collect(),
            );
        }
        println!("Edge from {} to {} routed", edge.from, edge.to);
    }
}

fn find_path(
    cur_x: usize,
    cur_y: usize,
    end_x: usize,
    end_y: usize,
    matrix_width: usize,
    matrix_height: usize,
    matrix: &HashMap<usize, (usize, usize, usize, usize)>,
) -> Vec<(usize, usize)> {
    let mut open_set = BinaryHeap::new();
    let mut came_from = HashMap::new();
    let mut g_score = HashMap::new();
    let mut f_score = HashMap::new();

    open_set.push((0, cur_x, cur_y));
    g_score.insert((cur_x, cur_y), 0);

    while let Some((_, mut x, mut y)) = open_set.pop() {
        if ((x as isize - end_x as isize).abs() <= STEP_SIZE as isize)
            && ((y as isize - end_y as isize).abs() <= STEP_SIZE as isize)
        {
            let mut path = vec![(x, y)];
            while let Some(&(px, py)) = came_from.get(&(x, y)) {
                path.push((px, py));
                x = px;
                y = py;
            }
            path.reverse();
            return path;
        }

        for (dx, dy) in [
            (0, STEP_SIZE as isize),
            (STEP_SIZE as isize, 0),
            (0, -(STEP_SIZE as isize)),
            (-(STEP_SIZE as isize), 0),
        ] {
            let nx = x as isize + dx;
            let ny = y as isize + dy;

            if nx < 0 || ny < 0 {
                continue;
            }

            let nx = nx as usize;
            let ny = ny as usize;

            if !is_in_grid(nx, ny, matrix_width, matrix_height) || is_in_obstacle(nx, ny, &matrix) {
                continue;
            }

            let tentative_g_score = g_score.get(&(x, y)).unwrap_or(&usize::MAX) + 1;

            if tentative_g_score < *g_score.get(&(nx, ny)).unwrap_or(&usize::MAX) {
                came_from.insert((nx, ny), (x, y));
                g_score.insert((nx, ny), tentative_g_score);
                let h_score = find_distance_to_end(nx, ny, end_x, end_y);
                f_score.insert((nx, ny), tentative_g_score + h_score);
                open_set.push((-(tentative_g_score as isize + h_score as isize), nx, ny));
            }
        }
    }

    vec![]
}

fn is_in_grid(x: usize, y: usize, matrix_width: usize, matrix_height: usize) -> bool {
    x < matrix_width && y < matrix_height
}

fn is_in_obstacle(
    x: usize,
    y: usize,
    matrix: &HashMap<usize, (usize, usize, usize, usize)>,
) -> bool {
    for (_, (x1, y1, x2, y2)) in matrix.iter() {
        if x >= *x1 && x <= *x2 && y >= *y1 && y <= *y2 {
            return true;
        }
    }
    false
}

fn find_distance_to_end(x: usize, y: usize, end_x: usize, end_y: usize) -> usize {
    ((x as isize - end_x as isize).abs() + (y as isize - end_y as isize).abs()) as usize
}
