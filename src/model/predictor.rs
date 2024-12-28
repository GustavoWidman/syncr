use diesel::prelude::*;

use crate::data::entities::BaseEntity;
use crate::data::entities::predictor::PredictorSave;
use serde::{Deserialize, Serialize};

// SUPER UTILS
use super::node::{NodeList, TreeNode};
use super::utils::{default_block_size, naivify_file_size};

#[derive(Serialize, Deserialize)]
pub struct CompressionTree {
    nodes: NodeList,
    naive_nodes: NodeList,
}

impl CompressionTree {
    //? Constructor Methods
    pub fn new() -> Self {
        Self {
            nodes: NodeList::new(),
            naive_nodes: NodeList::new(),
        }
    }
    pub fn load(conn: &mut SqliteConnection) -> Result<Self, anyhow::Error> {
        match PredictorSave::find_by_id(1, conn)? {
            Some(model) => Ok(Self::deserialize(&model.save)?),
            None => Ok(Self::new()),
        }
    }

    //? Serde Methods
    pub fn serialize(&self) -> Result<Vec<u8>, anyhow::Error> {
        Ok(bincode::serialize(&self)?)
    }
    pub fn deserialize(data: &[u8]) -> Result<Self, anyhow::Error> {
        Ok(bincode::deserialize(data)?)
    }

    //? Persistence
    pub async fn save(&self, conn: &mut SqliteConnection) -> Result<(), anyhow::Error> {
        use crate::schema::predictor_saves::dsl::*;

        match PredictorSave::find_by_id(1, conn)? {
            Some(old) => {
                old.update(conn, (save.eq(self.serialize()?),))?;
            }
            None => {
                PredictorSave::quick_insert(self.serialize()?, conn);
            }
        }

        Ok(())
    }

    //? Model Usage
    pub fn wonderful_predict(&mut self, file_size: usize) -> u32 {
        self.nodes
            .wonderful_find(file_size) // 50/50
            .unwrap_or_else(|| {
                // predict naively
                self.naive_nodes
                    .wonderful_find(naivify_file_size(file_size)) // 50/50
                    .unwrap_or(default_block_size(file_size))
            })
    }
    pub fn predict(&mut self, file_size: usize) -> u32 {
        self.nodes
            .find(file_size) // no wondering
            .unwrap_or_else(|| {
                // predict naively
                self.naive_nodes
                    .find(naivify_file_size(file_size))
                    .unwrap_or(default_block_size(file_size))
            })
    }

    pub fn tune(&mut self, file_size: usize, block_size: u32, rate: f32) -> anyhow::Result<()> {
        let node = TreeNode::new(file_size, rate, block_size, false);
        let naive = node
            .naive()
            .ok_or(anyhow::anyhow!("Could not naivify node"))?;

        self.nodes.push(node)?;
        self.naive_nodes.push(naive)?;

        Ok(())
    }
}
