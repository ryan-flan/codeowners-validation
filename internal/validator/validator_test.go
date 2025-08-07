package validator

import (
	"testing"

	"github.com/ryan-flan/codeowners-validation/internal/parser"
)

func TestNewValidatorArgsFromString(t *testing.T) {
	tests := []struct {
		name     string
		input    string
		expected ValidatorArgs
	}{
		{
			name:  "exists only",
			input: "exists",
			expected: ValidatorArgs{
				Exists:            true,
				DuplicatePatterns: false,
			},
		},
		{
			name:  "duplicate_patterns only",
			input: "duplicate_patterns",
			expected: ValidatorArgs{
				Exists:            false,
				DuplicatePatterns: true,
			},
		},
		{
			name:  "both validators",
			input: "exists,duplicate_patterns",
			expected: ValidatorArgs{
				Exists:            true,
				DuplicatePatterns: true,
			},
		},
		{
			name:  "all shorthand",
			input: "all",
			expected: ValidatorArgs{
				Exists:            true,
				DuplicatePatterns: true,
			},
		},
		{
			name:  "with whitespace",
			input: " exists , duplicate_patterns ",
			expected: ValidatorArgs{
				Exists:            true,
				DuplicatePatterns: true,
			},
		},
		{
			name:  "empty string",
			input: "",
			expected: ValidatorArgs{
				Exists:            false,
				DuplicatePatterns: false,
			},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := NewValidatorArgsFromString(tt.input)
			if result.Exists != tt.expected.Exists {
				t.Errorf("Expected Exists=%v, got %v", tt.expected.Exists, result.Exists)
			}
			if result.DuplicatePatterns != tt.expected.DuplicatePatterns {
				t.Errorf("Expected DuplicatePatterns=%v, got %v", tt.expected.DuplicatePatterns, result.DuplicatePatterns)
			}
		})
	}
}

func TestValidatorArgs_ShouldRunAll(t *testing.T) {
	tests := []struct {
		name     string
		args     ValidatorArgs
		expected bool
	}{
		{
			name:     "default args should run all",
			args:     ValidatorArgs{},
			expected: true,
		},
		{
			name:     "exists only should not run all",
			args:     ValidatorArgs{Exists: true},
			expected: false,
		},
		{
			name:     "duplicate_patterns only should not run all",
			args:     ValidatorArgs{DuplicatePatterns: true},
			expected: false,
		},
		{
			name:     "both enabled should not run all",
			args:     ValidatorArgs{Exists: true, DuplicatePatterns: true},
			expected: false,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := tt.args.ShouldRunAll()
			if result != tt.expected {
				t.Errorf("ShouldRunAll() = %v, want %v", result, tt.expected)
			}
		})
	}
}

func makeRule(pattern, original string) parser.CodeOwnerRule {
	return parser.CodeOwnerRule{
		Pattern:      pattern,
		OriginalPath: original,
		Owners:       []string{"@test"},
	}
}

func TestRunValidators(t *testing.T) {
	tests := []struct {
		name          string
		args          ValidatorArgs
		rules         []parser.CodeOwnerRule
		expectFailure bool
	}{
		{
			name: "run all by default",
			args: ValidatorArgs{},
			rules: []parser.CodeOwnerRule{
				makeRule("missing1.txt", "missing1.txt"),
				makeRule("dup.txt", "dup.txt"),
				makeRule("dup.txt", "dup.txt"),
			},
			expectFailure: true,
		},
		{
			name: "run only exists when enabled",
			args: ValidatorArgs{Exists: true, DuplicatePatterns: false},
			rules: []parser.CodeOwnerRule{
				makeRule("notfound.txt", "notfound.txt"),
			},
			expectFailure: true,
		},
		{
			name: "run only duplicates when enabled",
			args: ValidatorArgs{Exists: false, DuplicatePatterns: true},
			rules: []parser.CodeOwnerRule{
				makeRule("x.txt", "x.txt"),
				makeRule("x.txt", "x.txt"),
			},
			expectFailure: true,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			results, err := RunValidators(tt.args, tt.rules)
			if err != nil {
				t.Fatalf("RunValidators() error = %v", err)
			}

			hasFailures := len(results) > 0
			if hasFailures != tt.expectFailure {
				t.Errorf("RunValidators() hasFailures = %v, want %v", hasFailures, tt.expectFailure)
			}
		})
	}
}
