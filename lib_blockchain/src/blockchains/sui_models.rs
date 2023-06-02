use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct Response {
    pub jsonrpc: String,
    pub result: Option<Result>,
    pub error: Option<ResultError>,
    pub id: Option<i32>
}

#[derive(Debug, Deserialize)]
pub struct ResultError{
    pub code: String,
    pub message: String,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct Result {
    pub digest: String,
    pub transaction: Transaction,
    pub rawTransaction: String,
    pub effects: Effects,
    pub objectChanges: Vec<ObjectChange>,
}

#[derive(Debug, Deserialize)]
pub struct Transaction {
    pub data: Data,
    pub txSignatures: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct Data {
    pub messageVersion: String,
    pub transaction: InnerTransaction,
    pub sender: String,
    pub gasData: GasData,
}

#[derive(Debug, Deserialize)]
pub struct Effects {
    pub messageVersion: String,
    pub status: Status,
    pub executedEpoch: String,
    pub gasUsed: GasUsed,
    pub transactionDigest: String,
    pub mutated: Vec<Mutated>,
    pub gasObject: GasObject,
    pub eventsDigest: String,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct GasUsed {
    pub computationCost: String,
    pub storageCost: String,
    pub storageRebate: String,
    pub nonRefundableStorageFee: String,
}

#[derive(Debug, Deserialize)]
pub struct Mutated {
    pub owner: Owner,
    pub reference: Reference,
}

#[derive(Debug, Deserialize)]
pub struct Owner {
    #[serde(rename = "AddressOwner")]
    pub address_owner: String,
}

#[derive(Debug, Deserialize)]
pub struct Reference {
    pub objectId: String,
    pub version: i32,
    pub digest: String,
}

#[derive(Debug, Deserialize)]
pub struct GasObject {
    pub owner: ObjectOwner,
    pub reference: Reference,
}

#[derive(Debug, Deserialize)]
pub struct ObjectOwner {
    #[serde(rename = "ObjectOwner")]
    pub object_owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ObjectChange {
    #[serde(rename = "type")]
    pub change_type: String,
    pub sender: String,
    pub recipient: Owner2,
    #[serde(rename = "objectType")]
    pub object_type: String,
    #[serde(rename = "objectId")]
    pub object_id: String,
    pub version: String,
    pub digest: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Owner2 {
    #[serde(rename = "AddressOwner")]
    pub address_owner: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct InnerTransaction {
    pub TransferObjects: Vec<Vec<Input>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Input {
    pub Input: u64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GasData {
    pub payment: Vec<Object2>,
    pub owner: String,
    pub price: String,
    pub budget: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Object2 {
    pub objectId: String,
    pub version: u64,
    pub digest: String,
}
