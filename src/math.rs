use std::io::Write;
use std::fmt::Debug;
use std::io::BufWriter;
use std::ops::Add;
use egui::Vec2;
use petgraph::stable_graph::{NodeIndex, StableGraph};
use rand::random;

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

    pub fn adjacency_graph(&self) -> egui_graphs::Graph {

        let mut g: StableGraph<(), ()> = StableGraph::new();

        let worlds = self.iterator();
        let nodes: Vec<_> = worlds.map(|_| {

            g.add_node(())
        }).collect();

        let worlds = self.iterator();
        let mut i = 0;
        worlds.for_each(|w| {

            g.add_edge(nodes[i], nodes[self.world_to_index(self.tick(w))], ());

            i += 1;
        });

        let mut g = egui_graphs::Graph::from(&g);

        let node_indices: Vec<NodeIndex> = g.nodes_iter().map(|(i, _)| {

            i
        }).collect();

        node_indices.iter().for_each(|i| {

            let n = g.node_mut(*i).unwrap();

            n.set_location(n.location().add(Vec2::new(random::<f32>() * 10000.0, random::<f32>() * 10000.0)))
        });

        g
    }

    pub fn write_edge_file(&self, name: String) {

        let file = std::fs::File::create(name).unwrap();
        let mut writer = BufWriter::new(file);

        writeln!(writer, "Source,Target").unwrap();

        self.iterator().for_each(|w| {

            writeln!(writer, "{},{}", self.world_to_index(w.clone()), self.world_to_index(self.tick(w)));
        });

        writer.flush();
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