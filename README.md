# kameo-persistence

Persistence add-on for [Kameo](https://github.com/tqwewe/kameo) providing snapshot-based backup and restore of actors.

## Installation

```bash
cargo add kameo-persistence
```

## Quick Start

```rust
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use kameo::prelude::*;
use kameo_persistence::PersistentActor;
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PersistentActor)]
pub struct ManagerActor {
    pub data: String,
    pub sub_actors: HashMap<String, ActorRef<SubActor>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManagerActorArgs {
    pub data: String,
    pub sub_actors: HashMap<String, Url>,
}

impl From<&ManagerActor> for ManagerActorArgs {
    fn from(actor: &ManagerActor) -> Self {
        Self {
            data: actor.data.clone(),
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
    type Args = ManagerActorArgs;
    type Error = anyhow::Error;

    async fn on_start(args: Self::Args, _actor_ref: ActorRef<Self>) -> Result<Self, Self::Error> {
        Ok(Self {
            data: args.data,
            sub_actors: args
                .sub_actors
                .into_iter()
                .filter_map(|(name, url)| {
                    SubActor::respawn_persistent(url)
                        .await
                        .ok()
                        .map(|actor_ref| (name, actor_ref))
                })
                .collect()?,
        })
    }
}

#[derive(Debug, Clone, Actor, Serialize, Deserialize, PersistentActor)]
pub struct SubActor {
    pub config: String,
}

impl From<&SubActor> for SubActor {
    fn from(actor: &SubActor) -> Self {
        actor.clone()
    }
}
```

## Key Methods

- `PersistentActor` - A trait provides methods for persistent actors (derivable)
  - `spawn_persistent(key, args)` - Create a new persistent actor
  - `respawn_persistent(key)` - Restore an actor from snapshot
  - `try_respawn_persistent(key, args)` - Restore or create a new actor if not found
  - `save_snapshot(actor_ref)` - Save the current state of the actor
  - `persistence_key(actor_ref)` - Get the persistence key for the actor

## Storage

Currently supports file-based storage using URLs like `file:///path/to/snapshot`. However, HTTP(s), WebScockets, or Aws S3 like storages will be supported in the future.

## Examples

See `examples/` directory for detailed usage including:
- Manager actors with sub-actors
- Custom snapshot types
- Message handling with auto-save

## License

MIT