mod common;

use std::time::Duration;

use common::{LabkeyError, Mock, MockServer, ResponseTemplate, fixture, test_client};
#[cfg(feature = "internal-test-support")]
use labkey_rs::client::__internal_test_support;
use labkey_rs::common::AuditBehavior;
use labkey_rs::filter::Filter;
use labkey_rs::query::{
    CommandType, DeleteRowsOptions, ExecuteSqlOptions, GetQueryDetailsOptions, InsertRowsOptions,
    MoveRowsOptions, SaveRowsCommand, SaveRowsOptions, SelectDistinctOptions, SelectRowsOptions,
    TruncateTableOptions, UpdateRowsOptions,
};
use url::Url;
use wiremock::matchers::{
    basic_auth, body_json, body_string_contains, header, header_exists, method, path, query_param,
};

#[tokio::test]
async fn select_rows_sends_expected_get_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/MyProject/MyFolder/query-getQuery.api"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("query.queryName", "People"))
        .and(query_param("apiVersion", "17.1"))
        .and(query_param("query.sort", "Name"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "rowCount": 0,
            "rows": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .select_rows(
            SelectRowsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .sort("Name".to_string())
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.row_count, 0);
}

#[tokio::test]
async fn execute_sql_sends_expected_post_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/query-executeSql.api"))
        .and(query_param("query.sort", "DisplayName"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string_contains("\"schemaName\":\"core\""))
        .and(body_string_contains("\"apiVersion\":17.1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "core",
            "rowCount": 0,
            "rows": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .execute_sql(
            ExecuteSqlOptions::builder()
                .schema_name("core".to_string())
                .sql("SELECT DisplayName FROM core.users".to_string())
                .sort("DisplayName".to_string())
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.row_count, 0);
}

#[tokio::test]
async fn select_distinct_rows_sends_expected_get_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-selectDistinct.api"))
        .and(query_param("dataRegionName", "query"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("query.queryName", "People"))
        .and(query_param("query.columns", "Gender"))
        .and(query_param("query.showRows", "all"))
        .and(query_param("query.param.Site", "A"))
        .and(query_param("query.ignoreFilter", "1"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "values": ["F", "M"]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .select_distinct_rows(
            SelectDistinctOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .column("Gender".to_string())
                .container_path("/Alt/Container".to_string())
                .max_rows(-1)
                .ignore_filter(true)
                .parameters(std::iter::once(("Site".to_string(), "A".to_string())).collect())
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.values.len(), 2);
}

#[tokio::test]
async fn get_query_details_sends_expected_get_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-getQueryDetails.api"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("queryName", "People"))
        .and(query_param("fields", "Name"))
        .and(query_param("viewName", "All"))
        .and(query_param("fk", "CreatedBy"))
        .and(query_param("initializeMissingView", "true"))
        .and(query_param("includeTriggers", "false"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "name": "People",
            "columns": [{"name": "RowId", "fieldKey": "RowId"}],
            "views": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_query_details(
            GetQueryDetailsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .fields(vec!["Name".to_string()])
                .view_name(vec!["All".to_string()])
                .fk("CreatedBy".to_string())
                .initialize_missing_view(true)
                .include_triggers(false)
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.schema_name, "lists");
    assert_eq!(response.name, "People");
}

#[tokio::test]
async fn select_distinct_rows_supports_custom_data_region_and_positive_max_rows() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-selectDistinct.api"))
        .and(query_param("dataRegionName", "grid"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("grid.queryName", "People"))
        .and(query_param("grid.columns", "Gender"))
        .and(query_param("maxRows", "10"))
        .and(query_param("grid.param.Site", "A"))
        .and(query_param("grid.Status~eq", "Active"))
        .and(query_param("grid.ignoreFilter", "1"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "values": ["F"]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .select_distinct_rows(
            SelectDistinctOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .column("Gender".to_string())
                .container_path("/Alt/Container".to_string())
                .data_region_name("grid".to_string())
                .max_rows(10)
                .ignore_filter(true)
                .parameters(std::iter::once(("Site".to_string(), "A".to_string())).collect())
                .filter_array(vec![Filter::equal("Status", "Active")])
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.values, vec![serde_json::json!("F")]);
}

#[tokio::test]
async fn get_query_details_supports_multiple_fields_and_view_names() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-getQueryDetails.api"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("queryName", "People"))
        .and(query_param("fields", "Name"))
        .and(query_param("fields", "Status"))
        .and(query_param("viewName", "All"))
        .and(query_param("viewName", "Mine"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "name": "People",
            "columns": [{"name": "RowId", "fieldKey": "RowId"}],
            "views": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_query_details(
            GetQueryDetailsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .fields(vec!["Name".to_string(), "Status".to_string()])
                .view_name(vec!["All".to_string(), "Mine".to_string()])
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.schema_name, "lists");
    assert_eq!(response.name, "People");
}

#[tokio::test]
async fn json_error_body_maps_to_api_error() {
    let server = MockServer::start().await;
    let error_body: serde_json::Value = fixture("api_error.json");

    Mock::given(method("GET"))
        .and(path("/MyProject/MyFolder/query-getQuery.api"))
        .respond_with(ResponseTemplate::new(400).set_body_json(error_body))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .select_rows(
            SelectRowsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("Missing".to_string())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::Api { status, body }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
            assert_eq!(
                body.exception.as_deref(),
                Some("Query 'Missing' in schema 'lists' doesn't exist.")
            );
        }
        other => panic!("expected LabkeyError::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn non_json_error_body_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/MyProject/MyFolder/query-getQuery.api"))
        .respond_with(ResponseTemplate::new(502).set_body_string("gateway exploded"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .select_rows(
            SelectRowsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
            assert_eq!(text, "gateway exploded");
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
}

#[tokio::test]
#[cfg(feature = "internal-test-support")]
async fn timeout_helper_exercises_request_options_timeout_path() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/MyProject/MyFolder/query-getQuery.api"))
        .respond_with(
            ResponseTemplate::new(200)
                .set_delay(Duration::from_millis(200))
                .set_body_json(serde_json::json!({
                    "schemaName": "lists",
                    "rowCount": 0,
                    "rows": []
                })),
        )
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let url = Url::parse(&format!(
        "{}/MyProject/MyFolder/query-getQuery.api",
        server.uri()
    ))
    .expect("valid mock URL");

    let result = __internal_test_support::get_with_timeout::<serde_json::Value>(
        &client,
        url,
        &[],
        Duration::from_millis(25),
    )
    .await;

    match result {
        Err(LabkeyError::Http(error)) => {
            assert!(error.is_timeout());
        }
        other => panic!("expected timeout HTTP error, got {other:?}"),
    }
}

#[tokio::test]
#[cfg(feature = "internal-test-support")]
async fn multipart_helper_sends_form_parts() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/query-importData.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(header_exists("content-type"))
        .and(body_string_contains("name=\"schemaName\""))
        .and(body_string_contains("lists"))
        .and(body_string_contains("name=\"queryName\""))
        .and(body_string_contains("People"))
        .and(body_string_contains("name=\"file\"; filename=\"rows.csv\""))
        .and(body_string_contains("Name,Age"))
        .and(body_string_contains("Alice,30"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "ok": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let url = Url::parse(&format!(
        "{}/MyProject/MyFolder/query-importData.api",
        server.uri()
    ))
    .expect("valid mock URL");

    let file_part = reqwest::multipart::Part::text("Name,Age\nAlice,30")
        .file_name("rows.csv")
        .mime_str("text/csv")
        .expect("valid mime type");
    let form = reqwest::multipart::Form::new()
        .text("schemaName", "lists")
        .text("queryName", "People")
        .part("file", file_part);

    let response = __internal_test_support::post_multipart::<serde_json::Value>(&client, url, form)
        .await
        .expect("multipart request should succeed");

    assert_eq!(response.get("ok"), Some(&serde_json::Value::Bool(true)));
}

#[tokio::test]
async fn insert_rows_sends_expected_mutation_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-insertRows.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "rows": [{"Name": "Alice"}],
            "auditBehavior": "SUMMARY"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "command": "insert",
            "errors": [],
            "queryName": "People",
            "rows": [{"RowId": 7, "Name": "Alice"}],
            "rowsAffected": 1,
            "schemaName": "lists"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .insert_rows(
            InsertRowsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .rows(vec![serde_json::json!({"Name": "Alice"})])
                .container_path("/Alt/Container".to_string())
                .audit_behavior(AuditBehavior::Summary)
                .build(),
        )
        .await
        .expect("insert rows should succeed");

    assert_eq!(result.command, "insert");
    assert_eq!(result.rows_affected, 1);
}

#[tokio::test]
async fn update_rows_sends_expected_mutation_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-updateRows.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "rows": [{"RowId": 1, "Name": "Alicia"}],
            "skipReselectRows": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "command": "update",
            "errors": [],
            "queryName": "People",
            "rows": [{"RowId": 1, "Name": "Alicia"}],
            "rowsAffected": 1,
            "schemaName": "lists"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .update_rows(
            UpdateRowsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .rows(vec![serde_json::json!({"RowId": 1, "Name": "Alicia"})])
                .container_path("/Alt/Container".to_string())
                .skip_reselect_rows(true)
                .build(),
        )
        .await
        .expect("update rows should succeed");

    assert_eq!(result.command, "update");
    assert_eq!(result.rows_affected, 1);
}

#[tokio::test]
async fn delete_rows_supports_empty_rows_payload() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-deleteRows.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "rows": []
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "command": "delete",
            "errors": [],
            "queryName": "People",
            "rows": [],
            "rowsAffected": 0,
            "schemaName": "lists"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .delete_rows(
            DeleteRowsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .rows(Vec::new())
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("delete rows should succeed");

    assert_eq!(result.command, "delete");
    assert_eq!(result.rows_affected, 0);
}

#[tokio::test]
async fn truncate_table_sends_required_fields_without_rows() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-truncateTable.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "transacted": true,
            "auditBehavior": "NONE"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "command": "truncate",
            "errors": [],
            "queryName": "People",
            "rows": [],
            "rowsAffected": 0,
            "schemaName": "lists"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .truncate_table(
            TruncateTableOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .transacted(true)
                .audit_behavior(AuditBehavior::None)
                .build(),
        )
        .await
        .expect("truncate table should succeed");

    assert_eq!(result.command, "truncate");
    assert_eq!(result.rows_affected, 0);
}

#[tokio::test]
async fn move_rows_sends_expected_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-moveRows.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "targetContainerPath": "/Target/Folder",
            "rows": [{"RowId": 1}],
            "auditBehavior": "SUMMARY"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "command": "update",
            "errors": [],
            "queryName": "People",
            "rows": [{"RowId": 1}],
            "rowsAffected": 1,
            "schemaName": "lists",
            "success": true,
            "containerPath": "/Target/Folder",
            "updateCounts": {"rows": 1}
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .move_rows(
            MoveRowsOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .target_container_path("/Target/Folder".to_string())
                .rows(vec![serde_json::json!({"RowId": 1})])
                .container_path("/Alt/Container".to_string())
                .audit_behavior(AuditBehavior::Summary)
                .build(),
        )
        .await
        .expect("move rows should succeed");

    assert!(result.success);
    assert_eq!(result.result.rows_affected, 1);
}

#[tokio::test]
async fn save_rows_supports_empty_commands() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-saveRows.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "commands": [],
            "containerPath": "/Alt/Container",
            "validateOnly": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "committed": true,
            "errorCount": 0,
            "result": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .save_rows(
            SaveRowsOptions::builder()
                .commands(Vec::new())
                .container_path("/Alt/Container".to_string())
                .validate_only(true)
                .build(),
        )
        .await
        .expect("save rows should succeed");

    assert!(result.committed);
    assert_eq!(result.error_count, 0);
    assert!(result.result.is_empty());
}

#[tokio::test]
async fn save_rows_sends_command_wire_values() {
    let server = MockServer::start().await;

    let command = SaveRowsCommand::builder()
        .command(CommandType::Delete)
        .schema_name("lists".to_string())
        .query_name("People".to_string())
        .rows(vec![serde_json::json!({"RowId": 1})])
        .build();

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-saveRows.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "commands": [{
                "command": "delete",
                "schemaName": "lists",
                "queryName": "People",
                "rows": [{"RowId": 1}]
            }],
            "containerPath": "/Alt/Container"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "committed": true,
            "errorCount": 0,
            "result": [{
                "command": "delete",
                "errors": [],
                "queryName": "People",
                "rows": [{"RowId": 1}],
                "rowsAffected": 1,
                "schemaName": "lists"
            }]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .save_rows(
            SaveRowsOptions::builder()
                .commands(vec![command])
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("save rows should succeed");

    assert!(result.committed);
    assert_eq!(result.result.len(), 1);
    assert_eq!(result.result[0].command, "delete");
}
