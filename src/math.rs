use std::fmt::Debug;
use std::path::Component::ParentDir;
use rust_igraph::*;

pub struct AutoCells
<
    StateType: Copy + Debug,

    const STATE_COUNT: usize,
    const ADJACENT_COUNT: usize,
>
{
    pub adjacent_functions: [Box<dyn Fn(usize) -> usize>; ADJACENT_COUNT],
    pub rule_pointer: fn([StateType; ADJACENT_COUNT]) -> usize,

    pub world_size: usize,
}

impl
<
    StateType: Copy + Debug,

    const STATE_COUNT: usize,
    const ADJACENT_COUNT: usize
>
AutoCells<StateType, STATE_COUNT, ADJACENT_COUNT>
{
    pub fn new
    (
        adjacent_functions: [Box<dyn Fn(usize) -> usize>; ADJACENT_COUNT],
        rule_pointer: fn([StateType; ADJACENT_COUNT]) -> usize,

    ) -> Self {

        Self
        {
            adjacent_functions,
            rule_pointer,

            world_size: STATE_COUNT.pow(ADJACENT_COUNT as u32),
        }
    }

    pub fn tick(&self, world: Vec<StateType>) -> Vec<StateType> {

        assert_eq!(world.len(), self.world_size);

        (0..world.len()).map(|cell| -> StateType {

            world[(self.rule_pointer)(
                (0..ADJACENT_COUNT).map(|adjacent| {

                    world[self.adjacent_functions[adjacent](cell)]
                }).collect::<Vec<StateType>>().try_into().unwrap()
            )]

        }).collect()
    }
}

impl<const STATE_COUNT: usize, const ADJACENT_COUNT: usize> AutoCells<usize, STATE_COUNT, ADJACENT_COUNT> {

    pub fn world_to_index(&self, world: Vec<usize>) -> usize {

        (0..world.len()).map(|i| -> usize {

            world[i] * STATE_COUNT.pow(i as u32)
        }).sum()
    }

    pub fn iterator(&self) -> AutoCellsIterator<STATE_COUNT> {

        AutoCellsIterator {

            next: vec![0; STATE_COUNT.pow(ADJACENT_COUNT as u32)],
            done: false,
        }
    }


    /// returns a graph of the "shape" of a given autocells state, showing all nodes and the adjacencies between them.
    pub fn shape_graph(&self) -> Graph {

        let mut graph = Graph::new(self.world_size as u32, true).unwrap();

        (0..self.world_size).for_each(|i| {

            self.adjacent_functions.iter().for_each(|f| {

                graph.add_edge(VertexId::from(i as u32), VertexId::from(f(i) as u32)).unwrap();
            })
        });

        graph
    }

    /// returns a functional graph representing self.tick(w) for all world-states "w".
    pub fn action_graph(&self) -> Graph {

        let world_count = STATE_COUNT.pow(self.world_size as u32) as u32;

        let mut graph = Graph::new(world_count, true).unwrap();

        self.iterator().for_each(|i| {

            graph.add_edge(

                VertexId::from(Self::world_to_num(i.clone()) as u32),
                VertexId::from(Self::world_to_num(self.tick(i)) as u32),
            ).unwrap();
        });

        graph
    }

    pub fn world_to_num(world: Vec<usize>) -> usize {

        (0..world.len()).map(|i| -> usize {

            world[i] * STATE_COUNT.pow(i as u32)
        }).sum()
    }
}

pub fn trivial_autocells
<
    const STATE_COUNT: usize,
    const ADJACENT_COUNT: usize
>(self_adjacency: bool) ->
AutoCells
<
    usize,

    STATE_COUNT,
    ADJACENT_COUNT,
>
{
    assert_ne!(ADJACENT_COUNT * STATE_COUNT, 0);

    let mut adjacent_functions =
        (0..ADJACENT_COUNT - if self_adjacency { 1 } else { 0 })
            .map(|adjacent| -> Box<dyn Fn(usize) -> usize>
                {
                    Box::new(move |cell| -> usize {

                        let base = STATE_COUNT.pow(adjacent as u32);
                        let digit = (cell / base) % STATE_COUNT;

                        (cell - (digit * base)) + (((digit + 1) % STATE_COUNT) * base)
                    })

                }).collect::<Vec<Box<dyn Fn(usize) -> usize>>>();

    if self_adjacency
    {
        adjacent_functions.push(Box::new(|cell| -> usize { cell }))
    }

    let adjacent_functions: [Box<dyn Fn(usize) -> usize>; ADJACENT_COUNT] =
        match adjacent_functions.try_into() { Ok(x) => x, _ => unreachable!("misconstructed array in trivial_autocells()") };

    AutoCells::<usize, STATE_COUNT, ADJACENT_COUNT>::new(

        adjacent_functions,

        |adjacent_states|
            {
                let mut index: u32 = 0;
                adjacent_states.map(|adjacent_state| -> usize {

                    index += 1;
                    adjacent_state.pow(index - 1)

                }).iter().sum::<usize>()
            }
    )
}

/// returns false if the whole array overflowed
fn modular_add_to_array(arr: &mut [usize], modulus: usize) -> bool
{
    if arr.len() == 0 { return false; }
    if arr[0] < (modulus - 1) { arr[0] += 1; true }
    else
    {
        arr[0] = 0;
        modular_add_to_array(&mut arr[1..], modulus)
    }
}

pub struct AutoCellsIterator
<
    const STATE_COUNT: usize,
>
{
    next: Vec<usize>,
    done: bool,
}
impl<const STATE_COUNT: usize> Iterator for AutoCellsIterator<STATE_COUNT> {
    type Item = Vec<usize>;

    fn next(&mut self) -> Option<Vec<usize>> {

        if self.done { return None }

        let out = self.next.clone();

        if !modular_add_to_array(&mut self.next, STATE_COUNT) { self.done = true; }

        Some(out)
    }
}

/// transforms a functional graph into a vector of trees with flipped order and loops collapsed to a single point, along with the size of their core loops.
pub fn to_trees(graph: &Graph, cull_leaves: bool) -> Vec<(usize, Graph)> {

    decompose(&graph.reverse().unwrap().to_undirected(ToUndirectedMode::Each).unwrap()).unwrap().into_iter().map(|subgraph| {

        let mut subgraph = subgraph.to_directed(ToDirectedMode::Arbitrary).unwrap();

        let cycle = find_cycle(&subgraph, CycleMode::Out).unwrap();
        let center_vertex: VertexId = { subgraph.add_vertices(1).unwrap(); subgraph.vcount() - 1 };

        let cycle_child_edges = cycle.vertices.iter().map(|cycle_vertex| -> Vec<u32> {

            subgraph.edge_ids().filter_map(|edge_id| {
                if subgraph.edge(edge_id).unwrap().0 != *cycle_vertex { None } else { Some(edge_id) }
            }).collect()

        }).flatten().collect::<Vec<u32>>();

        let mut cycle_child_vertices: Vec<u32> = cycle_child_edges.iter().map(|child| { subgraph.edge(*child).unwrap().1 }).collect();

        cycle_child_vertices.sort_unstable(); cycle_child_vertices.dedup();

        subgraph.delete_edges(&*cycle_child_edges).unwrap();

        cycle_child_vertices.iter().for_each(|cycle_child_vertex| { subgraph.add_edge(center_vertex, *cycle_child_vertex).unwrap(); });

        if cull_leaves {

            subgraph.delete_vertices(
                &*(0..subgraph.vcount())
                    .filter(|vertex_id| {
                        subgraph.out_degree(*vertex_id).unwrap() == 0
                    }).collect::<Vec<u32>>()
            ).unwrap();
        }


        (cycle.vertices.len(), subgraph.reverse().unwrap())
    }).collect()
}