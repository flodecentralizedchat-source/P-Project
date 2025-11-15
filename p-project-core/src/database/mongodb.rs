use super::Database as DbTrait;
use crate::models::{Proposal, ProposalExecutionType, ProposalStatus};
use async_trait::async_trait;
use chrono::NaiveDateTime;
use futures_util::TryStreamExt;
use mongodb::{bson::DateTime, Client};
use serde::{Deserialize, Serialize};

pub struct MongoDatabase {
    database: mongodb::Database,
}

#[async_trait]
impl DbTrait for MongoDatabase {
    type Error = mongodb::error::Error;

    async fn save_proposal(&self, proposal: &Proposal) -> Result<(), Self::Error> {
        let collection = self.database.collection::<ProposalDocument>("proposals");
        let doc = ProposalDocument {
            id: proposal.id.clone(),
            title: proposal.title.clone(),
            description: proposal.description.clone(),
            creator_id: proposal.creator_id.clone(),
            created_at: proposal.created_at,
            voting_end_time: proposal.voting_end_time,
            status: proposal.status.clone(),
            execution_type: proposal.execution_type.clone(),
            execution_data: proposal.execution_data.clone(),
            executed_at: proposal
                .executed_at
                .map(|dt| DateTime::from_millis(dt.and_utc().timestamp_millis())),
        };
        collection.insert_one(doc, None).await?;
        Ok(())
    }

    async fn get_active_proposals(&self) -> Result<Vec<Proposal>, Self::Error> {
        let collection = self.database.collection::<ProposalDocument>("proposals");
        let filter = mongodb::bson::doc! { "status": "Active" };
        let cursor = collection.find(filter, None).await?;
        let proposals: Vec<Proposal> = cursor
            .try_collect::<Vec<ProposalDocument>>()
            .await?
            .into_iter()
            .map(|doc| Proposal {
                id: doc.id,
                title: doc.title,
                description: doc.description,
                creator_id: doc.creator_id,
                created_at: doc.created_at,
                voting_end_time: doc.voting_end_time,
                status: doc.status,
                execution_type: doc.execution_type,
                execution_data: doc.execution_data,
                executed_at: doc
                    .executed_at
                    .and_then(|dt| NaiveDateTime::from_timestamp_millis(dt.timestamp_millis())),
            })
            .collect();
        Ok(proposals)
    }

    async fn get_proposal(&self, proposal_id: &str) -> Result<Option<Proposal>, Self::Error> {
        let collection = self.database.collection::<ProposalDocument>("proposals");
        let filter = mongodb::bson::doc! { "id": proposal_id };
        let doc = collection.find_one(filter, None).await?;

        match doc {
            Some(doc) => Ok(Some(Proposal {
                id: doc.id,
                title: doc.title,
                description: doc.description,
                creator_id: doc.creator_id,
                created_at: doc.created_at,
                voting_end_time: doc.voting_end_time,
                status: doc.status,
                execution_type: doc.execution_type,
                execution_data: doc.execution_data,
                executed_at: doc
                    .executed_at
                    .and_then(|dt| NaiveDateTime::from_timestamp_millis(dt.timestamp_millis())),
            })),
            None => Ok(None),
        }
    }

    async fn update_proposal(&self, proposal: &Proposal) -> Result<(), Self::Error> {
        let collection = self.database.collection::<ProposalDocument>("proposals");
        let filter = mongodb::bson::doc! { "id": &proposal.id };
        let update = mongodb::bson::doc! {
            "$set": {
                "status": serde_json::to_string(&proposal.status).map_err(|e| mongodb::error::Error::custom(e.to_string()))?,
                "executed_at": proposal.executed_at.map(|dt| DateTime::from_millis(dt.and_utc().timestamp_millis())),
            }
        };
        collection.update_one(filter, update, None).await?;
        Ok(())
    }
}

impl MongoDatabase {
    pub async fn new(
        connection_string: &str,
        db_name: &str,
    ) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(connection_string).await?;
        let database = client.database(db_name);
        Ok(Self { database })
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProposalDocument {
    id: String,
    title: String,
    description: String,
    creator_id: String,
    created_at: NaiveDateTime,
    voting_end_time: NaiveDateTime,
    status: ProposalStatus,
    execution_type: Option<ProposalExecutionType>,
    execution_data: Option<String>,
    executed_at: Option<DateTime>,
}
