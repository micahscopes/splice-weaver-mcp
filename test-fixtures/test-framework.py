#!/usr/bin/env python3
"""
AST-Grep Testing Framework

This framework provides comprehensive testing capabilities for ast-grep patterns
and transformations across multiple programming languages.
"""

import json
import os
import sys
from pathlib import Path
from typing import Dict, List, Any, Optional
from dataclasses import dataclass, asdict
import subprocess
import tempfile

@dataclass
class TestCase:
    """Represents a single test case for ast-grep patterns."""
    name: str
    language: str
    code: str
    pattern: str
    expected_matches: int
    expected_vars: Optional[Dict[str, Any]] = None
    replacement: Optional[str] = None
    expected_replacement: Optional[str] = None
    description: Optional[str] = None

@dataclass
class TestResult:
    """Represents the result of a test case execution."""
    test_case: TestCase
    passed: bool
    actual_matches: int
    actual_vars: Optional[Dict[str, Any]] = None
    actual_replacement: Optional[str] = None
    error: Optional[str] = None

class AstGrepTester:
    """Main testing framework for ast-grep patterns."""
    
    def __init__(self, fixtures_dir: str = "test-fixtures"):
        self.fixtures_dir = Path(fixtures_dir)
        self.test_cases: List[TestCase] = []
        self.results: List[TestResult] = []
        
    def add_test_case(self, test_case: TestCase) -> None:
        """Add a test case to the test suite."""
        self.test_cases.append(test_case)
        
    def load_test_cases_from_json(self, json_file: str) -> None:
        """Load test cases from a JSON file."""
        with open(json_file, 'r') as f:
            data = json.load(f)
            for case_data in data.get('test_cases', []):
                test_case = TestCase(**case_data)
                self.add_test_case(test_case)
    
    def run_single_test(self, test_case: TestCase) -> TestResult:
        """Run a single test case and return the result."""
        try:
            # Test search pattern
            matches = self._run_ast_grep_search(test_case.code, test_case.pattern, test_case.language)
            
            passed = len(matches) == test_case.expected_matches
            actual_vars = matches[0].get('vars', {}) if matches else {}
            
            # Test replacement if specified
            actual_replacement = None
            if test_case.replacement:
                actual_replacement = self._run_ast_grep_replace(
                    test_case.code, test_case.pattern, test_case.replacement, test_case.language
                )
                if test_case.expected_replacement:
                    passed = passed and (actual_replacement == test_case.expected_replacement)
            
            # Test expected variables if specified
            if test_case.expected_vars and matches:
                vars_match = all(
                    actual_vars.get(k) == v for k, v in test_case.expected_vars.items()
                )
                passed = passed and vars_match
            
            return TestResult(
                test_case=test_case,
                passed=passed,
                actual_matches=len(matches),
                actual_vars=actual_vars,
                actual_replacement=actual_replacement
            )
            
        except Exception as e:
            return TestResult(
                test_case=test_case,
                passed=False,
                actual_matches=0,
                error=str(e)
            )
    
    def run_all_tests(self) -> List[TestResult]:
        """Run all test cases and return results."""
        self.results = []
        for test_case in self.test_cases:
            result = self.run_single_test(test_case)
            self.results.append(result)
        return self.results
    
    def _run_ast_grep_search(self, code: str, pattern: str, language: str) -> List[Dict[str, Any]]:
        """Run ast-grep search command and return matches."""
        with tempfile.NamedTemporaryFile(mode='w', suffix=f'.{self._get_file_extension(language)}', delete=False) as f:
            f.write(code)
            f.flush()
            
            try:
                # Run ast-grep search
                cmd = ['sg', 'search', '--pattern', pattern, '--lang', language, f.name]
                result = subprocess.run(cmd, capture_output=True, text=True, check=False)
                
                if result.returncode != 0:
                    return []
                
                # Parse ast-grep output (simplified - in real implementation would parse JSON)
                lines = result.stdout.strip().split('\n')
                matches = []
                if lines and lines[0]:
                    # For simplicity, assume each non-empty line is a match
                    for line in lines:
                        if line.strip():
                            matches.append({'text': line, 'vars': {}})
                
                return matches
                
            finally:
                os.unlink(f.name)
    
    def _run_ast_grep_replace(self, code: str, pattern: str, replacement: str, language: str) -> str:
        """Run ast-grep replace command and return transformed code."""
        with tempfile.NamedTemporaryFile(mode='w', suffix=f'.{self._get_file_extension(language)}', delete=False) as f:
            f.write(code)
            f.flush()
            
            try:
                # Run ast-grep replace
                cmd = ['sg', 'replace', '--pattern', pattern, '--replacement', replacement, '--lang', language, f.name]
                result = subprocess.run(cmd, capture_output=True, text=True, check=False)
                
                if result.returncode != 0:
                    return code
                
                # Read the transformed file
                with open(f.name, 'r') as transformed:
                    return transformed.read()
                
            finally:
                os.unlink(f.name)
    
    def _get_file_extension(self, language: str) -> str:
        """Get file extension for a programming language."""
        extensions = {
            'javascript': 'js',
            'typescript': 'ts',
            'python': 'py',
            'rust': 'rs',
            'go': 'go',
            'java': 'java',
            'c': 'c',
            'cpp': 'cpp',
            'csharp': 'cs',
            'php': 'php',
            'ruby': 'rb',
            'swift': 'swift',
            'kotlin': 'kt',
            'scala': 'scala',
            'bash': 'sh',
            'html': 'html',
            'css': 'css',
            'json': 'json',
            'yaml': 'yaml',
        }
        return extensions.get(language, 'txt')
    
    def generate_report(self) -> str:
        """Generate a comprehensive test report."""
        if not self.results:
            return "No test results available. Run tests first."
        
        total_tests = len(self.results)
        passed_tests = sum(1 for r in self.results if r.passed)
        failed_tests = total_tests - passed_tests
        
        report = f"""
# AST-Grep Test Report

## Summary
- **Total Tests**: {total_tests}
- **Passed**: {passed_tests}
- **Failed**: {failed_tests}
- **Success Rate**: {(passed_tests/total_tests)*100:.1f}%

## Test Results

"""
        
        for result in self.results:
            status = "✅ PASSED" if result.passed else "❌ FAILED"
            report += f"""
### {result.test_case.name} - {status}
- **Language**: {result.test_case.language}
- **Pattern**: `{result.test_case.pattern}`
- **Expected Matches**: {result.test_case.expected_matches}
- **Actual Matches**: {result.actual_matches}
"""
            
            if result.test_case.description:
                report += f"- **Description**: {result.test_case.description}\n"
            
            if result.error:
                report += f"- **Error**: {result.error}\n"
            
            if result.actual_vars:
                report += f"- **Variables**: {result.actual_vars}\n"
            
            if result.actual_replacement:
                report += f"- **Replacement Result**: ```{result.test_case.language}\n{result.actual_replacement}\n```\n"
        
        return report
    
    def save_report(self, filename: str) -> None:
        """Save test report to file."""
        report = self.generate_report()
        with open(filename, 'w') as f:
            f.write(report)
    
    def create_sample_test_cases(self) -> None:
        """Create sample test cases for demonstration."""
        # JavaScript function detection
        self.add_test_case(TestCase(
            name="javascript_function_detection",
            language="javascript",
            code='''
            function greet(name) {
                return "Hello, " + name;
            }
            
            const add = (a, b) => a + b;
            ''',
            pattern="function $NAME($PARAMS) { $BODY }",
            expected_matches=1,
            expected_vars={"NAME": "greet"},
            description="Detect JavaScript function declarations"
        ))
        
        # Console.log detection
        self.add_test_case(TestCase(
            name="console_log_detection",
            language="javascript",
            code='''
            console.log("Debug message");
            console.error("Error message");
            console.log(variable);
            ''',
            pattern="console.log($ARG)",
            expected_matches=2,
            description="Detect console.log statements"
        ))
        
        # Variable replacement
        self.add_test_case(TestCase(
            name="var_to_const_replacement",
            language="javascript",
            code="var oldVar = 'value';",
            pattern="var $NAME = $VALUE",
            expected_matches=1,
            replacement="const $NAME = $VALUE",
            expected_replacement="const oldVar = 'value';",
            description="Replace var with const"
        ))
        
        # Rust function detection
        self.add_test_case(TestCase(
            name="rust_function_detection",
            language="rust",
            code='''
            fn greet(name: &str) -> String {
                format!("Hello, {}!", name)
            }
            
            fn add(a: i32, b: i32) -> i32 {
                a + b
            }
            ''',
            pattern="fn $NAME($PARAMS) -> $RETURN { $BODY }",
            expected_matches=2,
            description="Detect Rust function definitions with return types"
        ))
        
        # Python function detection
        self.add_test_case(TestCase(
            name="python_function_detection",
            language="python",
            code='''
def greet(name):
    return f"Hello, {name}"

def add(a, b):
    return a + b
            ''',
            pattern="def $NAME($PARAMS):",
            expected_matches=2,
            description="Detect Python function definitions"
        ))

def main():
    """Main function to run the test framework."""
    tester = AstGrepTester()
    
    # Create sample test cases
    tester.create_sample_test_cases()
    
    # Run all tests
    print("Running ast-grep tests...")
    results = tester.run_all_tests()
    
    # Generate and display report
    report = tester.generate_report()
    print(report)
    
    # Save report to file
    tester.save_report("test-report.md")
    print("\\nTest report saved to test-report.md")
    
    # Exit with appropriate code
    failed_count = sum(1 for r in results if not r.passed)
    sys.exit(failed_count)

if __name__ == "__main__":
    main()