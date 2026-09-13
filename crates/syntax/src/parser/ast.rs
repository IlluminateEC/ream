pub enum Expression {
    BinaryOperation(Box<Self>, BinaryOperator, Box<Self>),
    UnaryOperation(UnaryOperator, Box<Self>),
    Literal(Literal),
    Match,
    Function {
        args: Vec<(String, Type)>,
        body: Box<Self>,
        return_type: Option<Type>,
    },
    FunctionCall {
        callee: Box<Self>,
        args: Vec<Self>,
    },

    VariableAccess(String),
    FieldAccess {
        on: Box<Self>,
        field: String,
    },
    IndexAccess {
        on: Box<Self>,
        index: Box<Self>,
    },
}

pub enum Type {
    /// `Int`, `String`, ...
    Named(String),
    /// `SomeType[Type, Type, Type]`
    GenericApplication(Box<Self>, Vec<Self>),

    Atom(String),

    /// `#{ key: Type, key: Type }`
    AtomMap(Vec<(String, Self)>),

    /// `T1 | T2 | T3 ...`
    Union(Vec<Self>),

    /// `T1 & T2 & T3 ...`
    Intersection(Vec<Self>),

    /// `!`
    Never,

    /// `*`
    Any,
}

pub enum Literal {
    Atom(String),
    Int(dashu_int::IBig),
    Float(f64),
    String(String),
    List(Vec<Expression>),
    Tuple(Vec<Expression>),
    Map(Vec<MapPair>),
    BitString(Vec<BitStringSegment>),
}

pub enum MapPair {
    /// atom: value
    AtomKey(String, Expression),
    /// `(expression): value`
    ExpressionKey(Expression, Expression),
    /// `..map`
    Spread(Expression),
}

pub enum BitStringSegment {
    Expression {
        body: Expression,
        encoding: BitStringEncoding,
    },
    /// ..expr
    Spread(Expression),
}

pub enum BitStringEncoding {
    /// :size#endianness#signedness
    Numeric {
        size: Option<Expression>,
        endianness: Endianness,
        signedness: Signedness,
    },

    /// #endianness#encoding
    String {
        encoding: StringEncoding,
        endianness: Option<Endianness>,
    },
}

pub enum StringEncoding {
    Ascii, // Extended ASCII / ISO-8859-1 (0-255)
    Utf8,
    Utf16,
    Utf32,
}

pub enum Signedness {
    Unsigned,
    Signed,
}

//<< "awawa"#utf8, some_int:16#be#signed, 0o744#be#unsigned >>
//                                        ^^^^^ ^^ ^^^^^^^^

// [1 , 2 , 3 , 4 , x + y] -> Literal::Array([...Int, Expression])

pub enum Endianness {
    Big,
    Native,
    Little,
}

pub enum UnaryOperator {
    Negative,
    LogicalNot,
    BitwiseNot,
}

pub enum BinaryOperator {
    Addition,
    Subtraction,
    Multiplication,
    Division,
    Modulus,

    BitwiseXOR,
    BitwiseAND,
    BitwiseOR,

    Pipe,

    LogicalAND,
    LogicalOR,

    GreaterThan,
    GreaterThanEq,
    LessThan,
    LessThanEq,
    EqualTo,
    NotEqualTo,
    Comparision,
}
