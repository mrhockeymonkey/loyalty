use log::info;
use mongodb::bson::doc;
use mongodb::Collection;
use thiserror::Error;
use loyalty_core::{BasicStampCard, UserId};

pub struct MongoDbStampCardRepository {
    pub collection: Collection<BasicStampCard> // TODO should this be public?
}

#[derive(Debug, Error)]
pub enum StampCardRepositoryError {
    #[error("mongodb error")]
    MongoDbError(#[from] mongodb::error::Error)
}


impl MongoDbStampCardRepository {
    pub async fn get_or_create_card(&mut self, user_id: &UserId) -> Result<BasicStampCard, StampCardRepositoryError> {
        info!("Searching for card with user_id {}", user_id);
        let filter = doc! {
            "user_id": user_id.to_string()
        };
        let find = self.collection
            .find_one(filter, None)
            .await?;

        match find {
            Some(card) => {
                info!("Found existing card for user_id {}", user_id);
                Ok(card)
            },
            None => {
                info!("Creating new card for user_id {}", user_id);
                let new_card = BasicStampCard::new(user_id.clone());
                self.collection.insert_one(&new_card, None).await?;
                Ok(new_card)
            }
        }
    }

    // TODO Command Query Separation
    pub async fn stamp_card(&mut self, user_id: &UserId) -> Result<BasicStampCard, StampCardRepositoryError> {
        let user_card = self.get_or_create_card(user_id).await?;
        let stamped_card = user_card.with_stamp();
        let filter = doc! {
            "user_id": user_id.to_string() // todo extract method
        };

        self.collection.replace_one(filter, &stamped_card, None).await?;

        info!("Card for user_id {} now has {} stamps", user_id, stamped_card.stamps);
        Ok(stamped_card)
    }

    pub async fn reset_card(&mut self, user_id: &UserId) -> Result<(), StampCardRepositoryError> {
        let new_card = BasicStampCard::new(user_id.clone());
        let filter = doc! {
            "user_id": user_id.to_string() // todo extract method
        };

        self.collection.replace_one(filter, &new_card, None).await?;

        info!("Card for user_id {} has been reset", user_id);
        Ok(())
    }
}