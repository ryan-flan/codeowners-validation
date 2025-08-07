package parser

import (
	"os"
	"strings"
	"testing"
)

func TestParseCodeOwners(t *testing.T) {
	tests := []struct {
		name           string
		content        string
		expectedRules  int
		expectedErrors int
	}{
		{
			name:           "valid line",
			content:        "src/lib.rs @alice",
			expectedRules:  1,
			expectedErrors: 0,
		},
		{
			name:           "ignores comments and blanks",
			content:        "# comment\n\nsrc/main.rs @bob",
			expectedRules:  1,
			expectedErrors: 0,
		},
		{
			name:           "detects invalid glob",
			content:        "docs/[ @bad",
			expectedRules:  0,
			expectedErrors: 1,
		},
		{
			name:           "trims leading trailing slashes",
			content:        "/foo/ @team",
			expectedRules:  1,
			expectedErrors: 0,
		},
		{
			name:           "parses multiple owners",
			content:        "src/ @alice @bob",
			expectedRules:  1,
			expectedErrors: 0,
		},
		{
			name:           "handles wildcard patterns",
			content:        "*.md @docs-team\n**/*.rs @rust-team",
			expectedRules:  2,
			expectedErrors: 0,
		},
		{
			name:           "handles anchored patterns",
			content:        "/README.md @docs\n/src/ @dev",
			expectedRules:  2,
			expectedErrors: 0,
		},
		{
			name:           "rejects empty pattern",
			content:        "/ @team",
			expectedRules:  0,
			expectedErrors: 1,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			reader := strings.NewReader(tt.content)
			result, err := ParseCodeOwners(reader)
			if err != nil {
				t.Fatalf("ParseCodeOwners() error = %v", err)
			}

			if len(result.Rules) != tt.expectedRules {
				t.Errorf("ParseCodeOwners() got %d rules, want %d", len(result.Rules), tt.expectedRules)
			}

			if len(result.InvalidLines) != tt.expectedErrors {
				t.Errorf("ParseCodeOwners() got %d invalid lines, want %d", len(result.InvalidLines), tt.expectedErrors)
			}
		})
	}
}

func TestParseCodeOwnersSpecificCases(t *testing.T) {
	t.Run("trims leading trailing slashes", func(t *testing.T) {
		reader := strings.NewReader("/foo/ @team")
		result, err := ParseCodeOwners(reader)
		if err != nil {
			t.Fatalf("ParseCodeOwners() error = %v", err)
		}

		if len(result.Rules) != 1 {
			t.Fatalf("Expected 1 rule, got %d", len(result.Rules))
		}

		rule := result.Rules[0]
		if rule.Pattern != "foo" {
			t.Errorf("Expected pattern 'foo', got '%s'", rule.Pattern)
		}
		if rule.OriginalPath != "/foo/" {
			t.Errorf("Expected original path '/foo/', got '%s'", rule.OriginalPath)
		}
	})

	t.Run("parses multiple owners", func(t *testing.T) {
		reader := strings.NewReader("src/ @alice @bob")
		result, err := ParseCodeOwners(reader)
		if err != nil {
			t.Fatalf("ParseCodeOwners() error = %v", err)
		}

		if len(result.Rules) != 1 {
			t.Fatalf("Expected 1 rule, got %d", len(result.Rules))
		}

		rule := result.Rules[0]
		expectedOwners := []string{"@alice", "@bob"}
		if len(rule.Owners) != len(expectedOwners) {
			t.Errorf("Expected %d owners, got %d", len(expectedOwners), len(rule.Owners))
		}
		for i, expected := range expectedOwners {
			if rule.Owners[i] != expected {
				t.Errorf("Expected owner %s, got %s", expected, rule.Owners[i])
			}
		}
	})

	t.Run("handles wildcard patterns", func(t *testing.T) {
		reader := strings.NewReader("*.md @docs-team\n**/*.rs @rust-team")
		result, err := ParseCodeOwners(reader)
		if err != nil {
			t.Fatalf("ParseCodeOwners() error = %v", err)
		}

		if len(result.Rules) != 2 {
			t.Fatalf("Expected 2 rules, got %d", len(result.Rules))
		}
		if len(result.InvalidLines) != 0 {
			t.Errorf("Expected no invalid lines, got %d", len(result.InvalidLines))
		}

		if result.Rules[0].Pattern != "*.md" {
			t.Errorf("Expected first pattern '*.md', got '%s'", result.Rules[0].Pattern)
		}
		if result.Rules[1].Pattern != "**/*.rs" {
			t.Errorf("Expected second pattern '**/*.rs', got '%s'", result.Rules[1].Pattern)
		}
	})

	t.Run("handles anchored patterns", func(t *testing.T) {
		reader := strings.NewReader("/README.md @docs\n/src/ @dev")
		result, err := ParseCodeOwners(reader)
		if err != nil {
			t.Fatalf("ParseCodeOwners() error = %v", err)
		}

		if len(result.Rules) != 2 {
			t.Fatalf("Expected 2 rules, got %d", len(result.Rules))
		}

		rule1 := result.Rules[0]
		if rule1.Pattern != "README.md" {
			t.Errorf("Expected pattern 'README.md', got '%s'", rule1.Pattern)
		}
		if rule1.OriginalPath != "/README.md" {
			t.Errorf("Expected original path '/README.md', got '%s'", rule1.OriginalPath)
		}

		rule2 := result.Rules[1]
		if rule2.Pattern != "src" {
			t.Errorf("Expected pattern 'src', got '%s'", rule2.Pattern)
		}
		if rule2.OriginalPath != "/src/" {
			t.Errorf("Expected original path '/src/', got '%s'", rule2.OriginalPath)
		}
	})
}

func TestParseCodeOwnersFile(t *testing.T) {
	// Create a temporary file for testing
	content := "src/lib.rs @alice\n*.md @docs"
	tmpfile, err := os.CreateTemp("", "codeowners_test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.Remove(tmpfile.Name())

	if _, err := tmpfile.Write([]byte(content)); err != nil {
		t.Fatal(err)
	}
	if err := tmpfile.Close(); err != nil {
		t.Fatal(err)
	}

	result, err := ParseCodeOwnersFile(tmpfile.Name())
	if err != nil {
		t.Fatalf("ParseCodeOwnersFile() error = %v", err)
	}

	if len(result.Rules) != 2 {
		t.Errorf("ParseCodeOwnersFile() got %d rules, want 2", len(result.Rules))
	}

	if len(result.InvalidLines) != 0 {
		t.Errorf("ParseCodeOwnersFile() got %d invalid lines, want 0", len(result.InvalidLines))
	}
}

func TestParseCodeOwnersFileNotExists(t *testing.T) {
	_, err := ParseCodeOwnersFile("nonexistent_file.txt")
	if err == nil {
		t.Error("ParseCodeOwnersFile() expected error for non-existent file")
	}
}
