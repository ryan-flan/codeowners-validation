package validator

import (
	"os"
	"path/filepath"
	"testing"

	"github.com/ryan-flan/codeowners-validation/internal/parser"
)

func TestExistsValidator_Validate(t *testing.T) {
	// Create a temporary directory for testing
	tmpDir, err := os.MkdirTemp("", "codeowners_test")
	if err != nil {
		t.Fatal(err)
	}
	defer os.RemoveAll(tmpDir)

	// Create test files and directories
	if err := os.WriteFile(filepath.Join(tmpDir, "exists.txt"), []byte("content"), 0644); err != nil {
		t.Fatal(err)
	}

	srcDir := filepath.Join(tmpDir, "src")
	if err := os.MkdirAll(srcDir, 0755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(srcDir, "main.rs"), []byte("fn main() {}"), 0644); err != nil {
		t.Fatal(err)
	}

	docsDir := filepath.Join(tmpDir, "docs")
	if err := os.MkdirAll(docsDir, 0755); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(docsDir, "README.md"), []byte("# Docs"), 0644); err != nil {
		t.Fatal(err)
	}
	if err := os.WriteFile(filepath.Join(tmpDir, "foo.md"), []byte("docs"), 0644); err != nil {
		t.Fatal(err)
	}

	validator := &ExistsValidator{RepoPath: tmpDir}

	tests := []struct {
		name            string
		rules           []parser.CodeOwnerRule
		expectedMissing int
	}{
		{
			name: "detects missing file",
			rules: []parser.CodeOwnerRule{
				makeRule("missing.txt", "missing.txt"),
			},
			expectedMissing: 1,
		},
		{
			name: "passes existing file",
			rules: []parser.CodeOwnerRule{
				makeRule("exists.txt", "exists.txt"),
			},
			expectedMissing: 0,
		},
		{
			name: "matches wildcard files",
			rules: []parser.CodeOwnerRule{
				makeRule("*.md", "*.md"),
			},
			expectedMissing: 0,
		},
		{
			name: "detects unmatched wildcards",
			rules: []parser.CodeOwnerRule{
				makeRule("*.xyz", "*.xyz"),
			},
			expectedMissing: 1,
		},
		{
			name: "handles anchored patterns",
			rules: []parser.CodeOwnerRule{
				makeRule("src/main.rs", "/src/main.rs"),
				makeRule("main.rs", "main.rs"),
			},
			expectedMissing: 0,
		},
		{
			name: "handles directory patterns",
			rules: []parser.CodeOwnerRule{
				makeRule("docs", "docs/"),
				makeRule("docs", "/docs/"),
			},
			expectedMissing: 0,
		},
		{
			name: "handles nested patterns",
			rules: []parser.CodeOwnerRule{
				makeRule("main.rs", "main.rs"),  // Should find src/main.rs
				makeRule("main.rs", "/main.rs"), // Should not find anything (not in root)
			},
			expectedMissing: 1, // Only the anchored one should fail
		},
		{
			name: "handles complex wildcards",
			rules: []parser.CodeOwnerRule{
				makeRule("**/*.rs", "**/*.rs"),
			},
			expectedMissing: 0,
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			missing, err := validator.Validate(tt.rules)
			if err != nil {
				t.Fatalf("Validate() error = %v", err)
			}

			if len(missing) != tt.expectedMissing {
				t.Errorf("Validate() returned %d missing files, want %d", len(missing), tt.expectedMissing)
				for _, rule := range missing {
					t.Logf("Missing: %s (original: %s)", rule.Pattern, rule.OriginalPath)
				}
			}
		})
	}
}

func TestExistsValidator_buildGlobPatterns(t *testing.T) {
	validator := &ExistsValidator{}

	tests := []struct {
		name     string
		rule     parser.CodeOwnerRule
		expected []string
	}{
		{
			name:     "anchored directory",
			rule:     makeRule("docs", "/docs/"),
			expected: []string{"docs", "docs/**"},
		},
		{
			name:     "anchored file",
			rule:     makeRule("README.md", "/README.md"),
			expected: []string{"README.md"},
		},
		{
			name:     "non-anchored directory",
			rule:     makeRule("lib", "lib/"),
			expected: []string{"**/lib", "**/lib/**"},
		},
		{
			name:     "non-anchored wildcard",
			rule:     makeRule("*.rs", "*.rs"),
			expected: []string{"*.rs"},
		},
		{
			name:     "non-anchored plain file",
			rule:     makeRule("config.json", "config.json"),
			expected: []string{"**/config.json"},
		},
		{
			name:     "complex wildcard",
			rule:     makeRule("**/*.test.js", "**/*.test.js"),
			expected: []string{"**/*.test.js"},
		},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			result := validator.buildGlobPatterns(tt.rule)
			if len(result) != len(tt.expected) {
				t.Fatalf("buildGlobPatterns() returned %d patterns, want %d", len(result), len(tt.expected))
			}
			for i, expected := range tt.expected {
				if result[i] != expected {
					t.Errorf("buildGlobPatterns()[%d] = %s, want %s", i, result[i], expected)
				}
			}
		})
	}
}
