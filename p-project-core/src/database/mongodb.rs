use crate::models::{Proposal, ProposalStatus};
use futures_util::TryStreamExt;
use mongodb::{Client, Database};
use serde::{Deserialize, Serialize};

pub struct MongoDatabase {
    database: Database,
}

impl MongoDatabase {
    pub async fn new(connection_string: &str, db_name: &str) -> Result<Self, mongodb::error::Error> {
        let client = Client::with_uri_str(connection_string).await?;
        let database = client.database(db_name);
        Ok(Self { database })
    }
    
    pub async fn save_proposal(&self, proposal: &Proposal) -> Result<(), mongodb::error::Error> {
        let collection = self.database.collection::<ProposalDocument>("proposals");
        let doc = ProposalDocument {
            id: proposal.id.clone(),
            title: proposal.title.clone(),
            description: proposal.description.clone(),
            creator_id: proposal.creator_id.clone(),
            created_at: proposal.created_at,
            voting_end_time: proposal.voting_end_time,
            status: proposal.status.clone(),
        };
        collection.insert_one(doc, None).await?;
        Ok(())
    }
    
    pub async fn get_active_proposals(&self) -> Result<Vec<Proposal>, mongodb::error::Error> {
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
            })
            .collect();
        Ok(proposals)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ProposalDocument {
    id: String,
    title: String,
    description: String,
    creator_id: String,
    created_at: chrono::NaiveDateTime,
    voting_end_time: chrono::NaiveDateTime,
    status: ProposalStatus,
}