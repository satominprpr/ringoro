use std::marker::PhantomData;

use async_trait::async_trait;
use mongodm::{mongo::bson::doc, operator::ToObjectId, Model};
use serde::{Deserialize, Serialize};

use crate::{
    context::MongodmContext, fcomp::behavior::EffectDef, result::Result, validator::Validate,
};

#[async_trait(?Send)]
pub trait MongodmValidate {
    async fn validate_on_create(&self, ctx: &impl MongodmContext) -> Result<()>;
    async fn validate_on_update(&self, ctx: &impl MongodmContext) -> Result<()>;
}

pub struct CreateServiceDefine<'a, M, Ctx>
where
    M: Serialize + Deserialize<'a> + Validate + MongodmValidate,
    Ctx: MongodmContext,
{
    p: PhantomData<fn() -> (M, Ctx)>,
}

#[async_trait(?Send)]
impl<'a, M, Ctx> EffectDef for CreateServiceDefine<'a, M, Ctx>
where
    M: Serialize + Deserialize<'a> + Validate + MongodmValidate,
    Ctx: MongodmContext,
{
    type In = M;
    type Out = ();
    type Ctx = Ctx;

    async fn def(input: &Self::In, ctx: &Self::Ctx) -> Result<()> {
        input.validate()?;
        input.validate_on_create(ctx).await?;
        ctx.repo::<M>().insert_one(input, None).await?;
        Ok(())
    }
}

struct UpdateServiceDefine<'a, M, Ctx>
where
    M: Serialize + Deserialize<'a> + Validate + MongodmValidate,
    Ctx: MongodmContext,
{
    p: PhantomData<fn() -> (M, Ctx)>,
}

#[async_trait(?Send)]
impl<'a, M, Ctx> EffectDef for UpdateServiceDefine<'a, M, Ctx>
where
    M: Serialize + Deserialize<'a> + Validate + MongodmValidate,
    Ctx: MongodmContext,
{
    type In = (String, M);
    type Out = ();
    type Ctx = Ctx;

    async fn def(input: &Self::In, ctx: &Self::Ctx) -> Result<()> {
        input.1.validate()?;
        input.1.validate_on_create(ctx).await?;
        ctx.repo::<M>()
            .replace_one(doc! { "_id": {ToObjectId: &input.0}}, &input.1, None)
            .await?;
        Ok(())
    }
}
