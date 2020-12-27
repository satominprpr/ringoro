use mongodm::doc;
use serde::{Deserialize, Serialize};

use crate::{
    context::Context,
    fcomps::{
        behavior::PanicBehave,
        convert::Convertible,
        service::{CRUDBehaviors, CRUDSevice, FromHookResult, SimpleCRUDServiceDef},
    },
    mongo::{
        service::{
            DeleteId, FindManyArgument, FindOneArgument, WithIdDeleteBehavior,
            WithIdFindManyBehavior, WithIdFindOneBehavior,
        },
        withid::{ConvertModelWithIdCursor, Id, WithId},
    },
    services::auth_hook::*,
    stores::{User, UserRepository},
    utils::{result::Result, simple_error},
};

#[derive(Deserialize, Debug)]
pub struct UserFindOneInput {
    pub id: Option<Id>,
}

#[derive(Serialize, Debug)]
pub struct UserOutput {
    pub id: Id,
    pub name: String,
}

impl Convertible<UserOutput> for WithId<User> {
    fn convert(self) -> Result<UserOutput> {
        Ok(UserOutput {
            id: self.0,
            name: self.1.name,
        })
    }
}

impl FromHookResult<Wrap<AuthInfo>, UserFindOneInput> for FindOneArgument {
    fn from_hook_result(
        Wrap { value: user }: Wrap<AuthInfo>,
        input: UserFindOneInput,
    ) -> Result<FindOneArgument> {
        if let Some(user) = user {
            if user.1.is_admin() {
                Ok(FindOneArgument(
                    doc! {
                        "_id": input.id.unwrap_or(user.0)
                    },
                    None,
                ))
            } else if input.id.is_none() {
                Ok(FindOneArgument(
                    doc! {
                        "_id": user.0
                    },
                    None,
                ))
            } else {
                Err(simple_error!("auth error"))
            }
        } else {
            Err(simple_error!("unexpected"))
        }
    }
}

impl FromHookResult<Wrap<AuthInfo>, UserFindOneInput> for DeleteId {
    fn from_hook_result(
        Wrap { value: user }: Wrap<AuthInfo>,
        input: UserFindOneInput,
    ) -> Result<DeleteId> {
        if let Some(user) = user {
            if user.1.is_admin() {
                Ok(DeleteId(input.id.unwrap_or(user.0)))
            } else if input.id.is_none() {
                Ok(DeleteId(user.0))
            } else {
                Err(simple_error!("auth error"))
            }
        } else {
            Err(simple_error!("unexpected"))
        }
    }
}

impl FromHookResult<Wrap<AuthInfo>, ()> for FindManyArgument {
    fn from_hook_result(Wrap { value: user }: Wrap<AuthInfo>, _: ()) -> Result<FindManyArgument> {
        if let Some(user) = user {
            if user.1.is_admin() {
                return Ok(FindManyArgument(doc! {}, None));
            }
        }
        Err(simple_error!("unexpected"))
    }
}

pub struct UserBehavior;
impl CRUDBehaviors for UserBehavior {
    type Ctx = Context;
    type CreateIn = ();
    type UpdateIn = ();
    type DeleteIn = UserFindOneInput;
    type FindOneIn = UserFindOneInput;
    type FindManyIn = ();
    type FindOneOut = Option<UserOutput>;
    type FindManyOut = ConvertModelWithIdCursor<UserOutput, User>;
    type Create = PanicBehave<(), WithId<User>, Context>;
    type Update = PanicBehave<(), WithId<User>, Context>;
    type Delete = WithIdDeleteBehavior<UserRepository>;
    type FindOne = WithIdFindOneBehavior<UserRepository>;
    type FindMany = WithIdFindManyBehavior<UserRepository>;
}

pub type UserService = CRUDSevice<
    SimpleCRUDServiceDef<
        UserBehavior,
        AuthHook<DenyAll, DenyAll, OnlyLoggedIn, OnlyLoggedIn, OnlyAdmin>,
    >,
>;

#[cfg(test)]
mod test {
    use futures::TryStreamExt;
    use mongodb::bson::Document;
    use pretty_assertions::assert_eq;

    use super::*;
    use crate::mongo::{context::Context as MongoContext, test_util, withid::RepositoryWithId};

    fn dummy_ctx(ctx: &MongoContext) -> Context {
        Context::new(ctx.clone(), None)
    }

    async fn repo(ctx: &MongoContext) -> UserRepository {
        UserRepository::new(&dummy_ctx(ctx)).await
    }

    async fn save_user(user: &User, ctx: &MongoContext) -> WithId<User> {
        let repo = repo(ctx).await;
        let id = repo.create(&user).await.unwrap();
        repo.find_one_by_id(&id).await.unwrap().unwrap()
    }

    async fn create_user(name: impl AsRef<str>, ctx: &MongoContext) -> WithId<User> {
        save_user(&User::new(String::from(name.as_ref())), ctx).await
    }

    async fn create_admin_user(name: impl AsRef<str>, ctx: &MongoContext) -> WithId<User> {
        save_user(&User::new_admin_user(String::from(name.as_ref())), ctx).await
    }

    async fn list(doc: Document, ctx: &MongoContext) -> Vec<WithId<User>> {
        repo(ctx)
            .await
            .find_many(doc, None)
            .await
            .unwrap()
            .try_collect::<Vec<WithId<User>>>()
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_crate_with_deny() {
        test_util::with_mongo(|ctx| async move {
            let _ = UserService::create((), &Context::new(ctx.as_ref().clone(), None))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_update_with_deny() {
        test_util::with_mongo(|ctx| async move {
            let _ = UserService::update((), &Context::new(ctx.as_ref().clone(), None))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_delete_with_valid_user() {
        test_util::with_mongo(|ctx| async move {
            let user = create_user("akari", ctx.as_ref()).await;
            let input = UserFindOneInput { id: None };
            UserService::delete(input, &Context::new(ctx.as_ref().clone(), Some(user)))
                .await
                .unwrap();
            assert_eq!(0, list(doc! {}, ctx.as_ref()).await.len());
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_delete_with_admin_user() {
        test_util::with_mongo(|ctx| async move {
            let user = create_user("akari", ctx.as_ref()).await;
            let admin = create_admin_user("akira", ctx.as_ref()).await;
            let input = UserFindOneInput { id: Some(user.0) };
            UserService::delete(input, &Context::new(ctx.as_ref().clone(), Some(admin)))
                .await
                .unwrap();
            let list = list(doc! {}, ctx.as_ref()).await;
            assert_eq!(1, list.len());
            assert_eq!("akira", list[0].1.name);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_delete_with_nologin() {
        test_util::with_mongo(|ctx| async move {
            let _ = create_user("akari", ctx.as_ref()).await;
            let input = UserFindOneInput { id: None };
            let _ = UserService::delete(input, &Context::new(ctx.as_ref().clone(), None))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_delete_with_not_admin_user() {
        test_util::with_mongo(|ctx| async move {
            let user = create_user("akari", ctx.as_ref()).await;
            let user1 = create_user("akira", ctx.as_ref()).await;
            let input = UserFindOneInput { id: Some(user1.0) };
            let _ = UserService::delete(input, &Context::new(ctx.as_ref().clone(), Some(user)))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_one_with_valid_user() {
        test_util::with_mongo(|ctx| async move {
            let user = create_user("akari", ctx.as_ref()).await;
            let input = UserFindOneInput { id: None };
            let result =
                UserService::find_one(input, &Context::new(ctx.as_ref().clone(), Some(user)))
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!("akari", result.name);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_one_with_admin_user() {
        test_util::with_mongo(|ctx| async move {
            let user = create_user("akari", ctx.as_ref()).await;
            let admin = create_admin_user("akira", ctx.as_ref()).await;
            let input = UserFindOneInput { id: Some(user.0) };
            let result =
                UserService::find_one(input, &Context::new(ctx.as_ref().clone(), Some(admin)))
                    .await
                    .unwrap()
                    .unwrap();
            assert_eq!("akari", result.name);
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_one_with_nologin() {
        test_util::with_mongo(|ctx| async move {
            let _ = create_user("akari", ctx.as_ref()).await;
            let input = UserFindOneInput { id: None };
            let _ = UserService::find_one(input, &Context::new(ctx.as_ref().clone(), None))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_one_with_not_admin_user() {
        test_util::with_mongo(|ctx| async move {
            let user = create_user("akari", ctx.as_ref()).await;
            let user1 = create_user("akira", ctx.as_ref()).await;
            let input = UserFindOneInput { id: Some(user1.0) };
            let _ = UserService::find_one(input, &Context::new(ctx.as_ref().clone(), Some(user)))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_many_with_admin_user() {
        test_util::with_mongo(|ctx| async move {
            let _ = create_user("akari", ctx.as_ref()).await;
            let admin = create_admin_user("akira", ctx.as_ref()).await;
            let result =
                UserService::find_many((), &Context::new(ctx.as_ref().clone(), Some(admin)))
                    .await
                    .unwrap()
                    .try_collect::<Vec<UserOutput>>()
                    .await
                    .unwrap();
            assert_eq!(2, result.len());
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_many_with_not_admin_user() {
        test_util::with_mongo(|ctx| async move {
            let user = create_user("akari", ctx.as_ref()).await;
            let _ = create_admin_user("akira", ctx.as_ref()).await;
            let _ = UserService::find_many((), &Context::new(ctx.as_ref().clone(), Some(user)))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_find_many_with_no_login() {
        test_util::with_mongo(|ctx| async move {
            let _ = create_user("akari", ctx.as_ref()).await;
            let _ = create_admin_user("akira", ctx.as_ref()).await;
            let _ = UserService::find_many((), &Context::new(ctx.as_ref().clone(), None))
                .await
                .unwrap_err();
            Ok(())
        })
        .await
        .unwrap()
    }
}
