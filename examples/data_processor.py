#!/usr/bin/env python3
"""
Example Data Processing Script
Demonstrates working with files and data transformation
"""

import json
import csv
from datetime import datetime
from pathlib import Path

def generate_sample_data():
    """Generate sample data for processing"""
    return [
        {"id": 1, "name": "Alice", "score": 95, "date": "2026-01-15"},
        {"id": 2, "name": "Bob", "score": 87, "date": "2026-01-16"},
        {"id": 3, "name": "Charlie", "score": 92, "date": "2026-01-17"},
        {"id": 4, "name": "Diana", "score": 88, "date": "2026-01-18"},
        {"id": 5, "name": "Eve", "score": 99, "date": "2026-01-19"},
    ]

def process_data(data: list) -> dict:
    """Process and analyze data"""
    scores = [item["score"] for item in data]
    
    return {
        "total_records": len(data),
        "average_score": sum(scores) / len(scores),
        "max_score": max(scores),
        "min_score": min(scores),
        "top_performer": max(data, key=lambda x: x["score"])["name"],
    }

def main():
    print("=" * 50)
    print("ScriptRunner - Data Processing Example")
    print("=" * 50)
    print()
    
    print("Generating sample data...")
    data = generate_sample_data()
    
    print(f"Processing {len(data)} records...")
    results = process_data(data)
    
    print()
    print("Results:")
    print(f"  Total Records: {results['total_records']}")
    print(f"  Average Score: {results['average_score']:.2f}")
    print(f"  Max Score: {results['max_score']}")
    print(f"  Min Score: {results['min_score']}")
    print(f"  Top Performer: {results['top_performer']}")
    print()
    
    # Save results as JSON
    output = {
        "timestamp": datetime.now().isoformat(),
        "analysis": results,
        "data": data
    }
    
    output_json = json.dumps(output, indent=2)
    print("Output JSON:")
    print(output_json)
    
    print()
    print("Processing completed successfully!")

if __name__ == "__main__":
    main()
