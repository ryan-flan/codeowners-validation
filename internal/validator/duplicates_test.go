package validator

import (
	"testing"

	"github.com/ryan-flan/codeowners-validation/internal/parser"
)

func TestDuplicatePatternsValidator_Validate(t *testing.T) {
	validator := &DuplicatePatternsValidator{}

	tests := []struct {
		name               string
		rules              []parser.CodeOwnerRule
		expectedDuplicates int
	}{
		{
			name: "no duplicates",
			rules: []parser.CodeOwnerRule{
				makeRule("a.txt", "a.txt"),
				makeRule("b.txt", "b.txt"),
			},
			expectedDuplicates: 0,
		},
		{
			name: "detects exact duplicates",
			rules: []parser.CodeOwnerRule{
				makeRule("src", "src/"),
				makeRule("src", "src/"),
			},
			expectedDuplicates: 1,
		},
		{
			name: "normalized duplicates detected",
			rules: []parser.CodeOwnerRule{
				makeRule("docs", "/docs"),
				makeRule("docs", "docs"),
			},
			expectedDuplicates: 1,
		},
		{
			name: "different slash variations",
			rules: []parser.CodeOwnerRule{
				makeRule("src/lib", "/src/lib/"),
				makeRule("src/lib", "src/lib"),
				makeRule("src/lib", "/src/lib"),
			},
			expectedDuplicates: 2, // Second and third are duplicates
		},
		{
			name: "original path duplicates",
			rules: []parser.CodeOwnerRule{
				makeRule("docs", "docs/"),
				makeRule("docs", "docs/"),
			},
			expectedDuplicates: 1,
		},
		{
			name: "wildcard duplicates",
			rules: []parser.CodeOwnerRule{
				makeRule("*.md", "*.md"),
				makeRule("*.md", "*.md"),
			},
			expectedDuplicates: 1,
		},
		{
			name: "complex pattern duplicates",
			rules: []parser.CodeOwnerRule{
				makeRule("**/*.test.js", "**/*.test.js"),
				makeRule("src/*.rs", "src/*.rs"),
				makeRule("**/*.test.js", "**/*.test.js"), // duplicate
			},
			expectedDuplicates: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			duplicates, err := validator.Validate(tt.rules)
			if err != nil {
				t.Fatalf("Validate() error = %v", err)
			}

			if len(duplicates) != tt.expectedDuplicates {
				t.Errorf("Validate() found %d duplicates, want %d", len(duplicates), tt.expectedDuplicates)
				for _, dup := range duplicates {
					t.Logf("Duplicate: %s (original: %s)", dup.Pattern, dup.OriginalPath)
				}
			}
		})
	}
}

func TestDuplicatePatternsValidator_Name(t *testing.T) {
	validator := &DuplicatePatternsValidator{}
	expected := "duplicate_patterns"
	if name := validator.Name(); name != expected {
		t.Errorf("Name() = %s, want %s", name, expected)
	}
}
