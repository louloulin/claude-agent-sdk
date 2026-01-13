---
name: data-engineering
description: "Expert in data pipeline design, ETL/ELT processes, data warehousing, and big data technologies"
version: "1.4.0"
author: "Data Team <data@example.com>"
tags:
  - data-engineering
  - etl
  - data-warehouse
  - big-data
  - pipelines
  - analytics
dependencies:
  - docker-helper
  - performance-optimizer
---

# Data Engineering Expert

You are a data engineering expert specializing in building scalable data pipelines, data warehouses, and analytics infrastructure.

## Data Pipeline Architecture

### Pipeline Patterns

```
Batch Processing (Traditional):
  Sources → Extraction → Transformation → Loading → Warehouse
  (Daily/Hourly)   (SQL/Spark)      (Modeling)    (Analytics)

Streaming (Real-Time):
  Sources → Ingestion → Stream Processing → Serving Layer → Analytics
  (Events)   (Kafka)      (Flink/Spark)      (Redis/DB)      (Real-Time)

Modern Data Stack (ELT):
  Sources → Ingestion → Warehouse (Raw) → Transformation (dbt) → Analytics
  (SaaS/API)   (Fivetran)      (Snowflake)      (SQL Models)         (BI Tools)
```

### Technology Stack

```yaml
Data Ingestion:
  Batch:
    - Airbyte: Open-source ELT
    - Fivetran: Managed ELT
    - AWS Glue: Serverless ETL
    - Sqoop: Hadoop ecosystem

  Streaming:
    - Apache Kafka: Distributed event streaming
    - AWS Kinesis: Managed streaming
    - Google Pub/Sub: Event streaming
    - Apache Pulsar: Cloud-native messaging

Data Storage:
  Data Warehouses:
    - Snowflake: Cloud data warehouse
    - Google BigQuery: Serverless analytics
    - Amazon Redshift: Petabyte-scale warehouse
    - Azure Synapse: Integrated analytics

  Data Lakes:
    - AWS S3 + Athena: Serverless data lake
    - Azure Data Lake: Hierarchical namespace
    - Google Cloud Storage: Object storage
    - Delta Lake: ACID transactions on data lakes
    - Apache Iceberg: Table format for huge datasets

Data Processing:
  Batch:
    - Apache Spark: Distributed computing
    - Pandas: Single-machine processing
    - Dask: Parallel computing
    - Ray: Distributed Python

  Streaming:
    - Apache Flink: Stream processing
    - Spark Streaming: Micro-batch
    - Kafka Streams: Stream processing
    - Apache Storm: Real-time computation

Orchestration:
  - Apache Airflow: Workflow orchestration
  - Prefect: Modern workflow orchestration
  - Dagster: Data-aware orchestration
  - AWS Step Functions: Serverless workflows
  - Kubeflow: ML pipelines

Transformation:
  - dbt: Data build tool (SQL transformations)
  - SQL: Standard query language
  - Pandas/Polars: Python transformations
```

## ETL/ELT Implementation

### Batch ETL with Apache Spark

```python
from pyspark.sql import SparkSession
from pyspark.sql.functions import col, to_date, year, month
from datetime import datetime

# Initialize Spark session
spark = SparkSession.builder \
    .appName("CustomerDataETL") \
    .config("spark.sql.adaptive.enabled", "true") \
    .config("spark.sql.adaptive.coalescePartitions.enabled", "true") \
    .getOrCreate()

# Extraction: Read from multiple sources
# 1. Read from PostgreSQL
customers_df = spark.read \
    .format("jdbc") \
    .option("url", "jdbc:postgresql://db-host:5432/warehouse") \
    .option("dbtable", "public.customers") \
    .option("user", "etl_user") \
    .option("password", "secure_password") \
    .option("driver", "org.postgresql.Driver") \
    .load()

# 2. Read from S3 (JSON files)
orders_df = spark.read \
    .format("json") \
    .option("inferSchema", "true") \
    .load("s3a://data-lake/raw/orders/*.json")

# 3. Read from Kafka (streaming data)
events_df = spark.read \
    .format("kafka") \
    .option("kafka.bootstrap.servers", "kafka-broker:9092") \
    .option("subscribe", "user-events") \
    .load()

# Transformation
# Clean and standardize data
customers_clean = customers_df \
    .filter(col("email").isNotNull()) \
    .filter(col("email").rlike("^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$")) \
    .withColumn("created_date", to_date(col("created_at"))) \
    .withColumn("created_year", year(col("created_date"))) \
    .withColumn("created_month", month(col("created_date"))) \
    .withColumn("full_name",
        col("first_name") + " " + col("last_name"))

# Join datasets
customer_orders = orders_df \
    .join(customers_clean, "customer_id", "inner") \
    .groupBy(
        col("customer_id"),
        col("full_name"),
        col("email")
    ) \
    .agg(
        {"order_amount": "sum", "order_id": "count"}
    ) \
    .withColumnRenamed("sum(order_amount)", "total_spent") \
    .withColumnRenamed("count(order_id)", "order_count") \
    .withColumn("customer_segment",
        col("total_spent") / col("order_count"))

# Loading: Write to data warehouse (Snowflake)
customer_orders.write \
    .format("net.snowflake.spark.snowflake") \
    .options(
        sfUrl="snowflake-account.snowflakecomputing.com",
        sfUser="etl_user",
        sfPassword="password",
        sfDatabase="analytics",
        sfSchema="public",
        sfWarehouse="compute_wh"
    ) \
    .option("dbtable", "customer_summary") \
    .mode("overwrite") \
    .save()

# Also save to S3 as Parquet (data lake)
customer_orders.write \
    .mode("overwrite") \
    .partitionBy("created_year", "created_month") \
    .format("parquet") \
    .save("s3a://data-lake/analytics/customer_summary/")

spark.stop()
```

### Streaming ETL with Kafka and Flink

```java
import org.apache.flink.streaming.api.datastream.DataStream;
import org.apache.flink.streaming.api.environment.StreamExecutionEnvironment;
import org.apache.flink.streaming.connectors.kafka.FlinkKafkaConsumer;
import org.apache.flink.streaming.connectors.kafka.FlinkKafkaProducer;
import org.apache.flink.api.common.serialization.SimpleStringSchema;
import org.apache.flink.streaming.api.functions.ProcessFunction;
import org.apache.flink.util.Collector;
import org.apache.flink.configuration.Configuration;
import com.fasterxml.jackson.databind.ObjectMapper;

import java.util.Properties;

public class StreamingETL {

    public static void main(String[] args) throws Exception {

        // Create execution environment
        final StreamExecutionEnvironment env = StreamExecutionEnvironment.getExecutionEnvironment();

        // Kafka source properties
        Properties kafkaProps = new Properties();
        kafkaProps.setProperty("bootstrap.servers", "kafka-broker:9092");
        kafkaProps.setProperty("group.id", "etl-consumer-group");

        // Create Kafka consumer
        FlinkKafkaConsumer<String> kafkaSource = new FlinkKafkaConsumer<>(
            "raw-events",
            new SimpleStringSchema(),
            kafkaProps
        );

        // Read from Kafka
        DataStream<String> rawEvents = env.addSource(kafkaSource);

        // Transform events
        DataStream<String> processedEvents = rawEvents.process(
            new ProcessFunction<String, String>() {

                private transient ObjectMapper objectMapper;

                @Override
                public void open(Configuration parameters) {
                    objectMapper = new ObjectMapper();
                }

                @Override
                public void processElement(
                    String value,
                    Context ctx,
                    Collector<String> out
                ) throws Exception {

                    try {
                        // Parse JSON event
                        JsonNode event = objectMapper.readTree(value);

                        // Extract fields
                        String eventType = event.get("type").asText();
                        String userId = event.get("user_id").asText();
                        long timestamp = event.get("timestamp").asLong();

                        // Validate and enrich
                        if (isValidUser(userId) && isValidTimestamp(timestamp)) {
                            // Add processing timestamp
                            ((ObjectNode) event).put("processed_at", System.currentTimeMillis());

                            // Transform to target format
                            String transformed = objectMapper.writeValueAsString(event);
                            out.collect(transformed);
                        }
                    } catch (Exception e) {
                        // Handle malformed events
                        System.err.println("Error processing event: " + e.getMessage());
                    }
                }
            }
        );

        // Write to different topics based on event type
        // For simplicity, we'll write to a single processed topic
        Properties producerProps = new Properties();
        producerProps.setProperty("bootstrap.servers", "kafka-broker:9092");

        FlinkKafkaProducer<String> kafkaSink = new FlinkKafkaProducer<>(
            "processed-events",
            new SimpleStringSchema(),
            producerProps
        );

        processedEvents.addSink(kafkaSink);

        // Execute job
        env.execute("Streaming ETL Job");
    }

    private static boolean isValidUser(String userId) {
        return userId != null && !userId.isEmpty();
    }

    private static boolean isValidTimestamp(long timestamp) {
        long now = System.currentTimeMillis();
        long oneDayMillis = 24 * 60 * 60 * 1000;
        return timestamp > (now - oneDayMillis) && timestamp <= now;
    }
}
```

### ELT with dbt (Data Build Tool)

```sql
-- models/staging/customers.sql
-- Raw staging table
with source as (
    select * from {{ source('raw', 'customers') }}
),

renamed as (
    select
        id as customer_id,
        first_name,
        last_name,
        email,
        created_at,
        updated_at
    from source
)

select * from renamed

---

-- models/intermediate/customer_orders.sql
-- Intermediate transformation
with customer_orders as (
    select
        customer_id,
        order_id,
        order_amount,
        order_date
    from {{ ref('staging_orders') }}
),

aggregated as (
    select
        customer_id,
        count(*) as total_orders,
        sum(order_amount) as total_spent,
        min(order_date) as first_order_date,
        max(order_date) as last_order_date
    from customer_orders
    group by customer_id
)

select * from aggregated

---

-- models/marts/customer_summary.sql
-- Final mart for analytics
with customers as (
    select * from {{ ref('staging_customers') }}
),

orders as (
    select * from {{ ref('customer_orders') }}
),

enriched as (
    select
        c.customer_id,
        c.first_name || ' ' || c.last_name as full_name,
        c.email,
        coalesce(o.total_orders, 0) as total_orders,
        coalesce(o.total_spent, 0) as total_spent,
        o.first_order_date,
        case
            when o.total_spent > 1000 then 'High Value'
            when o.total_spent > 500 then 'Medium Value'
            else 'Low Value'
        end as customer_segment,
        datediff('day', o.last_order_date, current_date) as days_since_last_order
    from customers c
    left join orders o using (customer_id)
)

select * from enriched
```

```yaml
# dbt_project.yml
name: data-engineering
version: '1.0.0'
config-version: 2

profile: 'data_warehouse'

model-paths: ["models"]
seed-paths: ["seeds"]
test-paths: ["tests"]
analysis-paths: ["analyses"]
macro-paths: ["macros"]

target-path: "target"
clean-targets:
  - "target"
  - "dbt_packages"

models:
  data_warehouse:
    staging:
      +materialized: view
    intermediate:
      +materialized: table
    marts:
      +materialized: table
      +schema: marts
```

## Data Warehouse Design

### Star Schema

```sql
-- Dimensional modeling for e-commerce

-- Dimension: Customers
CREATE TABLE dim_customers (
    customer_key INT PRIMARY KEY,
    customer_id INT UNIQUE,
    full_name VARCHAR(200),
    email VARCHAR(255),
    country_key INT,
    signup_date DATE,
    customer_segment VARCHAR(50),
    is_active BOOLEAN
);

-- Dimension: Products
CREATE TABLE dim_products (
    product_key INT PRIMARY KEY,
    product_id INT UNIQUE,
    product_name VARCHAR(500),
    category_key INT,
    brand VARCHAR(200),
    price DECIMAL(10, 2),
    launch_date DATE
);

-- Dimension: Date
CREATE TABLE dim_date (
    date_key INT PRIMARY KEY, -- YYYYMMDD format
    full_date DATE,
    day_of_week INT,
    day_name VARCHAR(10),
    week INT,
    month INT,
    month_name VARCHAR(15),
    quarter INT,
    year INT,
    is_holiday BOOLEAN,
    is_weekend BOOLEAN
);

-- Fact: Orders
CREATE TABLE fact_orders (
    order_key BIGINT PRIMARY KEY,
    order_id BIGINT UNIQUE,
    customer_key INT,
    product_key INT,
    date_key INT,
    order_date DATE,
    quantity INT,
    unit_price DECIMAL(10, 2),
    total_amount DECIMAL(10, 2),
    discount_amount DECIMAL(10, 2),
    net_amount DECIMAL(10, 2),
    FOREIGN KEY (customer_key) REFERENCES dim_customers(customer_key),
    FOREIGN KEY (product_key) REFERENCES dim_products(product_key),
    FOREIGN KEY (date_key) REFERENCES dim_date(date_key)
);

-- Create indexes for performance
CREATE INDEX idx_fact_customer ON fact_orders(customer_key);
CREATE INDEX idx_fact_product ON fact_orders(product_key);
CREATE INDEX idx_fact_date ON fact_orders(date_key);
CREATE INDEX idx_fact_order_date ON fact_orders(order_date);

-- Partitioning (for large tables)
CREATE TABLE fact_orders_partitioned (
    LIKE fact_orders INCLUDING ALL
) PARTITION BY RANGE (order_date);

-- Create monthly partitions
CREATE TABLE fact_orders_2024_01 PARTITION OF fact_orders_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-02-01');
```

### Snowflake Schema

```sql
-- More normalized structure for large enterprises

-- Core dimension
CREATE TABLE dim_customers (
    customer_key INT PRIMARY KEY,
    customer_id INT UNIQUE,
    full_name VARCHAR(200),
    email VARCHAR(255),
    signup_date DATE
);

-- Outrigger dimensions (normalized)
CREATE TABLE dim_customer_demographics (
    customer_key INT PRIMARY KEY REFERENCES dim_customers(customer_key),
    age_group VARCHAR(20),
    income_level VARCHAR(20),
    education_level VARCHAR(50),
    occupation VARCHAR(100)
);

CREATE TABLE dim_customer_geography (
    customer_key INT PRIMARY KEY REFERENCES dim_customers(customer_key),
    country VARCHAR(100),
    state_province VARCHAR(100),
    city VARCHAR(100),
    postal_code VARCHAR(20)
);

CREATE TABLE dim_customer_behavior (
    customer_key INT PRIMARY KEY REFERENCES dim_customers(customer_key),
    customer_segment VARCHAR(50),
    loyalty_tier VARCHAR(30),
    preferred_channel VARCHAR(50),
    days_since_last_purchase INT
);
```

## Orchestration with Airflow

```python
from datetime import datetime, timedelta
from airflow import DAG
from airflow.operators.python import PythonOperator
from airflow.operators.postgres import PostgresOperator
from airflow.providers.apache.spark.operators.spark_submit import SparkSubmitOperator
from airflow.providers.amazon.aws.transfers.s3_to_redshift import S3ToRedshiftOperator

default_args = {
    'owner': 'data-team',
    'depends_on_past': False,
    'start_date': datetime(2024, 1, 1),
    'email_on_failure': True,
    'email_on_retry': False,
    'retries': 2,
    'retry_delay': timedelta(minutes=5),
}

dag = DAG(
    'daily_customer_etl',
    default_args=default_args,
    description='Daily customer data ETL pipeline',
    schedule_interval='0 2 * * *',  # Run daily at 2 AM
    catchup=False,
    max_active_runs=1,
    tags=['etl', 'customer', 'daily'],
)

# Task 1: Extract from PostgreSQL and save to S3
extract_to_s3 = SparkSubmitOperator(
    task_id='extract_to_s3',
    application='/opt/airflow/jobs/extract_postgres.py',
    jars='/opt/spark/jars/postgresql-42.3.1.jar,/opt/spark/jars/hadoop-aws-3.3.1.jar',
    application_args=[
        '--source', 'postgresql://db-host/warehouse',
        '--table', 'public.customers',
        '--target', 's3a://data-lake/raw/customers/',
        '--date', '{{ ds }}'
    ],
    conn_id='spark_default',
    dag=dag,
)

# Task 2: Transform with Spark
transform_data = SparkSubmitOperator(
    task_id='transform_data',
    application='/opt/airflow/jobs/transform_customers.py',
    application_args=[
        '--input', 's3a://data-lake/raw/customers/',
        '--output', 's3a://data-lake/analytics/customer_summary/',
        '--date', '{{ ds }}'
    ],
    dag=dag,
)

# Task 3: Load into Redshift
load_to_redshift = S3ToRedshiftOperator(
    task_id='load_to_redshift',
    s3_bucket='data-lake',
    s3_key='analytics/customer_summary/',
    schema='public',
    table='customer_summary',
    copy_params=(
        "FORMAT AS PARQUET "
        "TIMEFORMAT 'auto' "
        "COMPUPDATE ON "
        "STATUPDATE ON"
    ),
    redshift_conn_id='redshift_default',
    aws_conn_id='aws_default',
    method='REPLACE',
    dag=dag,
)

# Task 4: Update materialized views
update_views = PostgresOperator(
    task_id='update_views',
    postgres_conn_id='analytics_db',
    sql="""
        REFRESH MATERIALIZED VIEW CONCURRENTLY mv_customer_metrics;
        REFRESH MATERIALIZED VIEW CONCURRENTLY mv_daily_revenue;
    """,
    dag=dag,
)

# Task 5: Data quality checks
def run_data_quality_checks(**context):
    from airflow.hooks.postgres_hook import PostgresHook

    hook = PostgresHook(postgres_conn_id='analytics_db')

    # Check 1: No null emails
    null_emails = hook.get_first(
        "SELECT COUNT(*) FROM customer_summary WHERE email IS NULL"
    )[0]

    if null_emails > 0:
        raise ValueError(f"Found {null_emails} null emails")

    # Check 2: Reasonable date range
    future_dates = hook.get_first(
        "SELECT COUNT(*) FROM customer_summary WHERE created_date > CURRENT_DATE"
    )[0]

    if future_dates > 0:
        raise ValueError(f"Found {future_dates} future dates")

    # Check 3: Row count check
    row_count = hook.get_first(
        "SELECT COUNT(*) FROM customer_summary"
    )[0]

    if row_count < 1000:
        raise ValueError(f"Row count {row_count} below threshold (1000)")

    print("All data quality checks passed!")

data_quality = PythonOperator(
    task_id='data_quality_checks',
    python_callable=run_data_quality_checks,
    dag=dag,
)

# Define task dependencies
extract_to_s3 >> transform_data >> load_to_redshift
load_to_redshift >> update_views
load_to_redshift >> data_quality
```

## Data Quality & Validation

### Great Expectations

```python
import great_expectations as ge
from great_expectations.core.batch import RuntimeBatchRequest

# Get context
context = ge.get_context()

# Create expectation suite
suite = context.add_or_update_expectation_suite("customer_data_suite")

# Define expectations
expectations = [
    {
        "expectation_type": "expect_column_to_exist",
        "kwargs": {"column": "customer_id"}
    },
    {
        "expectation_type": "expect_column_values_to_be_unique",
        "kwargs": {"column": "customer_id"}
    },
    {
        "expectation_type": "expect_column_values_to_not_be_null",
        "kwargs": {"column": "email"}
    },
    {
        "expectation_type": "expect_column_values_to_match_regex",
        "kwargs": {
            "column": "email",
            "regex": r"^[\w-\.]+@([\w-]+\.)+[\w-]{2,4}$"
        }
    },
    {
        "expectation_type": "expect_column_values_to_be_between",
        "kwargs": {
            "column": "age",
            "min_value": 0,
            "max_value": 120
        }
    },
    {
        "expectation_type": "expect_table_row_count_to_be_between",
        "kwargs": {
            "min_value": 1000,
            "max_value": 1000000
        }
    }
]

for expectation in expectations:
    context.add_expectation_to_suite(
        suite,
        expectation["expectation_type"],
        **expectation["kwargs"]
    )

# Validate data
batch_request = RuntimeBatchRequest(
    datasource_name="my_datasource",
    data_connector_name="runtime_data_connector",
    data_asset_name="customer_data",
    runtime_parameters={"batch_data": df},  # pandas DataFrame
    batch_identifiers={"batch_id": "daily_batch"}
)

validation_result = context.run_validation_operator(
    "action_list_operator",
    [batch_request]
)

# Check results
if validation_result["success"]:
    print("✅ All data quality checks passed!")
else:
    print("❌ Data quality issues detected")
    for result in validation_result.results:
        if not result.success:
            print(f"  {result.expectation_config.expectation_type}: FAILED")
```

### SQL Data Quality Checks

```sql
-- Data quality validation queries

-- Check 1: Null values in critical columns
SELECT
    'email_null_check' as check_name,
    COUNT(*) as issue_count,
    CASE WHEN COUNT(*) = 0 THEN 'PASS' ELSE 'FAIL' END as status
FROM customers
WHERE email IS NULL;

-- Check 2: Duplicate records
SELECT
    'duplicate_check' as check_name,
    COUNT(*) - COUNT(DISTINCT customer_id) as issue_count,
    CASE WHEN COUNT(*) = COUNT(DISTINCT customer_id) THEN 'PASS' ELSE 'FAIL' END as status
FROM customers;

-- Check 3: Referential integrity
SELECT
    'ref_integrity_check' as check_name,
    COUNT(*) as issue_count,
    'FAIL' as status
FROM orders o
LEFT JOIN customers c ON o.customer_id = c.customer_id
WHERE c.customer_id IS NULL;

-- Check 4: Data freshness
SELECT
    'freshness_check' as check_name,
    CASE
        WHEN MAX(created_at) >= CURRENT_DATE - INTERVAL '1 day' THEN 0
        ELSE 1
    END as issue_count,
    CASE
        WHEN MAX(created_at) >= CURRENT_DATE - INTERVAL '1 day' THEN 'PASS'
        ELSE 'FAIL'
    END as status
FROM customers;

-- Check 5: Value range validation
SELECT
    'age_range_check' as check_name,
    COUNT(*) as issue_count,
    CASE WHEN COUNT(*) = 0 THEN 'PASS' ELSE 'FAIL' END as status
FROM customers
WHERE age < 0 OR age > 120;
```

## Performance Optimization

### Query Optimization

```sql
-- Bad query: Full table scan
SELECT *
FROM orders
WHERE LOWER(customer_name) = 'john doe';

-- Good query: Index-friendly query with proper filtering
SELECT *
FROM orders
WHERE customer_id = 12345;

-- Bad: Using functions on columns in WHERE
WHERE DATE(order_date) = '2024-01-15'

-- Good: Use range queries
WHERE order_date >= '2024-01-15' AND order_date < '2024-01-16'

-- Bad: SELECT *
SELECT * FROM customers;

-- Good: Select only needed columns
SELECT customer_id, customer_name, email
FROM customers;

-- Bad: N+1 query problem (executing query in loop)
-- Good: Use JOIN or subqueries

-- Bad: Correlated subquery
SELECT customer_name,
    (SELECT COUNT(*) FROM orders WHERE orders.customer_id = customers.customer_id)
FROM customers;

-- Good: Use JOIN with GROUP BY
SELECT c.customer_name, COUNT(o.order_id) as order_count
FROM customers c
LEFT JOIN orders o ON o.customer_id = c.customer_id
GROUP BY c.customer_id;
```

### Partitioning Strategy

```sql
-- Partition large fact table by date
CREATE TABLE fact_orders_partitioned (
    order_key BIGINT,
    customer_key INT,
    product_key INT,
    order_date DATE,
    amount DECIMAL(10, 2),
    ...
) PARTITION BY RANGE (order_date);

-- Create partitions
CREATE TABLE fact_orders_2024_q1 PARTITION OF fact_orders_partitioned
    FOR VALUES FROM ('2024-01-01') TO ('2024-04-01');

CREATE TABLE fact_orders_2024_q2 PARTITION OF fact_orders_partitioned
    FOR VALUES FROM ('2024-04-01') TO ('2024-07-01');

-- Query pruning
SELECT *
FROM fact_orders_partitioned
WHERE order_date >= '2024-05-01' AND order_date < '2024-06-01';
-- Only scans fact_orders_2024_q2 partition
```

## Best Practices

### Pipeline Design
```
✅ DO:
  - Design for idempotency (re-running should be safe)
  - Handle late-arriving data gracefully
  - Implement data quality checkpoints
  - Monitor pipeline performance
  - Use schema evolution strategies
  - Document data lineage
  - Implement retry mechanisms
  - Use partitioning for large datasets
  - Compress data for storage efficiency
  - Back up critical data

❌ DON'T:
  - Hardcode configuration values
  - Skip data validation
  - Ignore error handling
  - Create tight coupling between stages
  - Forget about schema changes
  - Skip monitoring and alerting
  - Process everything in one huge job
  - Use SELECT * in production queries
```

### Data Modeling
```
Dimensional Modeling:
  ✅ Use star schema for simplicity
  ✅ Use snowflake for large dimensions
  ✅ Create surrogate keys
  ✅ Add date dimension table
  ✅ Use degenerate dimensions when appropriate
  ✅ Implement slowly changing dimensions (SCD Type 2)

Performance:
  ✅ Create appropriate indexes
  ✅ Use materialized views for complex queries
  ✅ Partition large tables
  ✅ Use columnar storage for analytics
  ✅ Cluster tables on join keys
  ✅ Use appropriate data types
```

## Tools & Resources

### ETL Tools
- **Apache Airflow**: Workflow orchestration
- **Prefect/Dagster**: Modern orchestration
- **Apache Spark**: Large-scale data processing
- **dbt**: Data transformation with SQL
- **Fivetran/Airbyte**: Data ingestion

### Data Warehouses
- **Snowflake**: Cloud data warehouse
- **BigQuery**: Serverless analytics
- **Redshift**: AWS data warehouse
- **ClickHouse**: Fast analytical database

### Documentation
- [Data Engineering Best Practices](https://www.dataengineeringpodcast.com/)
- [Modern Data Stack](https://www.moderndatastack.xyz/)
- [dbt Documentation](https://docs.getdbt.com/)
