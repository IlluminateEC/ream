use crate::parser::ast::Expression;
/// left side of a pattern expression in a match statment
///
/// { _, A, B } =>
/// literal =>
/// [ _, 241, 244] =>
/// #{ tag: 0, length: _, value: _ } =>
/// #{ index: _, node: #{ children: _, nebo: :true } } =>
/// << length: 32#be, type:   >>
/// << length: 32#be, data: (length * 8)#utf8 >>
pub enum Pattern {
    /// `:atom`
    Atom(String),

    /// `Int(value)`
    NamedType {
        type_name: String,
        binding_name: String,
    },

    Map(Vec<MapPair>),

    /// `{ _,  }`
    Tuple(Vec<Self>),

    /// `name`
    Binding(String),
    /// `_`
    Ignore,
}

pub enum MapPair {
    AtomKey { name: String, binding: Pattern },
    ExpressionKey { key: Expression, binding: Pattern },
    Spread(Pattern),
}
