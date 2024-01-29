//! Graph Strucutres
use crate::NodeValue as NodeValueTrait;
use std::{fmt::Debug, iter::Sum};
/// Graph errors
#[derive(Debug)]
pub enum GraphError {
    /// Invalid edge
    InvalidEdge,
    /// Invalid node
    InvalidNode,
    /// Invalid node id
    UnsortedEdges,
}

/// A graph
pub trait Graph<NI: Idx> {
    /// Get the node count
    fn node_count(&self) -> NI;
    /// Get the edge count
    fn edge_count(&self) -> NI;
}

/// A directed graph
pub trait UndirectedNeighbor<NI: Idx> {
    /// Get the neighbors
    type NeighborsIterator<'a>: Iterator<Item = &'a NI>
    where
        Self: 'a,
        NI: 'a;

    /// Get the neighbor
    fn neighbors(&self, node: NI) -> Self::NeighborsIterator<'_>;
}

/// A directed graph
pub trait UndirectNeighborWithValues<NI: Idx, EV> {
    /// Get the neighbors
    type NeighborsIterator<'a>: Iterator<Item = &'a Target<NI, EV>>
    where
        Self: 'a,
        NI: 'a,
        EV: 'a;

    /// Get the neighbor
    fn neighbors_with_values(&self, node: NI) -> Self::NeighborsIterator<'_>;
}

type GraphResult<T> = Result<T, GraphError>;

/// A graph
pub trait Idx:
    Copy
    + std::ops::Add<Output = Self>
    + std::ops::AddAssign
    + std::ops::Sub<Output = Self>
    + std::ops::Div<Output = Self>
    + std::ops::Mul<Output = Self>
    + Ord
    + Debug
    + Send
    + Sum
    + Sync
    + Sized
    + 'static
{
    /// Create a new index
    fn new(value: usize) -> Self;
    /// Get the index
    fn index(self) -> usize;
    /// Get the zero value
    fn zero() -> Self;
    /// Get the value and increment
    fn get_and_increment(this: &mut Self) -> Self {
        let value = *this;
        *this += Self::new(1);
        value
    }
}

macro_rules! impl_idx {
    ($TYPE:ty) => {
        impl Idx for $TYPE {
            #[inline]
            fn new(idx: usize) -> Self {
                assert!(idx <= <$TYPE>::MAX as usize);
                idx as $TYPE
            }

            #[inline]
            fn zero() -> Self {
                0
            }

            #[inline]
            fn index(self) -> usize {
                self as usize
            }
        }
    };
}

impl_idx!(u8);
impl_idx!(u16);
impl_idx!(u32);
impl_idx!(u64);
impl_idx!(usize);

impl_idx!(i8);
impl_idx!(i16);
impl_idx!(i32);
impl_idx!(i64);
impl_idx!(isize);

/// A weighted edge
#[derive(Debug, PartialEq, Eq)]
pub struct Target<NI, EV> {
    /// Target node id
    pub target: NI,
    /// Edge value
    pub value: EV,
}

impl<NI, EV> Target<NI, EV> {
    /// Create a new weighted edge
    pub fn new(target: NI, value: EV) -> Self {
        Self { target, value }
    }
}

/// Converts a tuple into a weighted edge.
pub trait Edges {
    /// Node id
    type NI: Idx;
    /// Edge value
    type EV;
    /// Iterator over the edges
    type EdgeIter: Iterator<Item = (Self::NI, Self::NI, Self::EV)>;

    /// Get the edges
    fn edges(&self) -> Self::EdgeIter;
    /// Get the maximum node id
    fn max_node_id(&self) -> Self::NI;

    /// Get the degrees
    fn degrees(&self, node: Self::NI) -> Vec<Self::NI> {
        let mut degrees = Vec::with_capacity(node.index());
        degrees.resize(node.index(), Self::NI::zero());
        self.edges().for_each(|(src, _, _)| {
            Self::NI::get_and_increment(&mut degrees[src.index()]);
        });
        self.edges().for_each(|(_, dst, _)| {
            Self::NI::get_and_increment(&mut degrees[dst.index()]);
        });

        degrees
    }
}

/// A list of edges
pub struct EdgeList<NI: Idx, EV> {
    list: Vec<(NI, NI, EV)>,
}

impl<NI: Idx, EV> EdgeList<NI, EV> {
    /// Create a new edge list
    pub fn new(list: Vec<(NI, NI, EV)>) -> Self {
        Self { list }
    }
}

impl<NI: Idx> EdgeList<NI, ()> {
    /// Create a new unweighted edge list
    pub fn new_unweighted(list: Vec<(NI, NI)>) -> Self {
        Self {
            list: list.into_iter().map(|(src, dst)| (src, dst, ())).collect(),
        }
    }
}

impl<NI: Idx, EV: Copy> Edges for EdgeList<NI, EV> {
    type NI = NI;
    type EV = EV;
    type EdgeIter = std::vec::IntoIter<(Self::NI, Self::NI, Self::EV)>;

    fn edges(&self) -> Self::EdgeIter {
        self.list.clone().into_iter()
    }

    fn max_node_id(&self) -> Self::NI {
        self.list
            .iter()
            .map(|(src, dst, _)| std::cmp::max(*src, *dst))
            .max()
            .unwrap()
    }
}

fn sum<NI: Idx>(degrees: &Vec<NI>) -> Vec<NI> {
    let mut last = *degrees.last().unwrap();
    let mut sums = degrees
        .into_iter()
        .scan(NI::zero(), |total, degree| {
            let value = *total;
            *total += *degree;
            Some(value)
        })
        .collect::<Vec<_>>();
    last += *sums.last().unwrap();
    sums.push(last);
    sums
}

/// A CSR graph
pub struct CSRGraph<Index: Idx, NI, EV> {
    offsets: Vec<Index>,
    targets: Vec<Target<NI, EV>>,
}

impl<NI, EV, E> From<(&E, NI)> for CSRGraph<NI, NI, EV>
where
    NI: Idx,
    E: Edges<NI = NI, EV = EV>,
    EV: Copy,
{
    fn from((edge_list, node_count): (&E, NI)) -> Self {
        Self::from_sorted_edges(edge_list, node_count).unwrap()
    }
}

impl<Index: Idx, NI> CSRGraph<Index, NI, ()> {
    /// Create a new CSR graph
    pub fn targets(&self, node: Index) -> &[NI] {
        assert_eq!(
            std::mem::size_of::<Target<NI, ()>>(),
            std::mem::size_of::<NI>()
        );
        assert_eq!(
            std::mem::align_of::<Target<NI, ()>>(),
            std::mem::align_of::<NI>()
        );

        let from = self.offsets[node.index()];
        let to = self.offsets[node.index() + 1];
        let len = (to - from).index();

        let targets = &self.targets[from.index()..to.index()];
        unsafe { std::slice::from_raw_parts(targets.as_ptr() as *const _, len) }
    }
}

impl<Index: Idx, NI, EV> CSRGraph<Index, NI, EV> {
    /// Create a new CSR graph
    pub fn targets_with_values(&self, node: Index) -> &[Target<NI, EV>] {
        let from = self.offsets[node.index()];
        let to = self.offsets[(node + Index::new(1)).index()];
        &self.targets[from.index()..to.index()]
    }
    /// Create a new CSR graph
    pub fn node_count(&self) -> Index {
        Index::new(self.offsets.len() - 1)
    }

    /// Create a new CSR graph
    pub fn edge_count(&self) -> Index {
        Index::new(self.targets.len())
    }

    /// Create a new CSR graph
    pub fn from_sorted_edges<E>(edge_list: &E, node_count: NI) -> GraphResult<Self>
    where
        NI: Idx,
        E: Edges<NI = NI, EV = EV>,
        EV: Copy,
    {
        let degrees = edge_list.degrees(node_count);

        let mut offsets = sum(&degrees);
        let edge_count = offsets[node_count.index()].index();
        let mut targets: Vec<Target<NI, EV>> = Vec::with_capacity(edge_count);
        let target_ptr = targets.as_mut_ptr();

        // These two loops assume we have an Undirected graph
        // SAFETY: We are writing to the targets vector, which is initialized with the correct size
        for (s, t, v) in edge_list.edges() {
            let offset = NI::get_and_increment(&mut offsets[s.index()]);
            unsafe {
                target_ptr
                    .add(offset.index())
                    .write(Target::new(t, v.clone()));
            }
        }
        for (s, t, v) in edge_list.edges() {
            let offset = NI::get_and_increment(&mut offsets[t.index()]);
            unsafe {
                target_ptr
                    .add(offset.index())
                    .write(Target::new(s, v.clone()));
            }
        }

        // SAFETY: We have initialized the targets vector with the correct size
        unsafe {
            targets.set_len(edge_count);
        }

        offsets.rotate_right(1);
        offsets[0] = NI::zero();
        let offsets: Vec<Index> = offsets.into_iter().map(|e| Index::new(e.index())).collect();

        Ok(Self { offsets, targets })
    }
}

/// Node values
pub struct NodeValues<NV> {
    values: Vec<NV>,
}

impl<NV> NodeValues<NV> {
    /// Create a new node values
    pub fn new(values: Vec<NV>) -> Self {
        Self { values }
    }

    fn get(&self, node: usize) -> &NV {
        &self.values[node]
    }
}

/// A graph
pub struct UndirectedCSRGraph<NI: Idx, NV = (), EV = ()> {
    csr: CSRGraph<NI, NI, EV>,
    node_values: NodeValues<NV>,
}

impl<NI: Idx, NV, EV> UndirectedCSRGraph<NI, NV, EV> {
    /// Create a new graph
    pub fn new(csr: CSRGraph<NI, NI, EV>, node_values: NodeValues<NV>) -> Self {
        Self { csr, node_values }
    }
}

impl<NI: Idx, NV, EV> Graph<NI> for UndirectedCSRGraph<NI, NV, EV> {
    fn node_count(&self) -> NI {
        self.csr.node_count()
    }

    fn edge_count(&self) -> NI {
        self.csr.edge_count() / NI::new(2)
    }
}

impl<NI: Idx, NV, EV> UndirectNeighborWithValues<NI, EV> for UndirectedCSRGraph<NI, NV, EV> {
    type NeighborsIterator<'a> = std::slice::Iter<'a, Target<NI, EV>> where NV:'a, EV: 'a, NI: 'a;

    fn neighbors_with_values(&self, node: NI) -> Self::NeighborsIterator<'_> {
        if node >= self.node_count() {
            return [].iter();
        }
        self.csr.targets_with_values(node).iter()
    }
}

impl<NI: Idx, NV, EV> NodeValueTrait<NI, NV> for UndirectedCSRGraph<NI, NV, EV> {
    fn node_value(&self, node: NI) -> &NV {
        self.node_values.get(node.index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn from_sorted_edges() {
        let edge_list = EdgeList::new(vec![
            (0, 1, 1),
            (0, 2, 1),
            (1, 2, 1),
            (1, 3, 1),
            (2, 3, 1),
            (3, 1, 0),
        ]);
        let graph = CSRGraph::from((&edge_list, 4));
        let graph = UndirectedCSRGraph::new(graph, NodeValues::new(vec![1, 2, 3, 4]));

        assert_eq!(graph.node_count(), 4);
        assert_eq!(graph.edge_count(), 6);

        let n0 = graph.neighbors_with_values(0).as_slice();
        let n1 = graph.neighbors_with_values(1).as_slice();

        assert!(n0 == &[Target::new(1, 1), Target::new(2, 1)]);
        assert!(
            n1 == &[
                Target {
                    target: 2,
                    value: 1
                },
                Target {
                    target: 3,
                    value: 1
                },
                Target {
                    target: 0,
                    value: 1
                },
                Target {
                    target: 3,
                    value: 0
                }
            ],
            "{:?}",
            n1
        );

        let n0v = graph.node_value(0);
        assert!(n0v == &1);
        let n1v = graph.node_value(1);
        assert!(n1v == &2);
        let n2v = graph.node_value(2);
        assert!(n2v == &3);
        let n3v = graph.node_value(3);
        assert!(n3v == &4);
    }
}
