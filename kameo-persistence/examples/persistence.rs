use kameo::prelude::*;
use kameo_persistence::PersistentActor;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, warn};
use url::Url;
use uuid::Uuid;

// Manager actor using Args as snapshot (for custom snapshot, use #[snapshot(CustomType)])
#[derive(Debug, Clone, PersistentActor)]
pub struct ManagerActor {
    pub config: String,
    pub sub_actors: HashMap<String, ActorRef<SubActor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerArgs {
    pub config: String,
    pub sub_actors: HashMap<String, Url>,
}

impl From<&ManagerActor> for ManagerArgs {
    fn from(actor: &ManagerActor) -> Self {
        Self {
            config: actor.config.clone(),
            sub_actors: actor
                .sub_actors
                .iter()
                .filter_map(|(name, actor_ref)| {
                    PersistentActor::persistence_key(actor_ref).map(|url| (name.clone(), url))
                })
                .collect(),
        }
    }
}

impl Actor for ManagerActor {
    type Args = ManagerArgs;
    type Error = anyhow::Error;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        let mut sub_actors = HashMap::new();

        for (name, url) in args.sub_actors {
            if let Ok(sub_actor) = SubActor::respawn_persistent(url).await {
                sub_actors.insert(name, sub_actor);
            }
        }

        Ok(Self {
            config: args.config,
            sub_actors,
        })
    }
}

#[derive(Debug, Clone, Actor, Serialize, Deserialize, PersistentActor)]
pub struct SubActor {
    pub data: String,
}

impl From<&SubActor> for SubActor {
    fn from(actor: &SubActor) -> Self {
        actor.clone()
    }
}

// Messages
impl Message<AddSubActor> for ManagerActor {
    type Reply = anyhow::Result<ActorRef<SubActor>>;

    async fn handle(
        &mut self,
        msg: AddSubActor,
        ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        let Some(key) = PersistentActor::persistence_key(&ctx.actor_ref()) else {
            debug!("ManagerActor is not persistent, building non-persistent sub-actor");

            return Ok(SubActor::spawn(SubActor { data: msg.data }));
        };

        let sub_key = key.join("sub-actors")?.join(&Uuid::new_v4().to_string())?;

        let Ok(sub_actor) = SubActor::spawn_persistent(
            sub_key.clone(),
            SubActor {
                data: msg.data.clone(),
            },
        )
        .await
        else {
            warn!(
                "Failed to spawn persistent sub-actor for key: {sub_key}, spawning non-persistent sub-actor instead"
            );
            return Ok(SubActor::spawn(SubActor { data: msg.data }));
        };

        Ok(sub_actor)
    }
}

impl Message<GetConfig> for ManagerActor {
    type Reply = String;

    async fn handle(
        &mut self,
        _msg: GetConfig,
        _ctx: &mut Context<Self, Self::Reply>,
    ) -> Self::Reply {
        self.config.clone()
    }
}

pub struct AddSubActor {
    pub name: String,
    pub data: String,
}

pub struct GetConfig;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manager_key = Url::parse("file:///tmp/manager")?;

    // Try to restore or create new manager
    let manager = ManagerActor::try_respawn_persistent(
        manager_key,
        ManagerArgs {
            config: "Default Config".to_string(),
            sub_actors: HashMap::new(),
        },
    )
    .await?;

    // Add a sub-actor
    manager
        .tell(AddSubActor {
            name: "worker1".to_string(),
            data: "Worker data".to_string(),
        })
        .await?;

    // Get config
    let config = manager.ask(GetConfig).await?;
    println!("Manager config: {}", config);

    Ok(())
}
