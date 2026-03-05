mod common;

use std::time::Duration;

use base64::Engine;
use common::{LabkeyError, Mock, MockServer, ResponseTemplate, fixture, test_client};
#[cfg(feature = "internal-test-support")]
use labkey_rs::client::__internal_test_support;
use labkey_rs::common::AuditBehavior;
use labkey_rs::filter::Filter;
use labkey_rs::query::{
    CommandType, DataViewType, DeleteQueryViewOptions, DeleteRowsOptions, ExecuteSqlOptions,
    GetDataAggregate, GetDataFilter, GetDataOptions, GetDataPivot, GetDataSort,
    GetDataSortDirection, GetDataSource, GetDataTransform, GetDataViewsOptions, GetQueriesOptions,
    GetQueryDetailsOptions, GetQueryViewsOptions, GetSchemasOptions, ImportDataOptions,
    ImportDataSource, InsertOption, InsertRowsOptions, MoveRowsOptions, SaveQueryViewsOptions,
    SaveRowsCommand, SaveRowsOptions, SaveSessionViewOptions, SelectDistinctOptions,
    SelectRowsOptions, TruncateTableOptions, UpdateRowsOptions, ValidateQueryOptions,
};
use labkey_rs::security::{
    AddGroupMembersOptions, CreateGroupOptions, CreateNewUserOptions, DeleteGroupOptions,
    DeletePolicyOptions, DeleteUserOptions, EnsureLoginOptions, GetFolderTypesOptions,
    GetGroupPermissionsOptions, GetGroupsForCurrentUserOptions, GetModulesOptions,
    GetPolicyOptions, GetReadableContainersOptions, GetRolesOptions, GetSecurableResourcesOptions,
    GetUserPermissionsOptions, GetUsersOptions, GetUsersWithPermissionsOptions, ImpersonateTarget,
    ImpersonateUserOptions, LogoutOptions, MoveContainerOptions, RemoveGroupMembersOptions,
    RenameGroupOptions, SavePolicyOptions, StopImpersonatingOptions, WhoAmIOptions,
};
use url::Url;
use wiremock::matchers::{
    basic_auth, body_json, body_string, body_string_contains, header, header_exists, method, path,
    query_param, query_param_is_missing,
};

fn waf_encode_for_test(value: &str) -> String {
    let url_encoded = urlencoding::encode(value);
    let b64 = base64::engine::general_purpose::STANDARD.encode(url_encoded.as_bytes());
    format!("/*{{{{base64/x-www-form-urlencoded/wafText}}}}*/{b64}")
}

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
async fn get_data_query_source_posts_expected_body_and_returns_rows() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-getData"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "source": {
                "type": "query",
                "schemaName": "lists",
                "queryName": "People"
            },
            "renderer": {
                "type": "json",
                "columns": [["Name"]],
                "includeDetailsColumn": true,
                "maxRows": 25,
                "offset": 5,
                "sort": [{
                    "fieldKey": ["Name"],
                    "dir": "DESC"
                }]
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "rowCount": 1,
            "rows": [{
                "data": {
                    "Name": { "value": "Alice" }
                }
            }]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_data(
            GetDataOptions::builder()
                .container_path("/Alt/Container".to_string())
                .source(GetDataSource::Query {
                    schema_name: "lists".to_string(),
                    query_name: "People".to_string(),
                })
                .columns(vec![vec!["Name".to_string()]])
                .include_details_column(true)
                .max_rows(25)
                .offset(5)
                .sort(vec![
                    GetDataSort::builder()
                        .field_key(vec!["Name".to_string()])
                        .dir(GetDataSortDirection::Desc)
                        .build(),
                ])
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.row_count, 1);
    assert_eq!(
        response.rows[0].data["Name"].value,
        serde_json::json!("Alice")
    );
}

#[tokio::test]
async fn get_data_posts_transforms_and_pivot_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-getData"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "source": {
                "type": "query",
                "schemaName": "lists",
                "queryName": "People"
            },
            "renderer": {
                "type": "json"
            },
            "transforms": [{
                "type": "aggregate",
                "groupBy": [["Department"]],
                "filters": [{
                    "fieldKey": ["Status"],
                    "type": "eq",
                    "value": "Active"
                }],
                "aggregates": [{
                    "fieldKey": ["Amount"],
                    "type": "sum"
                }]
            }],
            "pivot": {
                "by": ["Department"],
                "columns": [["Amount"]]
            }
        })))
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
        .get_data(
            GetDataOptions::builder()
                .container_path("/Alt/Container".to_string())
                .source(GetDataSource::Query {
                    schema_name: "lists".to_string(),
                    query_name: "People".to_string(),
                })
                .transforms(vec![
                    GetDataTransform::builder()
                        .type_("aggregate".to_string())
                        .group_by(vec![vec!["Department".to_string()]])
                        .filters(vec![
                            GetDataFilter::builder()
                                .field_key(vec!["Status".to_string()])
                                .type_("eq".to_string())
                                .value(serde_json::json!("Active"))
                                .build(),
                        ])
                        .aggregates(vec![
                            GetDataAggregate::builder()
                                .field_key(vec!["Amount".to_string()])
                                .type_("sum".to_string())
                                .build(),
                        ])
                        .build(),
                ])
                .pivot(
                    GetDataPivot::builder()
                        .by(vec!["Department".to_string()])
                        .columns(vec![vec!["Amount".to_string()]])
                        .build(),
                )
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.row_count, 0);
}

#[tokio::test]
async fn get_data_sql_source_waf_encodes_sql_and_omits_absent_optionals() {
    let server = MockServer::start().await;
    let sql = "SELECT DisplayName FROM core.users";
    let expected_sql = waf_encode_for_test(sql);

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/query-getData"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "source": {
                "type": "sql",
                "schemaName": "core",
                "sql": expected_sql
            },
            "renderer": {
                "type": "json"
            }
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "core",
            "rowCount": 0,
            "rows": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_data(
            GetDataOptions::builder()
                .source(GetDataSource::Sql {
                    schema_name: "core".to_string(),
                    sql: sql.to_string(),
                })
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
        .and(query_param("query.ignoreFilter", "true"))
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
async fn select_distinct_rows_omits_ignore_filter_when_unset() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-selectDistinct.api"))
        .and(query_param("dataRegionName", "query"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("query.queryName", "People"))
        .and(query_param("query.columns", "Gender"))
        .and(query_param_is_missing("query.ignoreFilter"))
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
        .and(query_param("grid.ignoreFilter", "true"))
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
async fn get_queries_sends_expected_params_and_deserializes_nested_queries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-getQueries.api"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("includeColumns", "true"))
        .and(query_param("includeSystemQueries", "false"))
        .and(query_param("includeTitle", "true"))
        .and(query_param("includeUserQueries", "true"))
        .and(query_param("includeViewDataUrl", "false"))
        .and(query_param("queryDetailColumns", "true"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "queries": [{
                "name": "People",
                "title": "People List",
                "columns": [{"name": "RowId", "caption": "Row Id"}],
                "canEdit": true,
                "canEditSharedViews": false,
                "hidden": false,
                "inherit": true,
                "isInherited": false,
                "isMetadataOverrideable": true,
                "isUserDefined": true,
                "snapshot": false,
                "viewDataUrl": "/list-grid.view?name=People"
            }]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_queries(
            GetQueriesOptions::builder()
                .schema_name("lists".to_string())
                .container_path("/Alt/Container".to_string())
                .include_columns(true)
                .include_system_queries(false)
                .include_title(true)
                .include_user_queries(true)
                .include_view_data_url(false)
                .query_detail_columns(true)
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.schema_name, "lists");
    assert_eq!(response.queries.len(), 1);
    assert_eq!(response.queries[0].name, "People");
}

#[tokio::test]
async fn get_schemas_deserializes_schema_keyed_fixture_response() {
    let server = MockServer::start().await;
    let payload: serde_json::Value = fixture("get_schemas.json");

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-getSchemas.api"))
        .and(query_param("includeHidden", "false"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("apiVersion", "17.1"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_schemas(
            GetSchemasOptions::builder()
                .container_path("/Alt/Container".to_string())
                .include_hidden(false)
                .schema_name("lists".to_string())
                .api_version("17.1".to_string())
                .build(),
        )
        .await
        .expect("request should succeed");

    let schemas = response
        .as_object()
        .expect("get_schemas should return object keyed by schema name");
    assert!(schemas.contains_key("lists"));
    assert!(schemas.contains_key("core"));
}

#[tokio::test]
async fn get_queries_supports_minimal_required_options() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/MyProject/MyFolder/query-getQueries.api"))
        .and(query_param("schemaName", "lists"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "queries": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_queries(
            GetQueriesOptions::builder()
                .schema_name("lists".to_string())
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response.schema_name, "lists");
    assert!(response.queries.is_empty());
}

#[tokio::test]
async fn get_query_views_sends_expected_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-getQueryViews.api"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("queryName", "People"))
        .and(query_param("viewName", "All"))
        .and(query_param("metadata", "{\"scope\":\"grid\"}"))
        .and(query_param("excludeSessionView", "true"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "views": [{"name": "All"}]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_query_views(
            GetQueryViewsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .view_name("All".to_string())
                .metadata(serde_json::json!({"scope": "grid"}))
                .exclude_session_view(true)
                .build(),
        )
        .await
        .expect("get query views should succeed");

    assert_eq!(response["queryName"], serde_json::json!("People"));
}

#[tokio::test]
async fn save_query_views_omits_false_boolean_flags() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-saveQueryViews.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "views": [{"name": "All"}],
            "session": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "saved": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .save_query_views(
            SaveQueryViewsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .views(serde_json::json!([{"name": "All"}]))
                .shared(false)
                .session(true)
                .hidden(false)
                .build(),
        )
        .await
        .expect("save query views should succeed");

    assert_eq!(response["saved"], serde_json::json!(true));
}

#[tokio::test]
async fn save_query_views_emits_true_flags_and_metadata() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-saveQueryViews.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "metadata": {"scope": "grid"},
            "views": [{"name": "All"}],
            "shared": true,
            "hidden": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "saved": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .save_query_views(
            SaveQueryViewsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .metadata(serde_json::json!({"scope": "grid"}))
                .views(serde_json::json!([{"name": "All"}]))
                .shared(true)
                .session(false)
                .hidden(true)
                .build(),
        )
        .await
        .expect("save query views should succeed");

    assert_eq!(response["saved"], serde_json::json!(true));
}

#[tokio::test]
async fn save_session_view_uses_flat_query_dot_keys() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-saveSessionView.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "query.queryName": "People",
            "query.viewName": "Session View",
            "newName": "Saved View",
            "shared": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "saved": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .save_session_view(
            SaveSessionViewOptions::builder()
                .container_path("/Alt/Container".to_string())
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .view_name("Session View".to_string())
                .new_name("Saved View".to_string())
                .shared(true)
                .replace(false)
                .build(),
        )
        .await
        .expect("save session view should succeed");

    assert_eq!(response["saved"], serde_json::json!(true));
}

#[tokio::test]
async fn delete_query_view_complete_semantics_follow_revert() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-deleteView.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "viewName": "Saved View"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deleted": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-deleteView.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "viewName": "Saved View",
            "complete": false
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deleted": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-deleteView.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "schemaName": "lists",
            "queryName": "People",
            "viewName": "Saved View",
            "complete": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deleted": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());

    let response_none = client
        .delete_query_view(
            DeleteQueryViewOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .view_name("Saved View".to_string())
                .build(),
        )
        .await
        .expect("delete query view with unset revert should succeed");
    assert_eq!(response_none["deleted"], serde_json::json!(true));

    let response_revert = client
        .delete_query_view(
            DeleteQueryViewOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .view_name("Saved View".to_string())
                .revert(true)
                .build(),
        )
        .await
        .expect("delete query view revert should succeed");
    assert_eq!(response_revert["deleted"], serde_json::json!(false));

    let response_complete = client
        .delete_query_view(
            DeleteQueryViewOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .view_name("Saved View".to_string())
                .revert(false)
                .build(),
        )
        .await
        .expect("delete query view complete should succeed");
    assert_eq!(response_complete["deleted"], serde_json::json!(true));
}

#[tokio::test]
async fn get_data_views_posts_browse_data_body_and_extracts_data_envelope() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/reports-browseData.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "includeData": true,
            "includeMetadata": false,
            "dataTypes": ["queries", "reports"]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": {
                "queries": [{"name": "People"}],
                "reports": []
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_data_views(
            GetDataViewsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .data_types(vec![DataViewType::Queries, DataViewType::Reports])
                .build(),
        )
        .await
        .expect("get data views should succeed");

    assert_eq!(result["queries"][0]["name"], serde_json::json!("People"));
}

#[tokio::test]
async fn get_data_views_missing_data_envelope_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/reports-browseData.api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "queries": []
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_data_views(GetDataViewsOptions::builder().build())
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::OK);
            assert!(text.contains("missing `data` field"));
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
}

#[tokio::test]
async fn get_data_views_invalid_data_envelope_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/reports-browseData.api"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "data": null
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_data_views(GetDataViewsOptions::builder().build())
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::OK);
            assert!(text.contains("invalid `data` field"));
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
}

#[tokio::test]
async fn validate_query_switches_action_when_metadata_validation_requested() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-validateQuery.api"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("queryName", "People"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "valid": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-validateQueryMetadata.api"))
        .and(query_param("schemaName", "lists"))
        .and(query_param("queryName", "People"))
        .and(query_param("viewName", "All"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "valid": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());

    let base_validation = client
        .validate_query(
            ValidateQueryOptions::builder()
                .container_path("/Alt/Container".to_string())
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .build(),
        )
        .await
        .expect("base validate query should succeed");
    assert!(base_validation.valid);

    let metadata_validation = client
        .validate_query(
            ValidateQueryOptions::builder()
                .container_path("/Alt/Container".to_string())
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .view_name("All".to_string())
                .validate_query_metadata(true)
                .build(),
        )
        .await
        .expect("metadata validate query should succeed");
    assert!(!metadata_validation.valid);
}

#[tokio::test]
async fn validate_query_forwards_sql_param() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/query-validateQuery.api"))
        .and(query_param(
            "sql",
            "SELECT * FROM lists.People WHERE Name = 'A&B'",
        ))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "valid": true
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .validate_query(
            ValidateQueryOptions::builder()
                .container_path("/Alt/Container".to_string())
                .sql("SELECT * FROM lists.People WHERE Name = 'A&B'".to_string())
                .build(),
        )
        .await
        .expect("validate query should succeed");

    assert!(result.valid);
}

#[tokio::test]
async fn get_server_date_uses_no_container_path_and_no_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/query-getServerDate.api"))
        .and(query_param_is_missing("schemaName"))
        .and(query_param_is_missing("queryName"))
        .and(query_param_is_missing("sql"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "date": "2026-03-04T18:00:00.000Z"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_server_date()
        .await
        .expect("get server date should succeed");

    assert_eq!(result.date, "2026-03-04T18:00:00.000Z");
}

#[tokio::test]
async fn get_data_json_error_maps_to_api_error() {
    let server = MockServer::start().await;
    let error_body: serde_json::Value = fixture("api_error.json");

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/query-getData"))
        .respond_with(ResponseTemplate::new(400).set_body_json(error_body))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_data(
            GetDataOptions::builder()
                .source(GetDataSource::Query {
                    schema_name: "lists".to_string(),
                    query_name: "People".to_string(),
                })
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::Api { status, .. }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
        }
        other => panic!("expected LabkeyError::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn get_data_non_json_error_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/query-getData"))
        .respond_with(ResponseTemplate::new(502).set_body_string("getData failed"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_data(
            GetDataOptions::builder()
                .source(GetDataSource::Query {
                    schema_name: "lists".to_string(),
                    query_name: "People".to_string(),
                })
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
            assert_eq!(text, "getData failed");
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
}

#[tokio::test]
async fn get_queries_json_error_maps_to_api_error() {
    let server = MockServer::start().await;
    let error_body: serde_json::Value = fixture("api_error.json");

    Mock::given(method("GET"))
        .and(path("/MyProject/MyFolder/query-getQueries.api"))
        .respond_with(ResponseTemplate::new(400).set_body_json(error_body))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_queries(
            GetQueriesOptions::builder()
                .schema_name("lists".to_string())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::Api { status, .. }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
        }
        other => panic!("expected LabkeyError::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn get_schemas_non_json_error_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/MyProject/MyFolder/query-getSchemas.api"))
        .respond_with(ResponseTemplate::new(502).set_body_string("schema service unavailable"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_schemas(GetSchemasOptions::builder().build())
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
            assert_eq!(text, "schema service unavailable");
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
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
async fn import_data_sends_text_source_and_optional_fields() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-import.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(header_exists("content-type"))
        .and(body_string_contains("name=\"schemaName\""))
        .and(body_string_contains("lists"))
        .and(body_string_contains("name=\"queryName\""))
        .and(body_string_contains("People"))
        .and(body_string_contains("name=\"text\""))
        .and(body_string_contains("Name,Age"))
        .and(body_string_contains("name=\"format\""))
        .and(body_string_contains("csv"))
        .and(body_string_contains("name=\"insertOption\""))
        .and(body_string_contains("IMPORT"))
        .and(body_string_contains("name=\"useAsync\""))
        .and(body_string_contains("\r\ntrue\r\n"))
        .and(body_string_contains("name=\"saveToPipeline\""))
        .and(body_string_contains("\r\nfalse\r\n"))
        .and(body_string_contains("name=\"importIdentity\""))
        .and(body_string_contains(
            "name=\"importIdentity\"\r\n\r\ntrue\r\n",
        ))
        .and(body_string_contains("name=\"importLookupByAlternateKey\""))
        .and(body_string_contains(
            "name=\"importLookupByAlternateKey\"\r\n\r\nfalse\r\n",
        ))
        .and(body_string_contains("name=\"auditUserComment\""))
        .and(body_string_contains("bulk text import"))
        .and(body_string_contains("name=\"auditDetails\""))
        .and(body_string_contains("\"source\":\"integration-test\""))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "rowCount": 1
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .import_data(
            ImportDataOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .source(ImportDataSource::Text("Name,Age\nAlice,30".to_string()))
                .container_path("/Alt/Container".to_string())
                .format("csv".to_string())
                .insert_option(InsertOption::Import)
                .use_async(true)
                .save_to_pipeline(false)
                .import_identity(true)
                .import_lookup_by_alternate_key(false)
                .audit_user_comment("bulk text import".to_string())
                .audit_details(serde_json::json!({"source": "integration-test"}))
                .build(),
        )
        .await
        .expect("import data should succeed");

    assert!(response.success);
    assert_eq!(response.row_count, 1);
    assert!(response.job_id.is_none());
}

#[tokio::test]
async fn import_data_sends_file_source() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-import.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(header_exists("content-type"))
        .and(body_string_contains("name=\"schemaName\""))
        .and(body_string_contains("lists"))
        .and(body_string_contains("name=\"queryName\""))
        .and(body_string_contains("People"))
        .and(body_string_contains("name=\"file\"; filename=\"rows.csv\""))
        .and(body_string_contains("Name,Age"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "rowCount": 1,
            "jobId": "job-42"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .import_data(
            ImportDataOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .source(ImportDataSource::File {
                    file_name: "rows.csv".to_string(),
                    bytes: b"Name,Age\nAlice,30".to_vec(),
                    mime_type: Some("text/csv".to_string()),
                })
                .build(),
        )
        .await
        .expect("file import should succeed");

    assert!(response.success);
    assert_eq!(response.row_count, 1);
    assert_eq!(response.job_id.as_deref(), Some("job-42"));
}

#[tokio::test]
async fn import_data_sends_path_source() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-import.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string_contains("name=\"schemaName\""))
        .and(body_string_contains("lists"))
        .and(body_string_contains("name=\"queryName\""))
        .and(body_string_contains("People"))
        .and(body_string_contains("name=\"path\""))
        .and(body_string_contains("/files/import/rows.tsv"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "rowCount": 5
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .import_data(
            ImportDataOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .source(ImportDataSource::Path("/files/import/rows.tsv".to_string()))
                .build(),
        )
        .await
        .expect("path import should succeed");

    assert!(response.success);
    assert_eq!(response.row_count, 5);
}

#[tokio::test]
async fn import_data_sends_module_resource_source() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-import.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string_contains("name=\"schemaName\""))
        .and(body_string_contains("lists"))
        .and(body_string_contains("name=\"queryName\""))
        .and(body_string_contains("People"))
        .and(body_string_contains("name=\"module\""))
        .and(body_string_contains("biologics"))
        .and(body_string_contains("name=\"moduleResource\""))
        .and(body_string_contains("data/test/lists/Vessel.tsv"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "rowCount": 3
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .import_data(
            ImportDataOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .source(ImportDataSource::ModuleResource {
                    module: "biologics".to_string(),
                    module_resource: "data/test/lists/Vessel.tsv".to_string(),
                })
                .build(),
        )
        .await
        .expect("module resource import should succeed");

    assert!(response.success);
    assert_eq!(response.row_count, 3);
}

#[tokio::test]
async fn import_data_sends_merge_insert_option_wire_value() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/query-import.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string_contains("name=\"insertOption\""))
        .and(body_string_contains("MERGE"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "rowCount": 2
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .import_data(
            ImportDataOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .container_path("/Alt/Container".to_string())
                .source(ImportDataSource::Text("Name,Age\nBob,31".to_string()))
                .insert_option(InsertOption::Merge)
                .build(),
        )
        .await
        .expect("merge import should succeed");

    assert!(response.success);
    assert_eq!(response.row_count, 2);
}

#[tokio::test]
async fn import_data_json_error_maps_to_api_error() {
    let server = MockServer::start().await;
    let error_body: serde_json::Value = fixture("api_error.json");

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/query-import.api"))
        .respond_with(ResponseTemplate::new(400).set_body_json(error_body))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .import_data(
            ImportDataOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .source(ImportDataSource::Text("Name\nAlice".to_string()))
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::Api { status, .. }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_REQUEST);
        }
        other => panic!("expected LabkeyError::Api, got {other:?}"),
    }
}

#[tokio::test]
async fn import_data_non_json_error_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/MyProject/MyFolder/query-import.api"))
        .respond_with(ResponseTemplate::new(502).set_body_string("import service unavailable"))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .import_data(
            ImportDataOptions::builder()
                .schema_name("lists".to_string())
                .query_name("People".to_string())
                .source(ImportDataSource::Text("Name\nAlice".to_string()))
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::BAD_GATEWAY);
            assert_eq!(text, "import service unavailable");
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
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

#[tokio::test]
async fn get_readable_containers_sends_expected_params_and_extracts_envelope() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/project-getReadableContainers.api"))
        .and(query_param("container", "/Home"))
        .and(query_param("includeSubfolders", "true"))
        .and(query_param("depth", "2"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "containers": ["/Home", "/Home/Project"]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_readable_containers(
            GetReadableContainersOptions::builder()
                .container_path("/Alt/Container".to_string())
                .container(vec!["/Home".to_string(), "/Ignored".to_string()])
                .include_subfolders(true)
                .depth(2)
                .build(),
        )
        .await
        .expect("request should succeed");

    assert_eq!(response, vec!["/Home", "/Home/Project"]);
}

#[tokio::test]
async fn get_readable_containers_invalid_envelope_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/MyProject/MyFolder/project-getReadableContainers.api",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "paths": ["/Home"]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_readable_containers(GetReadableContainersOptions::builder().build())
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::OK);
            assert!(text.contains("invalid getReadableContainers response"));
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
}

#[tokio::test]
async fn get_readable_containers_non_array_envelope_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path(
            "/MyProject/MyFolder/project-getReadableContainers.api",
        ))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "containers": "invalid"
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_readable_containers(GetReadableContainersOptions::builder().build())
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::OK);
            assert!(text.contains("invalid getReadableContainers response"));
        }
        other => panic!("expected LabkeyError::UnexpectedResponse, got {other:?}"),
    }
}

#[tokio::test]
async fn get_folder_types_posts_and_deserializes_folder_map() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/core-getFolderTypes.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "Study": {
                "name": "Study",
                "label": "Study Folder",
                "webParts": []
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_folder_types(
            GetFolderTypesOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("get folder types should succeed");

    assert_eq!(response.folder_types.len(), 1);
    assert_eq!(
        response
            .folder_types
            .get("Study")
            .expect("study type should exist")
            .name,
        "Study"
    );
}

#[tokio::test]
async fn get_modules_posts_and_deserializes_module_info_array() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/admin-getModules.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({})))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "folderType": "Collaboration",
            "modules": [
                {"name": "core", "properties": []},
                {"name": "query", "properties": [{"name": "version", "value": "1"}]}
            ]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_modules(
            GetModulesOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("get modules should succeed");

    assert_eq!(response.folder_type.as_deref(), Some("Collaboration"));
    assert_eq!(response.modules.len(), 2);
    assert_eq!(response.modules[1].name, "query");
}

#[tokio::test]
async fn move_container_posts_expected_body_and_url_contracts() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Source/Folder/core-moveContainer.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "container": "/Source/Folder",
            "parent": "/Target/Parent",
            "addAlias": true
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "newPath": "/Target/Parent/Folder"
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/core-moveContainer.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "container": "/Alt/Container",
            "parent": "/Target/Parent"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": false
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());

    let first = client
        .move_container(
            MoveContainerOptions::builder()
                .container("/Source/Folder".to_string())
                .parent("/Target/Parent".to_string())
                .add_alias(true)
                .build(),
        )
        .await
        .expect("move container should succeed");
    assert_eq!(first.success, Some(true));
    assert_eq!(
        first.extra.get("newPath"),
        Some(&serde_json::json!("/Target/Parent/Folder"))
    );

    let second = client
        .move_container(
            MoveContainerOptions::builder()
                .container_path("/Alt/Container".to_string())
                .container("/Alt/Container".to_string())
                .parent("/Target/Parent".to_string())
                .build(),
        )
        .await
        .expect("move container with override should succeed");
    assert_eq!(second.success, Some(false));
}

#[tokio::test]
async fn move_container_rejects_mismatched_container_path_override() {
    let client = test_client("https://labkey.example.com");

    let result = client
        .move_container(
            MoveContainerOptions::builder()
                .container_path("/Alt/Container".to_string())
                .container("/Source/Folder".to_string())
                .parent("/Target/Parent".to_string())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::InvalidInput(message)) => {
            assert_eq!(
                message,
                "move_container requires `container_path` to match `container` when provided"
            );
        }
        other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
    }
}

#[tokio::test]
async fn create_new_user_posts_expected_body_and_deserializes_response() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-createNewUser.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "email": "analyst@example.com",
            "sendEmail": true,
            "optionalMessage": "Welcome"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true,
            "htmlErrors": [],
            "users": [{
                "email": "analyst@example.com",
                "isNew": true,
                "message": "created",
                "userId": 410
            }]
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .create_new_user(
            CreateNewUserOptions::builder()
                .container_path("/Alt/Container".to_string())
                .email("analyst@example.com".to_string())
                .send_email(true)
                .optional_message("Welcome".to_string())
                .build(),
        )
        .await
        .expect("create new user should succeed");

    assert!(response.success);
    assert_eq!(response.users.len(), 1);
    assert_eq!(response.users[0].user_id, 410);
}

#[tokio::test]
async fn ensure_login_gets_expected_endpoint_and_deserializes_current_user() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-ensureLogin.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "currentUser": {
                "userId": 7,
                "email": "owner@example.com"
            }
        })))
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .ensure_login(
            EnsureLoginOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("ensure login should succeed");

    assert_eq!(response.current_user.user_id, 7);
}

#[tokio::test]
async fn logout_posts_no_body_to_login_logout_without_api_suffix() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/login-logout"))
        .and(query_param_is_missing("id"))
        .and(query_param_is_missing("email"))
        .and(query_param_is_missing("userId"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string(String::new()))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    client
        .logout(
            LogoutOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("logout should succeed");
}

#[tokio::test]
async fn who_am_i_gets_login_whoami_api_and_deserializes_fixture_response() {
    let server = MockServer::start().await;
    let payload: serde_json::Value = fixture("whoami.json");

    Mock::given(method("GET"))
        .and(path("/Alt/Container/login-whoami.api"))
        .and(query_param_is_missing("id"))
        .and(query_param_is_missing("email"))
        .and(query_param_is_missing("userId"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(payload))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .who_am_i(
            WhoAmIOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("who_am_i should succeed");

    assert_eq!(response.user_id, Some(101));
    assert_eq!(response.email.as_deref(), Some("analyst@example.com"));
}

#[tokio::test]
async fn delete_user_posts_typed_id_body_to_security_delete_user_without_api_suffix() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-deleteUser"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "id": 101
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deleted": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .delete_user(
            DeleteUserOptions::builder()
                .container_path("/Alt/Container".to_string())
                .id(101)
                .build(),
        )
        .await
        .expect("delete_user should succeed");

    assert_eq!(response.deleted, Some(true));
}

#[tokio::test]
async fn impersonate_user_uses_query_params_and_empty_request_body_for_both_target_modes() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/user-impersonateUser.api"))
        .and(query_param("userId", "101"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string(String::new()))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/user-impersonateUser.api"))
        .and(query_param("email", "analyst@example.com"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string(String::new()))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());

    client
        .impersonate_user(
            ImpersonateUserOptions::builder()
                .container_path("/Alt/Container".to_string())
                .target(ImpersonateTarget::UserId(101))
                .build(),
        )
        .await
        .expect("impersonate by user id should succeed");

    client
        .impersonate_user(
            ImpersonateUserOptions::builder()
                .container_path("/Alt/Container".to_string())
                .target(ImpersonateTarget::Email("analyst@example.com".to_string()))
                .build(),
        )
        .await
        .expect("impersonate by email should succeed");
}

#[tokio::test]
async fn impersonate_user_query_encodes_email_target_values() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/user-impersonateUser.api"))
        .and(query_param("email", "ana+lyst test@example.com"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string(String::new()))
        .respond_with(ResponseTemplate::new(200))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    client
        .impersonate_user(
            ImpersonateUserOptions::builder()
                .container_path("/Alt/Container".to_string())
                .target(ImpersonateTarget::Email(
                    "ana+lyst test@example.com".to_string(),
                ))
                .build(),
        )
        .await
        .expect("impersonate by encoded email should succeed");
}

#[tokio::test]
async fn stop_impersonating_treats_http_302_as_success_without_following_redirects() {
    let server = MockServer::start().await;
    let redirected = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/landing"))
        .respond_with(ResponseTemplate::new(200).set_body_string("unexpected redirect follow"))
        .expect(0)
        .mount(&redirected)
        .await;

    Mock::given(method("POST"))
        .and(path("/landing"))
        .respond_with(ResponseTemplate::new(200).set_body_string("unexpected redirect follow"))
        .expect(0)
        .mount(&redirected)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/login-stopImpersonating.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_string(String::new()))
        .respond_with(
            ResponseTemplate::new(302)
                .insert_header("Location", format!("{}/landing", redirected.uri())),
        )
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    client
        .stop_impersonating(
            StopImpersonatingOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("stop_impersonating should treat 302 as success");
}

#[tokio::test]
async fn impersonate_user_rejects_blank_email_target() {
    let client = test_client("https://labkey.example.com");

    let result = client
        .impersonate_user(
            ImpersonateUserOptions::builder()
                .target(ImpersonateTarget::Email("   ".to_string()))
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::InvalidInput(message)) => {
            assert_eq!(message, "impersonate_user email cannot be empty");
        }
        other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
    }
}

#[tokio::test]
async fn get_users_sends_expected_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/user-getUsers.api"))
        .and(query_param("groupId", "17"))
        .and(query_param("name", "ana"))
        .and(query_param("allMembers", "true"))
        .and(query_param("active", "false"))
        .and(query_param("permissions", "ReadPermission"))
        .and(query_param("permissions", "InsertPermission"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "container": "/Alt/Container",
            "users": [{"userId": 10}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let users = client
        .get_users(
            GetUsersOptions::builder()
                .container_path("/Alt/Container".to_string())
                .group_id(17)
                .name("ana".to_string())
                .all_members(true)
                .active(false)
                .permissions(vec![
                    "ReadPermission".to_string(),
                    "InsertPermission".to_string(),
                ])
                .build(),
        )
        .await
        .expect("get users should succeed");
    assert_eq!(users.users.len(), 1);
}

#[tokio::test]
async fn get_users_with_permissions_sends_expected_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/user-getUsersWithPermissions.api"))
        .and(query_param("group", "Researchers"))
        .and(query_param("includeInactive", "true"))
        .and(query_param("apiVersion", "23.11"))
        .and(query_param("permissions", "ReadPermission"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "users": [{"userId": 11}]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());

    let with_permissions = client
        .get_users_with_permissions(
            GetUsersWithPermissionsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .permissions(vec!["ReadPermission".to_string()])
                .group("Researchers".to_string())
                .include_inactive(true)
                .required_version("23.11".to_string())
                .build(),
        )
        .await
        .expect("get users with permissions should succeed");
    assert_eq!(with_permissions.users.len(), 1);
}

#[tokio::test]
async fn get_users_with_permissions_rejects_empty_permissions() {
    let client = test_client("https://labkey.example.com");

    let result = client
        .get_users_with_permissions(
            GetUsersWithPermissionsOptions::builder()
                .permissions(Vec::new())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::InvalidInput(message)) => {
            assert_eq!(
                message,
                "get_users_with_permissions requires at least one permission"
            );
        }
        other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
    }
}

#[tokio::test]
async fn group_create_and_membership_endpoints_send_expected_request_shapes() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-createGroup.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "name": "Research Analysts"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": 101,
            "name": "Research Analysts"
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-addGroupMember.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "groupId": 101,
            "principalIds": [201, 202]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "added": [201, 202]
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-removeGroupMember.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "groupId": 101,
            "principalIds": [201]
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "removed": [201]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());

    let created = client
        .create_group(
            CreateGroupOptions::builder()
                .container_path("/Alt/Container".to_string())
                .group_name("Research Analysts".to_string())
                .build(),
        )
        .await
        .expect("create group should succeed");
    assert_eq!(created.id, 101);

    let added = client
        .add_group_members(
            AddGroupMembersOptions::builder()
                .container_path("/Alt/Container".to_string())
                .group_id(101)
                .principal_ids(vec![201, 202])
                .build(),
        )
        .await
        .expect("add group members should succeed");
    assert_eq!(added.added, vec![201, 202]);

    let removed = client
        .remove_group_members(
            RemoveGroupMembersOptions::builder()
                .container_path("/Alt/Container".to_string())
                .group_id(101)
                .principal_ids(vec![201])
                .build(),
        )
        .await
        .expect("remove group members should succeed");
    assert_eq!(removed.removed, vec![201]);
}

#[tokio::test]
async fn add_group_members_rejects_empty_principal_ids() {
    let client = test_client("https://labkey.example.com");

    let result = client
        .add_group_members(
            AddGroupMembersOptions::builder()
                .group_id(101)
                .principal_ids(Vec::new())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::InvalidInput(message)) => {
            assert_eq!(
                message,
                "add_group_members requires at least one principal id"
            );
        }
        other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
    }
}

#[tokio::test]
async fn remove_group_members_rejects_empty_principal_ids() {
    let client = test_client("https://labkey.example.com");

    let result = client
        .remove_group_members(
            RemoveGroupMembersOptions::builder()
                .group_id(101)
                .principal_ids(Vec::new())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::InvalidInput(message)) => {
            assert_eq!(
                message,
                "remove_group_members requires at least one principal id"
            );
        }
        other => panic!("expected LabkeyError::InvalidInput, got {other:?}"),
    }
}

#[tokio::test]
async fn group_rename_and_delete_endpoints_send_expected_request_shapes() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-renameGroup.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "id": 101,
            "newName": "Senior Analysts"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "newName": "Senior Analysts",
            "oldName": "Research Analysts",
            "renamed": 101,
            "success": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-deleteGroup.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "id": 101
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "deleted": 1
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());

    let renamed = client
        .rename_group(
            RenameGroupOptions::builder()
                .container_path("/Alt/Container".to_string())
                .group_id(101)
                .new_name("Senior Analysts".to_string())
                .build(),
        )
        .await
        .expect("rename group should succeed");
    assert!(renamed.success);

    let deleted = client
        .delete_group(
            DeleteGroupOptions::builder()
                .container_path("/Alt/Container".to_string())
                .group_id(101)
                .build(),
        )
        .await
        .expect("delete group should succeed");
    assert_eq!(deleted.deleted, 1);
}

#[tokio::test]
async fn get_groups_for_current_user_gets_expected_response_shape() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-getGroupsForCurrentUser.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "groups": [{
                "id": 101,
                "name": "Senior Analysts",
                "isProjectGroup": true,
                "isSystemGroup": false
            }]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let groups = client
        .get_groups_for_current_user(
            GetGroupsForCurrentUserOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("get groups for current user should succeed");
    assert_eq!(groups.groups.len(), 1);
    assert_eq!(groups.groups[0].name, "Senior Analysts");
}

#[tokio::test]
async fn get_group_permissions_sends_expected_params_and_deserializes_children() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-getGroupPerms.api"))
        .and(query_param("includeSubfolders", "true"))
        .and(query_param("includeEmptyPermGroups", "false"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "container": {
                "id": "c1",
                "name": "Project",
                "path": "/Alt/Container",
                "groups": [{"groupId": 9, "name": "Readers"}],
                "children": [{
                    "id": "c2",
                    "name": "Subfolder",
                    "path": "/Alt/Container/Subfolder",
                    "groups": [],
                    "children": []
                }]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_group_permissions(
            GetGroupPermissionsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .include_subfolders(true)
                .include_empty_perm_groups(false)
                .build(),
        )
        .await
        .expect("get group permissions should succeed");

    assert_eq!(response.container.id, "c1");
    assert_eq!(response.container.children.len(), 1);
}

#[tokio::test]
async fn get_user_permissions_sends_user_email_and_deserializes_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-getUserPerms.api"))
        .and(query_param("userEmail", "analyst@example.com"))
        .and(query_param("includeSubfolders", "true"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "container": {
                "id": "c1",
                "name": "Project",
                "path": "/Alt/Container",
                "groups": [],
                "children": [],
                "effectivePermissions": ["org.labkey.api.security.permissions.ReadPermission"],
                "roles": ["org.labkey.security.roles.ReaderRole"]
            },
            "user": {
                "displayName": "Analyst",
                "userId": 101
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_user_permissions(
            GetUserPermissionsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .user_email("analyst@example.com".to_string())
                .include_subfolders(true)
                .build(),
        )
        .await
        .expect("get user permissions should succeed");

    assert_eq!(response.user.user_id, 101);
    assert_eq!(response.container.roles.len(), 1);
}

#[tokio::test]
async fn get_user_permissions_prefers_user_id_and_omits_user_email() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-getUserPerms.api"))
        .and(query_param("userId", "101"))
        .and(query_param_is_missing("userEmail"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "container": {
                "id": "c1",
                "name": "Project",
                "path": "/Alt/Container",
                "groups": [],
                "children": []
            },
            "user": {
                "userId": 101
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_user_permissions(
            GetUserPermissionsOptions::builder()
                .container_path("/Alt/Container".to_string())
                .user_id(101)
                .user_email("ignored@example.com".to_string())
                .build(),
        )
        .await
        .expect("get user permissions should succeed");

    assert_eq!(response.user.user_id, 101);
}

#[tokio::test]
async fn get_roles_expands_role_permission_references() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-getRoles.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "permissions": [
                {
                    "name": "Read",
                    "className": "org.labkey.api.security.permissions.ReadPermission",
                    "uniqueName": "org.labkey.api.security.permissions.ReadPermission"
                }
            ],
            "roles": [
                {
                    "name": "Reader",
                    "uniqueName": "org.labkey.security.roles.ReaderRole",
                    "permissions": ["org.labkey.api.security.permissions.ReadPermission"]
                }
            ]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let roles = client
        .get_roles(
            GetRolesOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await
        .expect("get roles should succeed");

    assert_eq!(roles.len(), 1);
    assert_eq!(roles[0].permissions.len(), 1);
    assert_eq!(roles[0].permissions[0].name, "Read");
}

#[tokio::test]
async fn get_securable_resources_extracts_envelope_and_params() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-getSecurableResources.api"))
        .and(query_param("includeSubfolders", "true"))
        .and(query_param("includeEffectivePermissions", "true"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resources": {
                "resourceId": "root",
                "name": "Root",
                "children": [{
                    "resourceId": "child",
                    "name": "Child",
                    "children": []
                }]
            }
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let resources = client
        .get_securable_resources(
            GetSecurableResourcesOptions::builder()
                .container_path("/Alt/Container".to_string())
                .include_subfolders(true)
                .include_effective_permissions(true)
                .build(),
        )
        .await
        .expect("get securable resources should succeed");

    assert_eq!(resources.resource_id, "root");
    assert_eq!(resources.children.len(), 1);
}

#[tokio::test]
async fn get_securable_resources_missing_envelope_maps_to_unexpected_response() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/Alt/Container/security-getSecurableResources.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "resource": {}
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client
        .get_securable_resources(
            GetSecurableResourcesOptions::builder()
                .container_path("/Alt/Container".to_string())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::UnexpectedResponse { status, text }) => {
            assert_eq!(status, reqwest::StatusCode::OK);
            assert!(text.contains("getSecurableResources"));
        }
        other => panic!("expected UnexpectedResponse, got {other:?}"),
    }
}

#[tokio::test]
async fn get_policy_posts_body_and_stamps_requested_resource_id() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-getPolicy.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "resourceId": "resource-1"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "policy": {
                "resourceId": "resource-from-server",
                "assignments": []
            },
            "relevantRoles": ["org.labkey.security.roles.ReaderRole"]
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let response = client
        .get_policy(
            GetPolicyOptions::builder()
                .container_path("/Alt/Container".to_string())
                .resource_id("resource-1".to_string())
                .build(),
        )
        .await
        .expect("get policy should succeed");

    assert_eq!(
        response.policy.resource_id.as_deref(),
        Some("resource-from-server")
    );
    assert_eq!(
        response.policy.requested_resource_id.as_deref(),
        Some("resource-1")
    );
    assert_eq!(response.relevant_roles.len(), 1);
}

#[tokio::test]
async fn save_policy_sends_expected_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-savePolicy.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "resourceId": "resource-1",
            "assignments": []
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let save_response = client
        .save_policy(
            SavePolicyOptions::builder()
                .container_path("/Alt/Container".to_string())
                .policy(serde_json::json!({
                    "policy": {
                        "resourceId": "resource-1",
                        "assignments": []
                    }
                }))
                .build(),
        )
        .await
        .expect("save policy should succeed");
    assert_eq!(save_response.success, Some(true));
}

#[tokio::test]
async fn delete_policy_sends_expected_request_shape() {
    let server = MockServer::start().await;

    Mock::given(method("POST"))
        .and(path("/Alt/Container/security-deletePolicy.api"))
        .and(header("x-requested-with", "XMLHttpRequest"))
        .and(basic_auth("apikey", "test-api-key"))
        .and(body_json(serde_json::json!({
            "resourceId": "resource-1"
        })))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "success": true
        })))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let delete_response = client
        .delete_policy(
            DeletePolicyOptions::builder()
                .container_path("/Alt/Container".to_string())
                .resource_id("resource-1".to_string())
                .build(),
        )
        .await
        .expect("delete policy should succeed");
    assert_eq!(delete_response.success, Some(true));
}

#[tokio::test]
async fn get_policy_rejects_blank_resource_id() {
    let client = test_client("https://labkey.example.com");

    let result = client
        .get_policy(
            GetPolicyOptions::builder()
                .resource_id("   ".to_string())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::InvalidInput(message)) => {
            assert_eq!(message, "get_policy resource_id cannot be empty");
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}

#[tokio::test]
async fn delete_policy_rejects_blank_resource_id() {
    let client = test_client("https://labkey.example.com");

    let result = client
        .delete_policy(
            DeletePolicyOptions::builder()
                .resource_id("\n\t".to_string())
                .build(),
        )
        .await;

    match result {
        Err(LabkeyError::InvalidInput(message)) => {
            assert_eq!(message, "delete_policy resource_id cannot be empty");
        }
        other => panic!("expected InvalidInput, got {other:?}"),
    }
}
