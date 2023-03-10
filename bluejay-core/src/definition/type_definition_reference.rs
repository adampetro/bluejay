use crate::definition::{
    EnumTypeDefinition, InputObjectTypeDefinition, InterfaceTypeDefinition, ObjectTypeDefinition,
    ScalarTypeDefinition, UnionTypeDefinition,
};
use crate::BuiltinScalarDefinition;
use std::marker::PhantomData;

#[derive(Debug)]
pub enum TypeDefinitionReference<
    CS: ScalarTypeDefinition,
    CSW: AsRef<CS>,
    O: ObjectTypeDefinition,
    OW: AsRef<O>,
    IO: InputObjectTypeDefinition,
    IOW: AsRef<IO>,
    E: EnumTypeDefinition,
    EW: AsRef<E>,
    U: UnionTypeDefinition,
    UW: AsRef<U>,
    I: InterfaceTypeDefinition,
    IW: AsRef<I>,
> {
    BuiltinScalarType(BuiltinScalarDefinition),
    CustomScalarType(CSW, PhantomData<CS>),
    ObjectType(OW, PhantomData<O>),
    InputObjectType(IOW, PhantomData<IO>),
    EnumType(EW, PhantomData<E>),
    UnionType(UW, PhantomData<U>),
    InterfaceType(IW, PhantomData<I>),
}

pub trait AbstractTypeDefinitionReference:
    Into<TypeDefinitionReferenceFromAbstract<Self>> + AsRef<TypeDefinitionReferenceFromAbstract<Self>>
{
    type CustomScalarTypeDefinition: ScalarTypeDefinition;
    type ObjectTypeDefinition: ObjectTypeDefinition;
    type InputObjectTypeDefinition: InputObjectTypeDefinition;
    type EnumTypeDefinition: EnumTypeDefinition;
    type UnionTypeDefinition: UnionTypeDefinition;
    type InterfaceTypeDefinition: InterfaceTypeDefinition;
    type WrappedCustomScalarTypeDefinition: AsRef<Self::CustomScalarTypeDefinition>;
    type WrappedObjectTypeDefinition: AsRef<Self::ObjectTypeDefinition>;
    type WrappedInputObjectTypeDefinition: AsRef<Self::InputObjectTypeDefinition>;
    type WrappedEnumTypeDefinition: AsRef<Self::EnumTypeDefinition>;
    type WrappedUnionTypeDefinition: AsRef<Self::UnionTypeDefinition>;
    type WrappedInterfaceTypeDefinition: AsRef<Self::InterfaceTypeDefinition>;
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        O: ObjectTypeDefinition,
        OW: AsRef<O>,
        IO: InputObjectTypeDefinition,
        IOW: AsRef<IO>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
        U: UnionTypeDefinition,
        UW: AsRef<U>,
        I: InterfaceTypeDefinition,
        IW: AsRef<I>,
    > AsRef<Self> for TypeDefinitionReference<CS, CSW, O, OW, IO, IOW, E, EW, U, UW, I, IW>
{
    fn as_ref(&self) -> &TypeDefinitionReference<CS, CSW, O, OW, IO, IOW, E, EW, U, UW, I, IW> {
        self
    }
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS> + Clone,
        O: ObjectTypeDefinition,
        OW: AsRef<O> + Clone,
        IO: InputObjectTypeDefinition,
        IOW: AsRef<IO> + Clone,
        E: EnumTypeDefinition,
        EW: AsRef<E> + Clone,
        U: UnionTypeDefinition,
        UW: AsRef<U> + Clone,
        I: InterfaceTypeDefinition,
        IW: AsRef<I> + Clone,
    > Clone for TypeDefinitionReference<CS, CSW, O, OW, IO, IOW, E, EW, U, UW, I, IW>
{
    fn clone(&self) -> Self {
        match self {
            Self::BuiltinScalarType(bstd) => Self::BuiltinScalarType(*bstd),
            Self::CustomScalarType(csw, _) => {
                Self::CustomScalarType(csw.clone(), Default::default())
            }
            Self::EnumType(etw, _) => Self::EnumType(etw.clone(), Default::default()),
            Self::ObjectType(otw, _) => Self::ObjectType(otw.clone(), Default::default()),
            Self::InterfaceType(itw, _) => Self::InterfaceType(itw.clone(), Default::default()),
            Self::UnionType(utw, _) => Self::UnionType(utw.clone(), Default::default()),
            Self::InputObjectType(iotd, _) => {
                Self::InputObjectType(iotd.clone(), Default::default())
            }
        }
    }
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        O: ObjectTypeDefinition,
        OW: AsRef<O>,
        IO: InputObjectTypeDefinition,
        IOW: AsRef<IO>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
        U: UnionTypeDefinition,
        UW: AsRef<U>,
        I: InterfaceTypeDefinition,
        IW: AsRef<I>,
    > AbstractTypeDefinitionReference
    for TypeDefinitionReference<CS, CSW, O, OW, IO, IOW, E, EW, U, UW, I, IW>
{
    type CustomScalarTypeDefinition = CS;
    type ObjectTypeDefinition = O;
    type InputObjectTypeDefinition = IO;
    type EnumTypeDefinition = E;
    type UnionTypeDefinition = U;
    type InterfaceTypeDefinition = I;
    type WrappedCustomScalarTypeDefinition = CSW;
    type WrappedObjectTypeDefinition = OW;
    type WrappedInputObjectTypeDefinition = IOW;
    type WrappedEnumTypeDefinition = EW;
    type WrappedUnionTypeDefinition = UW;
    type WrappedInterfaceTypeDefinition = IW;
}

impl<
        CS: ScalarTypeDefinition,
        CSW: AsRef<CS>,
        O: ObjectTypeDefinition,
        OW: AsRef<O>,
        IO: InputObjectTypeDefinition,
        IOW: AsRef<IO>,
        E: EnumTypeDefinition,
        EW: AsRef<E>,
        U: UnionTypeDefinition,
        UW: AsRef<U>,
        I: InterfaceTypeDefinition,
        IW: AsRef<I>,
    > TypeDefinitionReference<CS, CSW, O, OW, IO, IOW, E, EW, U, UW, I, IW>
{
    pub fn name(&self) -> &str {
        match self {
            Self::BuiltinScalarType(bsd) => bsd.name(),
            Self::CustomScalarType(cstd, _) => cstd.as_ref().name(),
            Self::ObjectType(otd, _) => otd.as_ref().name(),
            Self::InputObjectType(iotd, _) => iotd.as_ref().name(),
            Self::EnumType(etd, _) => etd.as_ref().name(),
            Self::UnionType(utd, _) => utd.as_ref().name(),
            Self::InterfaceType(itd, _) => itd.as_ref().name(),
        }
    }

    pub fn is_builtin(&self) -> bool {
        matches!(self, Self::BuiltinScalarType(_))
    }
}

pub type TypeDefinitionReferenceFromAbstract<T> = TypeDefinitionReference<
    <T as AbstractTypeDefinitionReference>::CustomScalarTypeDefinition,
    <T as AbstractTypeDefinitionReference>::WrappedCustomScalarTypeDefinition,
    <T as AbstractTypeDefinitionReference>::ObjectTypeDefinition,
    <T as AbstractTypeDefinitionReference>::WrappedObjectTypeDefinition,
    <T as AbstractTypeDefinitionReference>::InputObjectTypeDefinition,
    <T as AbstractTypeDefinitionReference>::WrappedInputObjectTypeDefinition,
    <T as AbstractTypeDefinitionReference>::EnumTypeDefinition,
    <T as AbstractTypeDefinitionReference>::WrappedEnumTypeDefinition,
    <T as AbstractTypeDefinitionReference>::UnionTypeDefinition,
    <T as AbstractTypeDefinitionReference>::WrappedUnionTypeDefinition,
    <T as AbstractTypeDefinitionReference>::InterfaceTypeDefinition,
    <T as AbstractTypeDefinitionReference>::WrappedInterfaceTypeDefinition,
>;
