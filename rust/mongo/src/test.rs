use std::marker::PhantomData;

use async_trait::async_trait;
use mongodm::{
    doc, f, mongo::options::FindOptions, /*operator::*, */ CollectionConfig, Indexes, Model,
};
use pretty_assertions::assert_eq;
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{
    context::{Context, MongodmContext},
    fcomps::{
        behavior::{Behave, BehaveDef},
        convert::Convertible,
        service::{CRUDHook, FromHookResult, HookResult},
        validate,
    },
    service::{DeleteId, FindManyArgument, FindOneArgument, WithIdCRUDService},
    test_util::with_mongo,
    utils::{result::Result, simple_error},
    validate_uniqueness,
    withid::{
        CreateOrUpdate, Id, RepositoryWithId, RepositoryWithIdBase, ValidatedRepositoryWithId,
        Validator, WithId,
    },
};

#[derive(Serialize, Deserialize, Validate)]
struct TricoUnit {
    #[validate(length(max = 10))]
    name: String,
    cu: String,
    co: String,
    pa: String,
}

impl TricoUnit {
    fn new(
        name: impl AsRef<str>,
        cu: impl AsRef<str>,
        co: impl AsRef<str>,
        pa: impl AsRef<str>,
    ) -> Self {
        Self {
            name: String::from(name.as_ref()),
            cu: String::from(cu.as_ref()),
            co: String::from(co.as_ref()),
            pa: String::from(pa.as_ref()),
        }
    }
}

struct TricoUnitCfg {}

impl CollectionConfig for TricoUnitCfg {
    fn collection_name() -> &'static str {
        "TricoUnit"
    }

    fn indexes() -> Indexes {
        Indexes::new()
    }
}

impl Model for TricoUnit {
    type CollConf = TricoUnitCfg;
}

#[derive(Serialize, Deserialize, Validate)]
pub struct User {
    name: String,
}

impl HookResult for My<Option<User>> {}

pub struct UserCfg {}

impl CollectionConfig for UserCfg {
    fn collection_name() -> &'static str {
        "User"
    }

    fn indexes() -> Indexes {
        Indexes::new()
    }
}

impl Model for User {
    type CollConf = UserCfg;
}

pub struct My<T>(T);

struct TricoUnitInput {
    name: String,
    cu: String,
    co: String,
    pa: String,
}

impl TricoUnitInput {
    fn new(
        name: impl AsRef<str>,
        cu: impl AsRef<str>,
        co: impl AsRef<str>,
        pa: impl AsRef<str>,
    ) -> Self {
        Self {
            name: String::from(name.as_ref()),
            cu: String::from(cu.as_ref()),
            co: String::from(co.as_ref()),
            pa: String::from(pa.as_ref()),
        }
    }
}

impl FromHookResult<My<Option<User>>, TricoUnitInput> for TricoUnit {
    fn from_hook_result(_: My<Option<User>>, input: TricoUnitInput) -> Result<Self> {
        Ok(Self {
            name: input.name,
            cu: input.cu,
            co: input.co,
            pa: input.pa,
        })
    }
}

impl FromHookResult<My<Option<User>>, Id> for DeleteId {
    fn from_hook_result(_: My<Option<User>>, input: Id) -> Result<Self> {
        Ok(Self(input))
    }
}

#[derive(Debug)]
struct TricoUnitOutput {
    pub id: Id,
    pub name: String,
    pub cu: String,
    pub co: String,
    pub pa: String,
}

impl Convertible<TricoUnitOutput> for WithId<TricoUnit> {
    fn convert(self) -> Result<TricoUnitOutput> {
        Ok(TricoUnitOutput {
            id: self.0,
            name: self.1.name,
            cu: self.1.cu,
            co: self.1.co,
            pa: self.1.pa,
        })
    }
}

pub enum FindOneInput {
    Name(String),
}

impl FromHookResult<My<Option<User>>, FindOneInput> for FindOneArgument {
    fn from_hook_result(_: My<Option<User>>, input: FindOneInput) -> Result<Self> {
        let FindOneInput::Name(name) = input;
        Ok(Self(doc! { f!(name in TricoUnit): name }, None))
    }
}

#[allow(dead_code)]
enum Order {
    Id,
    Name,
}

pub struct FindManyInput {
    cu: String,
    order: Order,
}

impl FromHookResult<My<Option<User>>, FindManyInput> for FindManyArgument {
    fn from_hook_result(_: My<Option<User>>, input: FindManyInput) -> Result<Self> {
        let query = doc! { f!(cu in TricoUnit): input.cu };
        let builder = FindOptions::builder();
        let option = match input.order {
            Order::Name => builder.sort(doc! { f!(name in TricoUnit): 1}),
            Order::Id => builder.sort(doc! { "_id": 1}),
        }
        .build();
        Ok(Self(query, Some(option)))
    }
}

struct TricoUnitValidator {}

#[async_trait(?Send)]
impl Validator for TricoUnitValidator {
    type Model = TricoUnit;
    type Ctx = Context;

    async fn validate(
        cu: CreateOrUpdate,
        model: &'_ Self::Model,
        ctx: &'_ Self::Ctx,
    ) -> Result<()> {
        validate_uniqueness! (<TricoUnit, name>, cu, model, ctx);
        Ok(())
    }
}

type Repo = ValidatedRepositoryWithId<TricoUnit, Context, TricoUnitValidator>;

struct BeforeHookBehavior {}

#[async_trait(?Send)]
impl BehaveDef for BeforeHookBehavior {
    type In = ();
    type Out = My<Option<User>>;
    type Ctx = Context;

    #[allow(clippy::unit_arg)]
    async fn def(_: (), ctx: &Context) -> Result<Self::Out> {
        let repo = ctx.repo::<User>();
        Ok(My(repo.find_one(None, None).await?))
    }
}

trait Deny {
    const DENY: &'static str;
}

struct DenyIf<D: Deny> {
    p: PhantomData<D>,
}

impl<D: Deny> validate::ValidateRefDef for DenyIf<D> {
    type T = My<Option<User>>;

    fn def(input: &Self::T) -> Result<()> {
        if let Some(user) = &input.0 {
            if user.name == D::DENY {
                Err(simple_error!("invalid user"))
            } else {
                Ok(())
            }
        } else {
            Ok(())
        }
    }
}

struct NameIsCraete {}
impl Deny for NameIsCraete {
    const DENY: &'static str = "create";
}

struct NameIsUpdate {}
impl Deny for NameIsUpdate {
    const DENY: &'static str = "update";
}

struct NameIsDelete {}
impl Deny for NameIsDelete {
    const DENY: &'static str = "delete";
}

struct NameIsFindOne {}
impl Deny for NameIsFindOne {
    const DENY: &'static str = "find-one";
}

struct NameIsFindMany {}
impl Deny for NameIsFindMany {
    const DENY: &'static str = "find-many";
}

struct BeforeHook {}
impl CRUDHook for BeforeHook {
    type Ctx = Context;
    type HookOut = My<Option<User>>;
    type Hook = Behave<BeforeHookBehavior>;
    type OnCreate = validate::Validate<DenyIf<NameIsCraete>>;
    type OnUpdate = validate::Validate<DenyIf<NameIsUpdate>>;

    type OnDelete = validate::Validate<DenyIf<NameIsDelete>>;
    type OnFindOne = validate::Validate<DenyIf<NameIsFindOne>>;
    type OnFindMany = validate::Validate<DenyIf<NameIsFindMany>>;
}

type Service = WithIdCRUDService<
    Repo,
    TricoUnitInput,
    TricoUnitOutput,
    FindOneInput,
    FindManyInput,
    BeforeHook,
>;

type UserRepo = RepositoryWithIdBase<User, Context>;

async fn add_user(name: &str, ctx: &Context) {
    let repo = UserRepo::new(ctx).await;
    let _ = repo
        .create(&User {
            name: String::from(name),
        })
        .await;
}

#[tokio::test]
async fn test_create() {
    with_mongo(|ctx| async move {
        let input = TricoUnitInput::new("unibo", "akari", "akira", "riamu");
        let id = Service::create(input, ctx.as_ref()).await?;
        let value = Repo::new(ctx.as_ref())
            .await
            .find_one_by_id(&id)
            .await?
            .unwrap();
        assert_eq!("unibo", value.1.name);
        assert_eq!("akari", value.1.cu);
        assert_eq!("akira", value.1.co);
        assert_eq!("riamu", value.1.pa);
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_create_with_invalid_data() {
    with_mongo(|ctx| async move {
        let input = TricoUnitInput::new("theunitnameistoolog", "akari", "akira", "riamu");
        let _err = Service::create(input, ctx.as_ref()).await.unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_create_with_invalid_with_ctx_validator() {
    with_mongo(|ctx| async move {
        let repo = Repo::new(ctx.as_ref()).await;
        let model = TricoUnit::new("unibo", "", "", "");
        let _ = repo.create(&model).await.unwrap();
        let input = TricoUnitInput::new("unibo", "akari", "akira", "riamu");
        let _err = Service::create(input, ctx.as_ref()).await.unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_create_fail_on_hook() {
    with_mongo(|ctx| async move {
        add_user("create", ctx.as_ref()).await;
        let input = TricoUnitInput::new("unibo", "akari", "akira", "riamu");
        let _err = Service::create(input, ctx.as_ref()).await.unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_update() {
    with_mongo(|ctx| async move {
        let repo = Repo::new(ctx.as_ref()).await;
        let model = TricoUnit::new("unibo", "", "", "");
        let id = repo.create(&model).await.unwrap();
        let input = TricoUnitInput::new("unibo", "akari", "akira", "riamu");
        Service::update(WithId(id.clone(), input), ctx.as_ref()).await?;
        let value = Repo::new(ctx.as_ref())
            .await
            .find_one_by_id(&id)
            .await?
            .unwrap();
        assert_eq!("unibo", value.1.name);
        assert_eq!("akari", value.1.cu);
        assert_eq!("akira", value.1.co);
        assert_eq!("riamu", value.1.pa);
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_update_with_invalid_data() {
    with_mongo(|ctx| async move {
        let repo = Repo::new(ctx.as_ref()).await;
        let model = TricoUnit::new("unibo", "", "", "");
        let id = repo.create(&model).await.unwrap();
        let input = TricoUnitInput::new("theunitnameistoolog", "akari", "akira", "riamu");
        Service::update(WithId(id.clone(), input), ctx.as_ref())
            .await
            .unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_update_with_invalid_with_ctx_validator() {
    with_mongo(|ctx| async move {
        let repo = Repo::new(ctx.as_ref()).await;
        let model = TricoUnit::new("unibo", "", "", "");
        let _ = repo.create(&model).await.unwrap();
        let model = TricoUnit::new("", "", "", "");
        let id = repo.create(&model).await.unwrap();
        let input = TricoUnitInput::new("unibo", "akari", "akira", "riamu");
        Service::update(WithId(id.clone(), input), ctx.as_ref())
            .await
            .unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_update_fail_on_hook() {
    with_mongo(|ctx| async move {
        add_user("update", ctx.as_ref()).await;
        let repo = Repo::new(ctx.as_ref()).await;
        let model = TricoUnit::new("unibo", "", "", "");
        let id = repo.create(&model).await.unwrap();
        let input = TricoUnitInput::new("unibo", "akari", "akira", "riamu");
        let _err = Service::update(WithId(id.clone(), input), ctx.as_ref())
            .await
            .unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_delete() {
    with_mongo(|ctx| async move {
        let repo = Repo::new(ctx.as_ref()).await;
        let _ = repo
            .create(&TricoUnit::new("nw", "sakura", "izumi", "ako"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("ng", "uzuki", "rin", "mio"))
            .await
            .unwrap();
        let id = repo
            .create(&TricoUnit::new("unibo", "akari", "akira", "riamu"))
            .await
            .unwrap();
        Service::delete(id.clone(), ctx.as_ref()).await?;
        assert!(Repo::new(ctx.as_ref())
            .await
            .find_one_by_id(&id)
            .await?
            .is_none());
        let c = repo.repo.count_documents(None, None).await.unwrap();
        assert_eq!(2, c);
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_update_fail_on_delete() {
    with_mongo(|ctx| async move {
        add_user("delete", ctx.as_ref()).await;
        let repo = Repo::new(ctx.as_ref()).await;
        let id = repo
            .create(&TricoUnit::new("unibo", "akari", "akira", "riamu"))
            .await
            .unwrap();
        let _ = Service::delete(id.clone(), ctx.as_ref()).await.unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_find_one() {
    with_mongo(|ctx| async move {
        let repo = Repo::new(ctx.as_ref()).await;
        let _ = repo
            .create(&TricoUnit::new("nw", "sakura", "izumi", "ako"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("ng", "uzuki", "rin", "mio"))
            .await
            .unwrap();
        let id = repo
            .create(&TricoUnit::new("unibo", "akari", "akira", "riamu"))
            .await
            .unwrap();
        let input = FindOneInput::Name(String::from("unibo"));
        let value = Service::find_one(input, ctx.as_ref()).await?.unwrap();
        assert_eq!(id, value.id);
        assert_eq!("unibo", value.name);
        assert_eq!("akari", value.cu);
        assert_eq!("akira", value.co);
        assert_eq!("riamu", value.pa);
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_find_one_fail_on_hook() {
    with_mongo(|ctx| async move {
        add_user("find-one", ctx.as_ref()).await;
        let repo = Repo::new(ctx.as_ref()).await;
        let _ = repo
            .create(&TricoUnit::new("nw", "sakura", "izumi", "ako"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("ng", "uzuki", "rin", "mio"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("unibo", "akari", "akira", "riamu"))
            .await
            .unwrap();
        let input = FindOneInput::Name(String::from("unibo"));
        let _err = Service::find_one(input, ctx.as_ref()).await.unwrap_err();
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_find_many() {
    use futures::TryStreamExt;
    with_mongo(|ctx| async move {
        let repo = Repo::new(ctx.as_ref()).await;
        let _ = repo
            .create(&TricoUnit::new("nw", "sakura", "izumi", "ako"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("ng", "uzuki", "rin", "mio"))
            .await
            .unwrap();
        let unibo_id = repo
            .create(&TricoUnit::new("unibo", "akari", "akira", "riamu"))
            .await
            .unwrap();
        let bw_id = repo
            .create(&TricoUnit::new("bw", "akari", "tsukasa", "akira"))
            .await
            .unwrap();
        let input = FindManyInput {
            cu: String::from("akari"),
            order: Order::Name,
        };
        let result = Service::find_many(input, ctx.as_ref())
            .await
            .unwrap()
            .try_collect::<Vec<_>>()
            .await?;
        assert_eq!(2, result.len());
        assert_eq!(bw_id, result[0].id);
        assert_eq!(unibo_id, result[1].id);
        Ok(())
    })
    .await
    .unwrap()
}

#[tokio::test]
async fn test_find_many_fail_on_hook() {
    with_mongo(|ctx| async move {
        add_user("find-many", ctx.as_ref()).await;
        let repo = Repo::new(ctx.as_ref()).await;
        let _ = repo
            .create(&TricoUnit::new("nw", "sakura", "izumi", "ako"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("ng", "uzuki", "rin", "mio"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("unibo", "akari", "akira", "riamu"))
            .await
            .unwrap();
        let _ = repo
            .create(&TricoUnit::new("bw", "akari", "tsukasa", "akira"))
            .await
            .unwrap();
        let input = FindManyInput {
            cu: String::from("akari"),
            order: Order::Name,
        };
        assert!(Service::find_many(input, ctx.as_ref()).await.is_err());
        Ok(())
    })
    .await
    .unwrap()
}
