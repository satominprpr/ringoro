use std::marker::PhantomData;

use async_trait::async_trait;

use crate::{
    context::Context,
    fcomps::{
        behavior::{Behave, BehaveDef},
        service::{CRUDHook, HookResult},
        validate::{Validate, ValidateRefDef},
        Callable,
        Deny,
        //Through,
    },
    mongo::withid::WithId,
    stores::User,
    utils::{result::Result, simple_error},
};

pub struct Wrap<T> {
    pub value: T,
}

impl<T> Wrap<T> {
    pub fn new(value: T) -> Self {
        Self { value }
    }
}

impl<T> HookResult for Wrap<T> {}

pub type AuthInfo = Option<WithId<User>>;

pub struct AuthHookBehavior {}

#[async_trait(?Send)]
impl BehaveDef for AuthHookBehavior {
    type In = ();
    type Out = Wrap<AuthInfo>;
    type Ctx = Context;

    #[allow(clippy::unit_arg)]
    async fn def(_: (), ctx: &Context) -> Result<Self::Out> {
        Ok(Wrap::new(ctx.user.clone()))
    }
}

pub struct OnlyLoggedInDef;
impl ValidateRefDef for OnlyLoggedInDef {
    type T = Wrap<AuthInfo>;

    fn def(input: &Self::T) -> Result<()> {
        if input.value.is_some() {
            Ok(())
        } else {
            Err(simple_error!("user not logged in"))
        }
    }
}

pub struct OnlyAdminDef;
impl ValidateRefDef for OnlyAdminDef {
    type T = Wrap<AuthInfo>;

    fn def(input: &Self::T) -> Result<()> {
        if input.value.is_some() {
            Ok(())
        } else {
            Err(simple_error!("invalid user"))
        }
    }
}

pub struct AuthHook<OnCreate, OnUpdate, OnDelete, OnFindOne, OnFindMany> {
    #[allow(clippy::type_complexity)]
    p: PhantomData<fn() -> (OnCreate, OnUpdate, OnDelete, OnFindOne, OnFindMany)>,
}

impl<OnCreate, OnUpdate, OnDelete, OnFindOne, OnFindMany> CRUDHook
    for AuthHook<OnCreate, OnUpdate, OnDelete, OnFindOne, OnFindMany>
where
    OnCreate: Callable<In = Wrap<AuthInfo>, Out = Wrap<AuthInfo>>,
    OnUpdate: Callable<In = Wrap<AuthInfo>, Out = Wrap<AuthInfo>>,
    OnDelete: Callable<In = Wrap<AuthInfo>, Out = Wrap<AuthInfo>>,
    OnFindOne: Callable<In = Wrap<AuthInfo>, Out = Wrap<AuthInfo>>,
    OnFindMany: Callable<In = Wrap<AuthInfo>, Out = Wrap<AuthInfo>>,
{
    type Ctx = Context;
    type HookOut = Wrap<AuthInfo>;
    type Hook = Behave<AuthHookBehavior>;
    type OnCreate = OnCreate;
    type OnUpdate = OnUpdate;
    type OnDelete = OnDelete;
    type OnFindOne = OnFindOne;
    type OnFindMany = OnFindMany;
}

pub type OnlyLoggedIn = Validate<OnlyLoggedInDef>;
//pub type AllowAll = Through<Wrap<AuthInfo>>;
pub type DenyAll = Deny<Wrap<AuthInfo>>;
pub type OnlyAdmin = Validate<OnlyAdminDef>;
