use core::fmt::{ Debug, Display, Formatter };


//------------------------------------------------------------------------------
//  OptionAb
//------------------------------------------------------------------------------
pub enum OptionAb<A, B>
{
    A(A),
    B(B),
}

impl<T> OptionAb<T, T>
{
    pub fn take( self ) -> T
    {
        match self
        {
            OptionAb::A(value) | OptionAb::B(value) => value,
        }
    }
}

impl<A, B> OptionAb<A, B>
{
    pub fn as_ref( &self ) -> OptionAb<&A, &B>
    {
        match self
        {
            OptionAb::A(value) => OptionAb::A(value),
            OptionAb::B(value) => OptionAb::B(value),
        }
    }

    pub fn a( &self ) -> Option<&A>
    {
        match self
        {
            OptionAb::A(value) => Some(value),
            OptionAb::B(_) => None,
        }
    }

    pub fn b( &self ) -> Option<&B>
    {
        match self
        {
            OptionAb::A(_) => None,
            OptionAb::B(value) => Some(value),
        }
    }
}

impl<A: Debug, B: Debug> OptionAb<A, B>
{
    pub fn unwrap_a( self ) -> A
    {
        match self
        {
            OptionAb::A(value) => value,
            OptionAb::B(_) =>
            {
                panic!("expected OptionAb::A(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_b( self ) -> B
    {
        match self
        {
            OptionAb::A(_) =>
            {
                panic!("expected OptionAb::B(_) but found {:?}", self)
            },
            OptionAb::B(value) => value,
        }
    }
}

impl<A: Debug, B: Debug> Debug for OptionAb<A, B>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAb::A(value) => write!(f, "OptionAb::A({:?})", value),
            OptionAb::B(value) => write!(f, "OptionAb::B({:?})", value),
        }
    }
}

impl<A: Display, B: Display> Display for OptionAb<A, B>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAb::A(value) => write!(f, "{}", value),
            OptionAb::B(value) => write!(f, "{}", value),
        }
    }
}

impl<A: PartialEq, B: PartialEq> PartialEq for OptionAb<A, B>
{
    fn eq( &self, other: &Self ) -> bool
    {
        match (self, other)
        {
            (OptionAb::A(value), OptionAb::A(other)) if value == other => true,
            (OptionAb::B(value), OptionAb::B(other)) if value == other => true,
            _ => false,
        }
    }
}

impl<A: PartialEq, B: PartialEq> Eq for OptionAb<A, B> {}


//------------------------------------------------------------------------------
//  OptionAbc
//------------------------------------------------------------------------------
pub enum OptionAbc<A, B, C>
{
    A(A),
    B(B),
    C(C),
}

impl<T> OptionAbc<T, T, T>
{
    pub fn take( self ) -> T
    {
        match self
        {
            OptionAbc::A(value)
            | OptionAbc::B(value)
            | OptionAbc::C(value)
            => value,
        }
    }
}

impl<A, B, C> OptionAbc<A, B, C>
{
    pub fn as_ref( &self ) -> OptionAbc<&A, &B, &C>
    {
        match self
        {
            OptionAbc::A(value) => OptionAbc::A(value),
            OptionAbc::B(value) => OptionAbc::B(value),
            OptionAbc::C(value) => OptionAbc::C(value),
        }
    }

    pub fn a( &self ) -> Option<&A>
    {
        match self
        {
            OptionAbc::A(value) => Some(value),
            _ => None,
        }
    }

    pub fn b( &self ) -> Option<&B>
    {
        match self
        {
            OptionAbc::B(value) => Some(value),
            _ => None,
        }
    }

    pub fn c( &self ) -> Option<&C>
    {
        match self
        {
            OptionAbc::C(value) => Some(value),
            _ => None,
        }
    }
}

impl<A: Debug, B: Debug, C: Debug> OptionAbc<A, B, C>
{
    pub fn unwrap_a( self ) -> A
    {
        match self
        {
            OptionAbc::A(value) => value,
            _ =>
            {
                panic!("expected OptionAbc::A(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_b( self ) -> B
    {
        match self
        {
            OptionAbc::B(value) => value,
            _ =>
            {
                panic!("expected OptionAbc::B(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_c( self ) -> C
    {
        match self
        {
            OptionAbc::C(value) => value,
            _ =>
            {
                panic!("expected OptionAbc::C(_) but found {:?}", self)
            },
        }
    }
}

impl<A: Debug, B: Debug, C: Debug> Debug for OptionAbc<A, B, C>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAbc::A(value) => write!(f, "OptionAbc::A({:?})", value),
            OptionAbc::B(value) => write!(f, "OptionAbc::B({:?})", value),
            OptionAbc::C(value) => write!(f, "OptionAbc::C({:?})", value),
        }
    }
}

impl<A: Display, B: Display, C: Display> Display for OptionAbc<A, B, C>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAbc::A(value) => write!(f, "{}", value),
            OptionAbc::B(value) => write!(f, "{}", value),
            OptionAbc::C(value) => write!(f, "{}", value),
        }
    }
}

impl<A: PartialEq, B: PartialEq, C: PartialEq> PartialEq for OptionAbc<A, B, C>
{
    fn eq( &self, other: &Self ) -> bool
    {
        match (self, other)
        {
            (OptionAbc::A(value), OptionAbc::A(other)) if value == other =>
            {
                true
            },
            (OptionAbc::B(value), OptionAbc::B(other)) if value == other =>
            {
                true
            },
            (OptionAbc::C(value), OptionAbc::C(other)) if value == other =>
            {
                true
            },
            _ => false,
        }
    }
}

impl<A: PartialEq, B: PartialEq, C: PartialEq> Eq for OptionAbc<A, B, C> {}


//------------------------------------------------------------------------------
//  OptionAbcd
//------------------------------------------------------------------------------
pub enum OptionAbcd<A, B, C, D>
{
    A(A),
    B(B),
    C(C),
    D(D),
}

impl<T> OptionAbcd<T, T, T, T>
{
    pub fn take( self ) -> T
    {
        match self
        {
            OptionAbcd::A(value)
            | OptionAbcd::B(value)
            | OptionAbcd::C(value)
            | OptionAbcd::D(value)
            => value,
        }
    }
}

impl<A, B, C, D> OptionAbcd<A, B, C, D>
{
    pub fn as_ref( &self ) -> OptionAbcd<&A, &B, &C, &D>
    {
        match self
        {
            OptionAbcd::A(value) => OptionAbcd::A(value),
            OptionAbcd::B(value) => OptionAbcd::B(value),
            OptionAbcd::C(value) => OptionAbcd::C(value),
            OptionAbcd::D(value) => OptionAbcd::D(value),
        }
    }

    pub fn a( &self ) -> Option<&A>
    {
        match self
        {
            OptionAbcd::A(value) => Some(value),
            _ => None,
        }
    }

    pub fn b( &self ) -> Option<&B>
    {
        match self
        {
            OptionAbcd::B(value) => Some(value),
            _ => None,
        }
    }

    pub fn c( &self ) -> Option<&C>
    {
        match self
        {
            OptionAbcd::C(value) => Some(value),
            _ => None,
        }
    }

    pub fn d( &self ) -> Option<&D>
    {
        match self
        {
            OptionAbcd::D(value) => Some(value),
            _ => None,
        }
    }
}

impl<A: Debug, B: Debug, C: Debug, D: Debug> OptionAbcd<A, B, C, D>
{
    pub fn unwrap_a( self ) -> A
    {
        match self
        {
            OptionAbcd::A(value) => value,
            _ =>
            {
                panic!("expected OptionAbcd::A(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_b( self ) -> B
    {
        match self
        {
            OptionAbcd::B(value) => value,
            _ =>
            {
                panic!("expected OptionAbcd::B(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_c( self ) -> C
    {
        match self
        {
            OptionAbcd::C(value) => value,
            _ =>
            {
                panic!("expected OptionAbcd::C(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_d( self ) -> D
    {
        match self
        {
            OptionAbcd::D(value) => value,
            _ =>
            {
                panic!("expected OptionAbcd::D(_) but found {:?}", self)
            },
        }
    }
}

impl<A: Debug, B: Debug, C: Debug, D: Debug> Debug for OptionAbcd<A, B, C, D>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAbcd::A(value) => write!(f, "OptionAbcd::A({:?})", value),
            OptionAbcd::B(value) => write!(f, "OptionAbcd::B({:?})", value),
            OptionAbcd::C(value) => write!(f, "OptionAbcd::C({:?})", value),
            OptionAbcd::D(value) => write!(f, "OptionAbcd::D({:?})", value),
        }
    }
}

impl<A: Display, B: Display, C: Display, D: Display> Display
    for OptionAbcd<A, B, C, D>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAbcd::A(value) => write!(f, "{}", value),
            OptionAbcd::B(value) => write!(f, "{}", value),
            OptionAbcd::C(value) => write!(f, "{}", value),
            OptionAbcd::D(value) => write!(f, "{}", value),
        }
    }
}

impl<A: PartialEq, B: PartialEq, C: PartialEq, D: PartialEq> PartialEq
    for OptionAbcd<A, B, C, D>
{
    fn eq( &self, other: &Self ) -> bool
    {
        match (self, other)
        {
            (OptionAbcd::A(value), OptionAbcd::A(other)) if value == other =>
            {
                true
            },
            (OptionAbcd::B(value), OptionAbcd::B(other)) if value == other =>
            {
                true
            },
            (OptionAbcd::C(value), OptionAbcd::C(other)) if value == other =>
            {
                true
            },
            (OptionAbcd::D(value), OptionAbcd::D(other)) if value == other =>
            {
                true
            },
            _ => false,
        }
    }
}

impl<A: PartialEq, B: PartialEq, C: PartialEq, D: PartialEq> Eq
    for OptionAbcd<A, B, C, D> {}


//------------------------------------------------------------------------------
//  OptionAbcde
//------------------------------------------------------------------------------
pub enum OptionAbcde<A, B, C, D, E>
{
    A(A),
    B(B),
    C(C),
    D(D),
    E(E),
}

impl<T> OptionAbcde<T, T, T, T, T>
{
    pub fn take( self ) -> T
    {
        match self
        {
            OptionAbcde::A(value)
            | OptionAbcde::B(value)
            | OptionAbcde::C(value)
            | OptionAbcde::D(value)
            | OptionAbcde::E(value)
            => value,
        }
    }
}

impl<A, B, C, D, E> OptionAbcde<A, B, C, D, E>
{
    pub fn as_ref( &self ) -> OptionAbcde<&A, &B, &C, &D, &E>
    {
        match self
        {
            OptionAbcde::A(value) => OptionAbcde::A(value),
            OptionAbcde::B(value) => OptionAbcde::B(value),
            OptionAbcde::C(value) => OptionAbcde::C(value),
            OptionAbcde::D(value) => OptionAbcde::D(value),
            OptionAbcde::E(value) => OptionAbcde::E(value),
        }
    }

    pub fn a( &self ) -> Option<&A>
    {
        match self
        {
            OptionAbcde::A(value) => Some(value),
            _ => None,
        }
    }

    pub fn b( &self ) -> Option<&B>
    {
        match self
        {
            OptionAbcde::B(value) => Some(value),
            _ => None,
        }
    }

    pub fn c( &self ) -> Option<&C>
    {
        match self
        {
            OptionAbcde::C(value) => Some(value),
            _ => None,
        }
    }

    pub fn d( &self ) -> Option<&D>
    {
        match self
        {
            OptionAbcde::D(value) => Some(value),
            _ => None,
        }
    }

    pub fn e( &self ) -> Option<&E>
    {
        match self
        {
            OptionAbcde::E(value) => Some(value),
            _ => None,
        }
    }
}

impl<A: Debug, B: Debug, C: Debug, D: Debug, E: Debug>
    OptionAbcde<A, B, C, D, E>
{
    pub fn unwrap_a( self ) -> A
    {
        match self
        {
            OptionAbcde::A(value) => value,
            _ =>
            {
                panic!("expected OptionAbcde::A(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_b( self ) -> B
    {
        match self
        {
            OptionAbcde::B(value) => value,
            _ =>
            {
                panic!("expected OptionAbcde::B(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_c( self ) -> C
    {
        match self
        {
            OptionAbcde::C(value) => value,
            _ =>
            {
                panic!("expected OptionAbcde::C(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_d( self ) -> D
    {
        match self
        {
            OptionAbcde::D(value) => value,
            _ =>
            {
                panic!("expected OptionAbcde::D(_) but found {:?}", self)
            },
        }
    }

    pub fn unwrap_e( self ) -> E
    {
        match self
        {
            OptionAbcde::E(value) => value,
            _ =>
            {
                panic!("expected OptionAbcde::E(_) but found {:?}", self)
            },
        }
    }
}

impl<A: Debug, B: Debug, C: Debug, D: Debug, E: Debug> Debug
    for OptionAbcde<A, B, C, D, E>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAbcde::A(value) => write!(f, "OptionAbcde::A({:?})", value),
            OptionAbcde::B(value) => write!(f, "OptionAbcde::B({:?})", value),
            OptionAbcde::C(value) => write!(f, "OptionAbcde::C({:?})", value),
            OptionAbcde::D(value) => write!(f, "OptionAbcde::D({:?})", value),
            OptionAbcde::E(value) => write!(f, "OptionAbcde::E({:?})", value),
        }
    }
}

impl<A: Display, B: Display, C: Display, D: Display, E: Display> Display
    for OptionAbcde<A, B, C, D, E>
{
    fn fmt( &self, f: &mut Formatter<'_> ) -> Result<(), std::fmt::Error>
    {
        match self
        {
            OptionAbcde::A(value) => write!(f, "{}", value),
            OptionAbcde::B(value) => write!(f, "{}", value),
            OptionAbcde::C(value) => write!(f, "{}", value),
            OptionAbcde::D(value) => write!(f, "{}", value),
            OptionAbcde::E(value) => write!(f, "{}", value),
        }
    }
}

impl<A: PartialEq, B: PartialEq, C: PartialEq, D: PartialEq, E: PartialEq>
    PartialEq for OptionAbcde<A, B, C, D, E>
{
    fn eq( &self, other: &Self ) -> bool
    {
        match (self, other)
        {
            (OptionAbcde::A(value), OptionAbcde::A(other)) if value == other =>
            {
                true
            },
            (OptionAbcde::B(value), OptionAbcde::B(other)) if value == other =>
            {
                true
            },
            (OptionAbcde::C(value), OptionAbcde::C(other)) if value == other =>
            {
                true
            },
            (OptionAbcde::D(value), OptionAbcde::D(other)) if value == other =>
            {
                true
            },
            (OptionAbcde::E(value), OptionAbcde::E(other)) if value == other =>
            {
                true
            },
            _ => false,
        }
    }
}

impl<A: PartialEq, B: PartialEq, C: PartialEq, D: PartialEq, E: PartialEq> Eq
    for OptionAbcde<A, B, C, D, E> {}
