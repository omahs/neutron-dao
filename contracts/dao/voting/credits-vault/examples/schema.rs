use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, export_schema_with_title, remove_schemas, schema_for};
use cosmwasm_std::Addr;
use credits_vault::msg::{ExecuteMsg, InstantiateMsg, ListBondersResponse, MigrateMsg, QueryMsg};
use credits_vault::state::Config;
use cw_controllers::ClaimsResponse;
use cwd_interface::voting::{
    InfoResponse, IsActiveResponse, TotalPowerAtHeightResponse, VotingPowerAtHeightResponse,
};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(MigrateMsg), &out_dir);

    export_schema(&schema_for!(InfoResponse), &out_dir);
    export_schema(&schema_for!(TotalPowerAtHeightResponse), &out_dir);
    export_schema(&schema_for!(VotingPowerAtHeightResponse), &out_dir);
    export_schema(&schema_for!(IsActiveResponse), &out_dir);
    export_schema(&schema_for!(ClaimsResponse), &out_dir);
    export_schema(&schema_for!(ListBondersResponse), &out_dir);

    // Auto TS code generation expects the query return type as QueryNameResponse
    // Here we map query resonses to the correct name
    export_schema_with_title(&schema_for!(Addr), &out_dir, "DaoResponse");
    export_schema_with_title(&schema_for!(Config), &out_dir, "GetConfigResponse");
}