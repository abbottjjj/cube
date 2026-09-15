use crate::test_fixtures::cube_bridge::MockSchema;
use crate::test_fixtures::test_utils::TestContext;
use indoc::indoc;

fn create_context() -> TestContext {
    let schema = MockSchema::from_yaml_file("common/integration_multi_stage.yaml");
    TestContext::new(schema).unwrap()
}

const SEED: &str = "integration_multi_stage_tables.sql";

#[tokio::test(flavor = "multi_thread")]
async fn test_two_add_group_by() {
    let ctx = create_context();

    let query = indoc! {r#"
        measures:
          - orders.amount_by_id
          - orders.amount_by_category
        time_dimensions:
          - dimension: orders.created_at
            granularity: month
            dateRange:
              - "2024-01-01"
              - "2024-03-31"
    "#};

    ctx.build_sql(query).unwrap();

    if let Some(result) = ctx.try_execute_pg(query, SEED).await {
        insta::assert_snapshot!(result);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_add_group_by_and_reduce_by() {
    let ctx = create_context();

    let query = indoc! {r#"
        measures:
          - orders.amount_by_id
          - orders.amount_reduce_category
        dimensions:
          - orders.category
        time_dimensions:
          - dimension: orders.created_at
            granularity: month
            dateRange:
              - "2024-01-01"
              - "2024-03-31"
        order:
          - id: orders.category
    "#};

    ctx.build_sql(query).unwrap();

    if let Some(result) = ctx.try_execute_pg(query, SEED).await {
        insta::assert_snapshot!(result);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_shift_and_add_group_by() {
    let ctx = create_context();

    let query = indoc! {r#"
        measures:
          - orders.amount_prev_month
          - orders.amount_by_id
        time_dimensions:
          - dimension: orders.created_at
            granularity: month
            dateRange:
              - "2024-01-01"
              - "2024-03-31"
    "#};

    ctx.build_sql(query).unwrap();

    if let Some(result) = ctx.try_execute_pg(query, SEED).await {
        insta::assert_snapshot!(result);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_rank_and_regular_multi_stage() {
    let ctx = create_context();

    let query = indoc! {r#"
        measures:
          - orders.total_amount
          - orders.amount_rank
          - orders.amount_by_id
        time_dimensions:
          - dimension: orders.created_at
            granularity: month
            dateRange:
              - "2024-01-01"
              - "2024-03-31"
    "#};

    ctx.build_sql(query).unwrap();

    if let Some(result) = ctx.try_execute_pg(query, SEED).await {
        insta::assert_snapshot!(result);
    }
}

/// Regression: multi-stage ratio (mom_growth) + regular sum (total_amount) +
/// non-time dimension must not NULL-out the regular measure on Keys FKA path
/// (MySQL/GBase dialects without FULL JOIN).
#[tokio::test(flavor = "multi_thread")]
async fn test_mom_growth_with_regular_measure_and_dimension() {
    let ctx = create_context();

    let query = indoc! {r#"
        measures:
          - orders.mom_growth
          - orders.total_amount
        dimensions:
          - orders.category
        time_dimensions:
          - dimension: orders.created_at
            granularity: month
            dateRange:
              - "2024-01-01"
              - "2024-03-31"
        order:
          - id: orders.category
    "#};

    let sql = ctx.build_sql(query).unwrap();
    assert!(
        !sql.contains("fk_aggregate_keys"),
        "regular + multi-stage with dimensions should use regular-first join, not UNION keys"
    );
    assert!(
        sql.contains("LEFT JOIN"),
        "multi-stage branch should LEFT JOIN to the regular measure anchor"
    );

    if let Some(result) = ctx.try_execute_pg(query, SEED).await {
        insta::assert_snapshot!(result);
    }
}

#[tokio::test(flavor = "multi_thread")]
async fn test_three_multi_stage_types() {
    let ctx = create_context();

    let query = indoc! {r#"
        measures:
          - orders.amount_by_id
          - orders.amount_reduce_category
          - orders.amount_prev_month
        dimensions:
          - orders.category
        time_dimensions:
          - dimension: orders.created_at
            granularity: month
            dateRange:
              - "2024-01-01"
              - "2024-03-31"
        order:
          - id: orders.category
    "#};

    ctx.build_sql(query).unwrap();

    if let Some(result) = ctx.try_execute_pg(query, SEED).await {
        insta::assert_snapshot!(result);
    }
}
