package validator

import (
	"fmt"

	"github.com/ryan-flan/codeowners-validation/internal/parser"
)

// DuplicatePatternsValidator validates that there are no duplicate patterns
type DuplicatePatternsValidator struct{}

// Name returns the validator name
func (v *DuplicatePatternsValidator) Name() string {
	return "duplicate_patterns"
}

// Validate finds duplicate patterns in the rules with optimized memory allocation
func (v *DuplicatePatternsValidator) Validate(rules []parser.CodeOwnerRule) ([]parser.CodeOwnerRule, error) {
	patternSet := make(map[string]bool, len(rules))
	originalPathSet := make(map[string]bool, len(rules))
	duplicates := make([]parser.CodeOwnerRule, 0, len(rules)/20) // Estimate ~5% duplicates

	for _, rule := range rules {
		isOriginalPathDuplicate := originalPathSet[rule.OriginalPath]
		isPatternDuplicate := patternSet[rule.Pattern]

		if isOriginalPathDuplicate || isPatternDuplicate {
			duplicates = append(duplicates, rule)

			// Only print a warning if it is the normalized duplicate
			if !isOriginalPathDuplicate && isPatternDuplicate {
				fmt.Println("Warning: Duplicate pattern found in normalized pattern")
				fmt.Println("Please raise an issue if this seems incorrect.")
				fmt.Printf("Pattern: %s\n", rule.Pattern)
				fmt.Printf("Original: %s\n", rule.OriginalPath)
			}
		}

		originalPathSet[rule.OriginalPath] = true
		patternSet[rule.Pattern] = true
	}

	return duplicates, nil
}
