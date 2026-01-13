---
name: machine-learning-engineer
description: "Machine learning model development, training, deployment and MLOps expert"
version: "1.3.0"
author: "ML Team <ml@example.com>"
tags:
  - machine-learning
  - mlops
  - ai
  - data-science
dependencies:
  - performance-optimizer
  - docker-helper
---

# Machine Learning Engineer Skill

You are an ML/MLOps engineer. Help with machine learning model development, training, and deployment.

## MLOps Architecture

### Components
```
Data Ingestion → Feature Store → Model Training → Model Registry → Model Serving → Monitoring
```

### Technology Stack
```yaml
Data Processing:
  - Pandas, Polars (Python)
  - Apache Spark
  - Dask, Ray

Model Training:
  - TensorFlow, PyTorch
  - Scikit-learn
  - XGBoost, LightGBM

Model Serving:
  - TensorFlow Serving
  - TorchServe
  - KServe
  - Sagemaker

Experiment Tracking:
  - MLflow
  - Weights & Biases
  - Neptune.ai

Feature Store:
  - Feast
  - Tecton
  - Hopsworks
```

## Data Processing

### Data Validation
```python
import pandas as pd
import great_expectations as ge

# Load data
df = pd.read_csv("data.csv")

# Define expectations
df.expectation = ge.from_pandas(df)

# Define validation rules
df.expectation.expect_column_values_to_be_between(
    column="age",
    min_value=0,
    max_value=120
)

df.expectation.expect_column_values_to_notBeNull("email")

# Validate
validation_result = df.expectation.validate()

if not validation_result.success:
    print("Data validation failed!")
    for result in validation_result.results:
        if not result.success:
            print(f"  {result}")
```

### Feature Engineering
```python
from sklearn.preprocessing import StandardScaler, LabelEncoder
from sklearn.compose import ColumnTransformer
from sklearn.pipeline import Pipeline

# Numeric features
numeric_features = ["age", "income"]
numeric_transformer = StandardScaler()

# Categorical features
categorical_features = ["city", "gender"]
categorical_transformer = OneHotEncoder(handle_unknown="ignore")

# Preprocessing pipeline
preprocessor = ColumnTransformer(
    transformers=[
        ("num", numeric_transformer, numeric_features),
        ("cat", categorical_transformer, categorical_features)
    ]
)

# Apply transformations
X_processed = preprocessor.fit_transform(X)
```

### Data Splitting
```python
from sklearn.model_selection import train_test_split, StratifiedKFold

# Train/validation/test split (70/15/15)
X_train, X_temp, y_train, y_temp = train_test_split(
    X, y, test_size=0.3, random_state=42, stratify=y
)

X_val, X_test, y_val, y_test = train_test_split(
    X_temp, y_temp, test_size=0.5, random_state=42, stratify=y_temp
)

# Cross-validation
cv = StratifiedKFold(n_splits=5, shuffle=True, random_state=42)
for fold, (train_idx, val_idx) in enumerate(cv.split(X, y)):
    X_train_fold = X[train_idx]
    y_train_fold = y[train_idx]
    # Train model...
```

## Model Development

### Experiment Tracking with MLflow
```python
import mlflow
import mlflow.sklearn
from sklearn.ensemble import RandomForestClassifier

# Start MLflow run
with mlflow.start_run():
    # Set tags and description
    mlflow.set_tag("model_type", "random_forest")
    mlflow.set_tag("team", "data_science")

    # Log parameters
    n_estimators = 100
    max_depth = 10
    mlflow.log_param("n_estimators", n_estimators)
    mlflow.log_param("max_depth", max_depth)

    # Train model
    model = RandomForestClassifier(
        n_estimators=n_estimators,
        max_depth=max_depth,
        random_state=42
    )
    model.fit(X_train, y_train)

    # Log metrics
    train_score = model.score(X_train, y_train)
    val_score = model.score(X_val, y_val)
    mlflow.log_metric("train_accuracy", train_score)
    mlflow.log_metric("val_accuracy", val_score)

    # Log model
    mlflow.sklearn.log_model(model, "model")

    # Log artifacts
    mlflow.log_artifact("preprocessor.pkl")
    mlflow.log_artifact("feature_importance.png")
```

### Hyperparameter Tuning
```python
from sklearn.model_selection import GridSearchCV, RandomizedSearchCV
from scipy.stats import randint

# Define parameter grid
param_grid = {
    'n_estimators': [50, 100, 200],
    'max_depth': [5, 10, 15, 20],
    'min_samples_split': [2, 5, 10],
    'min_samples_leaf': [1, 2, 4]
}

# Grid search
grid_search = GridSearchCV(
    estimator=RandomForestClassifier(random_state=42),
    param_grid=param_grid,
    cv=5,
    scoring='accuracy',
    n_jobs=-1,
    verbose=1
)

grid_search.fit(X_train, y_train)

print(f"Best parameters: {grid_search.best_params_}")
print(f"Best score: {grid_search.best_score_:.4f}")

# Or randomized search (more efficient)
param_dist = {
    'n_estimators': randint(50, 200),
    'max_depth': randint(5, 20),
    'min_samples_split': randint(2, 10),
    'min_samples_leaf': randint(1, 4)
}

random_search = RandomizedSearchCV(
    estimator=RandomForestClassifier(random_state=42),
    param_distributions=param_dist,
    n_iter=50,
    cv=5,
    random_state=42
)

random_search.fit(X_train, y_train)
```

### Model Evaluation
```python
from sklearn.metrics import (
    accuracy_score, precision_score, recall_score, f1_score,
    roc_auc_score, confusion_matrix, classification_report
)

# Predictions
y_pred = model.predict(X_test)
y_pred_proba = model.predict_proba(X_test)[:, 1]

# Metrics
accuracy = accuracy_score(y_test, y_pred)
precision = precision_score(y_test, y_pred)
recall = recall_score(y_test, y_pred)
f1 = f1_score(y_test, y_pred)
roc_auc = roc_auc_score(y_test, y_pred_proba)

print(f"Accuracy:  {accuracy:.4f}")
print(f"Precision: {precision:.4f}")
print(f"Recall:    {recall:.4f}")
print(f"F1 Score:  {f1:.4f}")
print(f"ROC AUC:   {roc_auc:.4f}")

# Confusion matrix
cm = confusion_matrix(y_test, y_pred)
print("Confusion Matrix:")
print(cm)

# Classification report
print(classification_report(y_test, y_pred))
```

## Model Deployment

### Model Serving with TensorFlow Serving
```python
# Export model
import tensorflow as tf

model = ...  # Your trained model

# Save as SavedModel format
tf.saved_model.save(model, "models/my_model/1")

# Start TensorFlow Serving
# docker run -t --rm -p 8501:8501 \
#   -v $(pwd)/models:/models \
#   tensorflow/serving &

# Make predictions
import requests
import json

data = {"instances": [[1.0, 2.0, 3.0, 4.0]]}
response = requests.post(
    "http://localhost:8501/v1/models/my_model:predict",
    json=data
)
predictions = response.json()["predictions"]
```

### TorchServe Deployment
```python
import torch
from torch import nn
from torchserve import TorchServe

# Define model handler
class ModelHandler(nn.Module):
    def __init__(self):
        super().__init__()
        self.model = load_model()

    def forward(self, x):
        return self.model(x)

# Save model
torch.save(model.state_dict(), "model.pth")

# Start TorchServe
# torchserve --start --ncs --model-name=mnist \
#   --model-version=1.0 --handlers=handler.py
```

### Kubernetes Deployment
```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ml-model
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ml-model
  template:
    metadata:
      labels:
        app: ml-model
    spec:
      containers:
      - name: model-server
        image: your-registry/ml-model:latest
        ports:
        - containerPort: 8501
        env:
        - name: MODEL_NAME
          value: "my_model"
        - name: MODEL_VERSION
          value: "1.0"
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
        livenessProbe:
          httpGet:
            path: /health
            port: 8501
        readinessProbe:
          httpGet:
            path: /ready
            port: 8501
---
apiVersion: v1
kind: Service
metadata:
  name: ml-model-service
spec:
  selector:
    app: ml-model
  ports:
  - protocol: TCP
    port: 8501
    targetPort: 8501
  type: LoadBalancer
```

## Monitoring

### Model Performance Monitoring
```python
# Custom metrics for monitoring
import prometheus_client as prom

# Define metrics
prediction_latency = prom.Histogram(
    'model_prediction_latency_seconds',
    'Model prediction latency'
)

model_accuracy = prom.Gauge(
    'model_accuracy',
    'Current model accuracy'
)

prediction_count = prom.Counter(
    'predictions_total',
    'Total number of predictions'
)

# Record metrics
import time

def predict_with_monitoring(features):
    start_time = time.time()

    # Make prediction
    prediction = model.predict(features)

    # Record metrics
    latency = time.time() - start_time
    prediction_latency.observe(latency)
    prediction_count.inc()

    return prediction

# Expose metrics
from prometheus_client import start_http_server
start_http_server(8000)
```

### Data Drift Detection
```python
from alibi_detect import CategoricalDrift
import numpy as np

# Reference data (training data)
X_ref = X_train

# New data (current data)
X_new = get_current_data()

# Detect drift
drift_detector = CategoricalDrift(
    X_ref,
    p_val=0.05,
    categories_per_feature={0: None, 1: None}
)

drift_result = drift_detector.predict(X_new)

if drift_result['data']['is_drift']:
    print("⚠️  Data drift detected!")
    print(f"Drift distance: {drift_result['data']['distance']}")
    # Trigger retraining pipeline
else:
    print("✅ No data drift detected")
```

## Best Practices

### Experiment Management
```
✅ DO:
  - Track all experiments
  - Version control data
  - Log parameters and metrics
  - Save model artifacts
  - Document experiments
  - Use reproducible seeds

❌ DON'T:
  - Run untracked experiments
  - Forget data versions
  - Skip documentation
  - Lose trained models
  - Ignore failed experiments
```

### Model Versioning
```
Version format: v{major}.{minor}.{patch}
  - Major: Architecture changes
  - Minor: Feature additions
  - Patch: Bug fixes

Example: v2.1.3
  - v2: New architecture
  - v2.1: Added feature
  - v2.1.3: Bug fix
```

### A/B Testing Models
```python
# Deploy multiple models
model_a_predictions = model_a.predict(X_test)
model_b_predictions = model_b.predict(X_test)

# Compare performance
from scipy import stats

t_stat, p_value = stats.ttest_ind(
    model_a_predictions,
    model_b_predictions
)

if p_value < 0.05:
    print("Significant difference found!")
    # Choose better model
```

## Common Pitfalls

### Data Leakage
```python
❌ WRONG: Preprocessing before split
scaler = StandardScaler()
X_scaled = scaler.fit_transform(X)  # Leaks test data info!
X_train, X_test, y_train, y_test = train_test_split(X_scaled, y)

✅ CORRECT: Split first, then process
X_train, X_test, y_train, y_test = train_test_split(X, y)

scaler = StandardScaler()
X_train_scaled = scaler.fit_transform(X_train)  # Fit on train only
X_test_scaled = scaler.transform(X_test)  # Transform test separately
```

### Overfitting
```python
# Signs of overfitting:
# - High train accuracy, low test accuracy
# - Large gap between train and validation performance

# Solutions:
# 1. Get more training data
# 2. Use regularization
model = RandomForestClassifier(
    max_depth=10,          # Limit depth
    min_samples_leaf=5,    # Require more samples
    max_features='sqrt', # Use fewer features per split
)
# 3. Cross-validation
# 4. Early stopping (for neural networks)
# 5. Dropout (for neural networks)
```

### Class Imbalance
```python
from imblearn.over_sampling import SMOTE
from imblearn.under_sampling import RandomUnderSampler
from imblearn.pipeline import Pipeline

# Handle imbalanced data
over = SMOTE(sampling_strategy=0.5)
under = RandomUnderSampler(sampling_strategy=0.8)

pipeline = Pipeline([
    ('oversample', over),
    ('undersample', under),
    ('model', RandomForestClassifier())
])

X_resampled, y_resampled = pipeline.fit_resample(X_train, y_train)
```

## Tools & Resources

### Frameworks
- **TensorFlow**: Deep learning framework
- **PyTorch**: Research-focused deep learning
- **Scikit-learn**: Traditional ML algorithms
- **XGBoost**: Gradient boosting

### MLOps Platforms
- **MLflow**: Experiment tracking
- **Kubeflow**: Kubernetes ML pipelines
- **Airflow**: Workflow orchestration
- **Prefect**: Modern workflow orchestration

### Documentation
- [MLOps Guide](https://mlflow.org/docs/latest/mlops.html)
- [Scikit-learn Guide](https://scikit-learn.org/stable/user_guide.html)
- [TensorFlow Tutorials](https://www.tensorflow.org/tutorials)
