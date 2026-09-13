use std::marker::PhantomData;

use array_list::ArrayList;

pub trait ArenaId: Clone + Copy + std::fmt::Debug {
    fn from_u32(id: u32) -> Self;
    fn as_u32(self) -> u32;
}

pub struct Arena<Item, Id: ArenaId, const SIZE: usize = 64> {
    nodes: ArrayList<Item, SIZE>,
    _id: PhantomData<Id>,
}

impl<Item, Id: ArenaId> Arena<Item, Id> {
    pub const fn new() -> Self {
        Self {
            nodes: ArrayList::new(),
            _id: PhantomData,
        }
    }

    #[allow(clippy::cast_possible_truncation)]
    pub fn add(&mut self, item: Item) -> Id {
        self.nodes.push_back(item);
        Id::from_u32(self.nodes.len() as u32)
    }

    /// # Panics
    /// An attempt to get an element that does not exist
    /// will result in a panic
    #[allow(clippy::panic)]
    pub fn get(&self, id: Id) -> &Item {
        self.nodes
            .get(id.as_u32() as usize)
            .unwrap_or_else(|| panic!("Element with {id:?} does not exist"))
    }
}

impl<Item, Id: ArenaId> Default for Arena<Item, Id> {
    fn default() -> Self {
        Self::new()
    }
}
/*

*/

pub enum Type {
    Atom(String),
    String,
    Int,
    Bool,
    Float,
    Map(TypeId, TypeId),
    Record(Vec<(String, TypeId)>),
    Tuple(Vec<TypeId>),
    List(TypeId),

    Variable(TypeId),

    Union(Vec<TypeId>),
    Intersection(Vec<TypeId>),
}

impl Type {
    pub fn is_never(&self, arena: &Arena<Self, TypeId>) -> bool {
        // match self {
        //     Self::Union([]) => true,
        //     Self::Intersection(nodes) => nodes
        //         .iter()
        //         .reduce(|left, right| left.unify(right))
        //         .map(|value| value.is_never())
        //         .unwrap_or(false),

        //     _ => false,
        // }

        todo!()
    }

    // when self & other == other / self ⊆ other
    pub fn is_subtype(&self, other: &Self) -> bool {
        todo!()
    }

    // (number) | (int) -> Union[number, int]
    #[must_use]
    pub fn unify(&self, other: &Self) -> Self {
        todo!()
    }
}

pub enum Pattern {
    Tuple(Vec<()>),
}

pub struct MatchPattern {
    // How to handle type narrowing?
    pattern: Pattern,
    body: NodeId,
}

pub enum Node {
    Producer {
        constraint: TypeId,
    },
    Match {
        input: NodeId,
        patterns: Vec<MatchPattern>,
    },
    Application {
        callee: NodeId,
        arguments: Vec<NodeId>,
    },
    Consumer {
        from: NodeId,
        constraint: TypeId,
    },
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct NodeId(u32);

impl ArenaId for NodeId {
    fn from_u32(id: u32) -> Self {
        Self(id)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TypeId(u32);

impl ArenaId for TypeId {
    fn from_u32(id: u32) -> Self {
        Self(id)
    }

    fn as_u32(self) -> u32 {
        self.0
    }
}

pub struct TypeEngine {
    types: Arena<Type, TypeId>,
    nodes: Arena<Node, NodeId>,
}

impl TypeEngine {
    pub const fn new() -> Self {
        Self {
            types: Arena::new(),
            nodes: Arena::new(),
        }
    }
}

impl Default for TypeEngine {
    fn default() -> Self {
        Self::new()
    }
}
